mod demo;
mod gauge;
mod layout;
mod render;
mod state;
mod theme;
mod widgets;

use crate::tui::demo::DemoState;
use crate::tui::render::render;
use crate::tui::state::{HistoryBuffer, TunerUiState, output_to_ui_state};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, Stdout};
use std::sync::mpsc::Receiver;
use std::time::Duration;
use tuner_core::{PresetId, TunerMode};
use tuner_dsp_native::{AudioCapture, AudioInputKind};
use tuner_engine::{PitchDetectorConfig, SessionMode, TunerConfig, TunerEngine};

const LIVE_LOOP_TICK: Duration = Duration::from_millis(16);
const DEMO_LOOP_TICK: Duration = Duration::from_millis(50);
const OVERLAY_TTL_FRAMES: u16 = 90;

pub fn run_live_tui(initial_mode: SessionMode, calibration_hz: f32) -> io::Result<()> {
    let detector_config = PitchDetectorConfig::default();
    let mut audio_capture = AudioCapture::new(
        detector_config.frame_size,
        detector_config.hop_size,
        detector_config.sample_rate,
    );
    let audio_start = audio_capture
        .start()
        .map_err(|error| io::Error::other(error.to_string()))?;
    let detector_config = PitchDetectorConfig {
        sample_rate: audio_start.sample_rate_hz,
        ..detector_config
    };
    let mut tuner_config = TunerConfig::from_detector_config(detector_config, initial_mode);
    tuner_config.calibration_hz = calibration_hz;
    let mut tuner_engine = TunerEngine::new(detector_config, tuner_config);

    let mut terminal = init_terminal()?;
    let _terminal_guard = TerminalGuard::new(&mut terminal)?;
    let result = run_live_loop(
        &mut terminal,
        &mut tuner_engine,
        &audio_start.receiver,
        audio_start.input_kind,
        initial_mode,
        audio_start.sample_rate_hz,
    );

    audio_capture.stop();
    result
}

pub fn run_demo_tui() -> io::Result<()> {
    let mut terminal = init_terminal()?;
    let _terminal_guard = TerminalGuard::new(&mut terminal)?;
    let demo = DemoState::new();
    let mut help_visible = false;

    loop {
        if let Some(action) = poll_ui_action(DEMO_LOOP_TICK)? {
            match action {
                UiAction::Quit => return Ok(()),
                UiAction::ToggleHelp => help_visible = !help_visible,
                UiAction::CloseOverlay => {
                    if help_visible {
                        help_visible = false;
                    } else {
                        return Ok(());
                    }
                }
                UiAction::ToggleMode
                | UiAction::OpenPresetPicker
                | UiAction::PickerUp
                | UiAction::PickerDown
                | UiAction::Confirm
                | UiAction::None => {}
            }
        }

        let mut frame_state = demo.current_frame();
        frame_state.overlays.help_visible = help_visible;
        terminal.draw(|frame| render(frame, &frame_state))?;
    }
}

fn run_live_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    tuner_engine: &mut TunerEngine,
    receiver: &Receiver<Vec<f32>>,
    _input_kind: AudioInputKind,
    initial_mode: SessionMode,
    sample_rate_hz: u32,
) -> io::Result<()> {
    let mut history = HistoryBuffer::new();
    let mut animation_tick = 0_u64;
    let mut current_mode = initial_mode;
    let mut overlay_notice: Option<OverlayNotice> = None;
    let mut help_visible = false;
    let mut preset_picker = PresetPicker::new(current_mode.current_preset());
    let mut ui_state = TunerUiState::no_signal(
        current_mode,
        sample_rate_hz,
        history.snapshot(),
        animation_tick,
    );

    loop {
        while let Some(action) = poll_ui_action(Duration::from_millis(0))? {
            if preset_picker.visible {
                match action {
                    UiAction::Quit => {
                        preset_picker.visible = false;
                    }
                    UiAction::ToggleHelp => {
                        help_visible = !help_visible;
                    }
                    UiAction::PickerUp => preset_picker.move_up(),
                    UiAction::PickerDown => preset_picker.move_down(),
                    UiAction::Confirm => {
                        if let Some(preset_id) = preset_picker.selected_preset_id() {
                            let applied = tuner_engine.select_preset(preset_id);
                            current_mode = tuner_engine.session_mode();
                            preset_picker.sync_to(current_mode.current_preset());
                            preset_picker.visible = false;
                            history = HistoryBuffer::new();
                            overlay_notice = Some(OverlayNotice::new(format!(
                                "PRESET: {}",
                                applied.display_name().to_ascii_uppercase()
                            )));
                            animation_tick = animation_tick.wrapping_add(1);
                            ui_state = TunerUiState::no_signal(
                                current_mode,
                                sample_rate_hz,
                                history.snapshot(),
                                animation_tick,
                            );
                        }
                    }
                    UiAction::OpenPresetPicker => {}
                    UiAction::ToggleMode | UiAction::CloseOverlay | UiAction::None => {}
                }
                continue;
            }

            match action {
                UiAction::Quit => return Ok(()),
                UiAction::ToggleHelp => help_visible = !help_visible,
                UiAction::CloseOverlay => {
                    if help_visible {
                        help_visible = false;
                    } else {
                        return Ok(());
                    }
                }
                UiAction::OpenPresetPicker if current_mode.is_analyze() => {
                    preset_picker.visible = true;
                    preset_picker.sync_to(current_mode.current_preset());
                }
                UiAction::ToggleMode => {
                    if let Some(message) =
                        handle_live_action(action, tuner_engine, &mut current_mode, &mut history)
                    {
                        overlay_notice = Some(OverlayNotice::new(message));
                        animation_tick = animation_tick.wrapping_add(1);
                        ui_state = TunerUiState::no_signal(
                            current_mode,
                            sample_rate_hz,
                            history.snapshot(),
                            animation_tick,
                        );
                    }
                }
                UiAction::PickerUp
                | UiAction::PickerDown
                | UiAction::Confirm
                | UiAction::OpenPresetPicker
                | UiAction::None => {}
            }
        }

        while let Ok(frame) = receiver.try_recv() {
            let output = tuner_engine.process_frame(&frame);
            history.record(&output);
            animation_tick = animation_tick.wrapping_add(1);
            ui_state = output_to_ui_state(
                &output,
                current_mode,
                sample_rate_hz,
                history.snapshot(),
                animation_tick,
            );
        }

        animation_tick = animation_tick.wrapping_add(1);
        ui_state.animation_tick = animation_tick;
        ui_state.overlays.notice = overlay_notice.as_ref().map(|notice| notice.message.clone());
        ui_state.overlays.help_visible = help_visible;
        ui_state.overlays.preset_picker.visible = preset_picker.visible;
        ui_state.overlays.preset_picker.items = preset_picker.items();
        ui_state.overlays.preset_picker.selected_index = preset_picker.selected_index;
        if let Some(notice) = overlay_notice.as_mut() {
            if notice.ttl_frames > 0 {
                notice.ttl_frames -= 1;
            }
            if notice.ttl_frames == 0 {
                overlay_notice = None;
            }
        }
        terminal.draw(|frame| render(frame, &ui_state))?;
        std::thread::sleep(LIVE_LOOP_TICK);
    }
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn poll_ui_action(timeout: Duration) -> io::Result<Option<UiAction>> {
    if !event::poll(timeout)? {
        return Ok(None);
    }

    let event = event::read()?;
    Ok(match event {
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => match key_event.code {
            KeyCode::Char('q') => Some(UiAction::Quit),
            KeyCode::Esc => Some(UiAction::CloseOverlay),
            KeyCode::Char('h') | KeyCode::Char('?') => Some(UiAction::ToggleHelp),
            KeyCode::Char('m') => Some(UiAction::ToggleMode),
            KeyCode::Char('p') | KeyCode::Char('P') => Some(UiAction::OpenPresetPicker),
            KeyCode::Up => Some(UiAction::PickerUp),
            KeyCode::Down => Some(UiAction::PickerDown),
            KeyCode::Enter => Some(UiAction::Confirm),
            _ => Some(UiAction::None),
        },
        _ => None,
    })
}

fn handle_live_action(
    action: UiAction,
    tuner_engine: &mut TunerEngine,
    current_mode: &mut SessionMode,
    history: &mut HistoryBuffer,
) -> Option<String> {
    match action {
        UiAction::ToggleMode if current_mode.is_analyze() => {
            let next_mode = tuner_engine.toggle_analyze_mode()?;
            *current_mode = next_mode;
            *history = HistoryBuffer::new();
            Some(mode_notice(next_mode))
        }
        _ => None,
    }
}

fn mode_notice(mode: SessionMode) -> String {
    match mode {
        SessionMode::Analyze(TunerMode::Chromatic) => "MODE: CHROMATIC".to_string(),
        SessionMode::Analyze(TunerMode::Preset(preset_id)) => format!(
            "MODE: PRESET // {}",
            preset_id.display_name().to_ascii_uppercase()
        ),
        SessionMode::Target(note) => format!("MODE: TARGET {}", note.label()),
    }
}

#[derive(Debug, Clone)]
struct OverlayNotice {
    message: String,
    ttl_frames: u16,
}

impl OverlayNotice {
    fn new(message: String) -> Self {
        Self {
            message,
            ttl_frames: OVERLAY_TTL_FRAMES,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UiAction {
    Quit,
    CloseOverlay,
    ToggleHelp,
    ToggleMode,
    OpenPresetPicker,
    PickerUp,
    PickerDown,
    Confirm,
    None,
}

#[derive(Debug, Clone)]
struct PresetPicker {
    visible: bool,
    selected_index: usize,
}

impl PresetPicker {
    fn new(current: Option<PresetId>) -> Self {
        let mut picker = Self {
            visible: false,
            selected_index: 0,
        };
        picker.sync_to(current);
        picker
    }

    fn sync_to(&mut self, current: Option<PresetId>) {
        if let Some(preset_id) = current
            && let Some(index) = PresetId::ALL
                .iter()
                .position(|candidate| *candidate == preset_id)
        {
            self.selected_index = index;
        }
    }

    fn move_up(&mut self) {
        if self.selected_index == 0 {
            self.selected_index = PresetId::ALL.len().saturating_sub(1);
        } else {
            self.selected_index -= 1;
        }
    }

    fn move_down(&mut self) {
        self.selected_index = (self.selected_index + 1) % PresetId::ALL.len();
    }

    fn selected_preset_id(&self) -> Option<PresetId> {
        PresetId::ALL.get(self.selected_index).copied()
    }

    fn items(&self) -> Vec<(PresetId, String)> {
        PresetId::ALL
            .iter()
            .copied()
            .map(|preset_id| (preset_id, preset_id.display_name().to_string()))
            .collect()
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn new(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<Self> {
        terminal.clear()?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}
