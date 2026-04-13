use crate::ui::theme;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::{Frame, Terminal};
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

const FRAME_TICK: Duration = Duration::from_millis(45);
const BOOT_STEP_MS: u64 = 320;
const HANDOFF_DURATION: Duration = Duration::from_millis(520);
const BOOT_LINES: [&str; 6] = [
    "KERNEL CLOCK",
    "TERM LINK",
    "AUDIO LINK",
    "JACK DETECTION",
    "DSP ENGINE",
    "NOISE FILTER",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplashOutcome {
    Finished,
    Aborted,
}

#[derive(Debug, Clone)]
pub struct SplashScreen {
    started_at: Instant,
    finished: bool,
    handoff_started_at: Option<Instant>,
}

impl SplashScreen {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            finished: false,
            handoff_started_at: None,
        }
    }

    pub fn tick(&mut self) {
        if self.completed_steps() >= BOOT_LINES.len() && self.handoff_started_at.is_none() {
            self.handoff_started_at = Some(Instant::now());
        }

        if let Some(handoff_started_at) = self.handoff_started_at
            && handoff_started_at.elapsed() >= HANDOFF_DURATION
        {
            self.finished = true;
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    fn tick_index(&self) -> u64 {
        (self.elapsed().as_millis() / FRAME_TICK.as_millis()) as u64
    }

    fn completed_steps(&self) -> usize {
        let count = (self.elapsed().as_millis() / BOOT_STEP_MS as u128) as usize;
        count.min(BOOT_LINES.len())
    }

    fn active_step(&self) -> Option<usize> {
        if self.is_handoff_phase() {
            None
        } else {
            let completed = self.completed_steps();
            (completed < BOOT_LINES.len()).then_some(completed)
        }
    }

    fn is_handoff_phase(&self) -> bool {
        self.handoff_started_at.is_some() && !self.finished
    }

    fn handoff_progress(&self) -> f32 {
        self.handoff_started_at
            .map(|started_at| {
                (started_at.elapsed().as_secs_f32() / HANDOFF_DURATION.as_secs_f32())
                    .clamp(0.0, 1.0)
            })
            .unwrap_or(0.0)
    }

    fn offset_label(&self) -> &'static str {
        if self.tick_index() % 10 < 5 {
            "+01c"
        } else {
            "+00c"
        }
    }
}

pub fn run_splashscreen() -> io::Result<SplashOutcome> {
    let mut terminal = init_terminal()?;
    let _terminal_guard = TerminalGuard::new(&mut terminal)?;
    let mut splash = SplashScreen::new();

    loop {
        while event::poll(Duration::from_millis(0))? {
            let event = event::read()?;
            if let Event::Key(key_event) = event {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                if matches!(key_event.code, KeyCode::Char('q') | KeyCode::Esc) {
                    return Ok(SplashOutcome::Aborted);
                }
            }
        }

        splash.tick();
        terminal.draw(|frame| render(frame, &splash))?;

        if splash.is_finished() {
            return Ok(SplashOutcome::Finished);
        }

        std::thread::sleep(FRAME_TICK);
    }
}

fn render(frame: &mut Frame, splash: &SplashScreen) {
    frame.render_widget(Clear, frame.area());
    frame.render_widget(Block::default().style(theme::background()), frame.area());

    let root = build_layout(frame.area());
    render_header(frame, root[0], splash);
    render_boot_bus(frame, root[1], splash);
    render_signal_snapshot(frame, root[2], splash);
    render_footer(frame, root[3], splash);
}

fn build_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(10),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(area.inner(ratatui::layout::Margin {
            vertical: if area.height > 22 { 1 } else { 0 },
            horizontal: if area.width > 72 { 2 } else { 1 },
        }))
        .to_vec()
}

fn render_header(frame: &mut Frame, area: Rect, splash: &SplashScreen) {
    let glitch = splash.tick_index().is_multiple_of(11);
    let title = if glitch {
        "RUSTUNER_2077 :: BOOT SYS"
    } else {
        "RUSTUNER_2077 :: BOOT SYSTEM"
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::chrome());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(title, theme::amber()))).alignment(Alignment::Left),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("NODE: ", theme::muted()),
            Span::styled("LOCAL", theme::cyan()),
            Span::styled("   PROFILE: ", theme::muted()),
            Span::styled("Chromatic / Preset / Live Input Monitor", theme::subtitle()),
        ])),
        rows[1],
    );
}

fn render_boot_bus(frame: &mut Frame, area: Rect, splash: &SplashScreen) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::chrome())
        .title(Line::from(vec![
            Span::styled(" ", theme::muted()),
            Span::styled("INIT BUS", theme::cyan()),
            Span::styled(" ", theme::muted()),
        ]));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let constraints = vec![Constraint::Length(1); BOOT_LINES.len()];
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let completed = splash.completed_steps();
    let active = splash.active_step();
    let cursor = if splash.tick_index() % 8 < 4 {
        "_"
    } else {
        " "
    };

    for (index, label) in BOOT_LINES.iter().enumerate() {
        let (prefix, style) = if completed > index || splash.is_handoff_phase() {
            ("[OK]", theme::ok())
        } else if active == Some(index) {
            ("[..]", theme::accent(splash.tick_index().is_multiple_of(2)))
        } else {
            ("[  ]", theme::pending())
        };

        let suffix = if active == Some(index) && !splash.is_handoff_phase() {
            cursor
        } else {
            ""
        };

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(" ", theme::muted()),
                Span::styled(*label, style),
                Span::styled(suffix, theme::accent(true)),
            ])),
            rows[index],
        );
    }
}

fn render_signal_snapshot(frame: &mut Frame, area: Rect, splash: &SplashScreen) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::chrome())
        .title(Line::from(vec![
            Span::styled(" ", theme::muted()),
            Span::styled("SIGNAL SNAPSHOT", theme::magenta()),
            Span::styled(" ", theme::muted()),
        ]));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("NOTE: ", theme::muted()),
            Span::styled("E2", theme::amber()),
            Span::styled("   OFFSET: ", theme::muted()),
            Span::styled(
                splash.offset_label(),
                theme::accent(splash.tick_index().is_multiple_of(2)),
            ),
            Span::styled("   FREQ: ", theme::muted()),
            Span::styled("82.41 Hz", theme::cyan()),
        ])),
        rows[0],
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "LOW ───────┬──────◎──────┬────── HIGH",
            theme::cyan(),
        ))),
        rows[1],
    );

    frame.render_widget(
        Paragraph::new(build_axis_line(splash.tick_index())),
        rows[2],
    );
}

fn render_footer(frame: &mut Frame, area: Rect, splash: &SplashScreen) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::chrome())
        .title(Line::from(vec![
            Span::styled(" ", theme::muted()),
            Span::styled("STATUS", theme::amber()),
            Span::styled(" ", theme::muted()),
        ]));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let status = if splash.is_handoff_phase() {
        "STATUS :: READY"
    } else {
        "STATUS :: SYNCING"
    };
    let handoff = if splash.is_handoff_phase() {
        if splash.tick_index() % 6 < 3 {
            "HANDOFF :: UI MAIN LOOP"
        } else {
            "HANDOFF // UI MAIN LOOP"
        }
    } else {
        "Q / ESC :: ABORT"
    };

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(status, theme::ok()))),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                handoff,
                theme::handoff(splash.tick_index().is_multiple_of(2)),
            ),
            Span::styled(
                if splash.is_handoff_phase() {
                    format!("   {:>3.0}%", splash.handoff_progress() * 100.0)
                } else {
                    String::new()
                },
                theme::subtitle(),
            ),
        ])),
        rows[1],
    );
}

fn build_axis_line(tick_index: u64) -> Line<'static> {
    let position = (tick_index as usize % 9) + 1;
    let mut spans = Vec::with_capacity(12);
    for index in 0..12 {
        let style = if index == position {
            theme::amber()
        } else if index % 2 == 0 {
            theme::cyan()
        } else {
            theme::magenta()
        };
        let glyph = if index == position { "█" } else { "─" };
        spans.push(Span::styled(glyph, style));
    }
    Line::from(spans)
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
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
