use crate::tui::layout::ScreenClass;
use crate::tui::state::{SignalPhase, TunerUiState};
use crate::tui::theme;
use crate::tui::widgets::tune_gauge;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, area: Rect, state: &TunerUiState, screen_class: ScreenClass) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::chrome())
        .title(Line::from(vec![
            Span::styled(" ", theme::label()),
            Span::styled("PITCH//CORE", theme::title()),
            Span::styled(" ", theme::label()),
        ]));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let sections = if matches!(screen_class, ScreenClass::Compact) {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Min(4),
                Constraint::Length(1),
            ])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(5),
                Constraint::Length(1),
            ])
            .split(inner)
    };

    frame.render_widget(
        Paragraph::new(note_block(state, screen_class)).alignment(Alignment::Center),
        sections[0],
    );
    frame.render_widget(
        Paragraph::new(metrics_line(state, screen_class)).alignment(Alignment::Center),
        sections[1],
    );
    tune_gauge::render(frame, sections[2], state, screen_class);
    frame.render_widget(
        Paragraph::new(status_line(state, screen_class)).alignment(Alignment::Center),
        sections[3],
    );
}

fn note_block(state: &TunerUiState, screen_class: ScreenClass) -> Vec<Line<'static>> {
    if matches!(screen_class, ScreenClass::Compact) {
        vec![Line::from(Span::styled(
            format!("[ {} ]", state.note_label),
            theme::note(state.phase),
        ))]
    } else {
        vec![
            Line::from(Span::styled("DETECTED NOTE", theme::muted())),
            Line::from(Span::styled(
                format!("[ {} ]", state.note_label),
                theme::note(state.phase),
            )),
        ]
    }
}

fn metrics_line(state: &TunerUiState, screen_class: ScreenClass) -> Line<'static> {
    let offset_style = match state.phase {
        SignalPhase::NoSignal => theme::label(),
        _ => theme::emphasis(state.phase),
    };
    let target_display = state
        .string_label
        .as_ref()
        .map(|label| format!("{label} {}", state.target_label))
        .unwrap_or_else(|| state.target_label.clone());

    if matches!(screen_class, ScreenClass::Compact) {
        Line::from(vec![
            Span::styled(format_frequency(state.frequency_hz), theme::telemetry()),
            Span::styled("   ", theme::label()),
            Span::styled(format_cents(state.cents_off, state.phase), offset_style),
            Span::styled("   ", theme::label()),
            Span::styled(target_display, theme::note(state.phase)),
        ])
    } else {
        Line::from(vec![
            Span::styled("FREQ ", theme::label()),
            Span::styled(format_frequency(state.frequency_hz), theme::telemetry()),
            Span::styled("   OFFSET ", theme::label()),
            Span::styled(format_cents(state.cents_off, state.phase), offset_style),
            Span::styled("   TARGET ", theme::label()),
            Span::styled(target_display, theme::note(state.phase)),
        ])
    }
}

fn status_line(state: &TunerUiState, screen_class: ScreenClass) -> Line<'static> {
    let vector = direction_vector(state.phase, state.animation_tick);
    if matches!(screen_class, ScreenClass::Compact) {
        Line::from(vec![
            Span::styled(vector, theme::muted()),
            Span::styled(" ", theme::muted()),
            Span::styled(state.status_line, theme::emphasis(state.phase)),
            Span::styled("   ", theme::label()),
            Span::styled(state.direction_label(), theme::note(state.phase)),
        ])
    } else {
        Line::from(vec![
            Span::styled(vector, theme::muted()),
            Span::styled("   ", theme::muted()),
            Span::styled("STATUS ", theme::label()),
            Span::styled(state.status_line, theme::emphasis(state.phase)),
            Span::styled("   ACTION ", theme::label()),
            Span::styled(state.direction_label(), theme::note(state.phase)),
        ])
    }
}

fn direction_vector(phase: SignalPhase, animation_tick: u64) -> &'static str {
    match phase {
        SignalPhase::TooLow => {
            if animation_tick % 6 < 3 {
                ">>>"
            } else {
                " > "
            }
        }
        SignalPhase::TooHigh => {
            if animation_tick % 6 < 3 {
                "<<<"
            } else {
                " < "
            }
        }
        SignalPhase::Searching => "[~]",
        SignalPhase::Unstable => "[!]",
        SignalPhase::InTune => "[◎]",
        SignalPhase::NoSignal => "[ ]",
    }
}

fn format_frequency(frequency_hz: Option<f32>) -> String {
    frequency_hz
        .map(|value| format!("{value:>6.2} Hz"))
        .unwrap_or_else(|| "--.-- Hz".to_string())
}

fn format_cents(cents_off: f32, phase: SignalPhase) -> String {
    match phase {
        SignalPhase::NoSignal => "--.- c".to_string(),
        _ => format!("{cents_off:+.1} c"),
    }
}
