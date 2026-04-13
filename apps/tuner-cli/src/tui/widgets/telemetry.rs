use crate::tui::layout::ScreenClass;
use crate::tui::state::TunerUiState;
use crate::tui::theme;
use crate::tui::widgets::history;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, area: Rect, state: &TunerUiState, screen_class: ScreenClass) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::chrome())
        .title(Line::styled(" TELEMETRY ", theme::label()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    frame.render_widget(
        Paragraph::new(primary_metrics(state, screen_class)),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new(secondary_metrics(state, screen_class)),
        rows[1],
    );
}

fn primary_metrics(state: &TunerUiState, screen_class: ScreenClass) -> Line<'static> {
    let mut spans = vec![
        Span::styled("TARGET ", theme::label()),
        Span::styled(state.target_label.clone(), theme::telemetry()),
        Span::styled("   TARGET Hz ", theme::label()),
        Span::styled(format_frequency(state.target_hz), theme::telemetry()),
        Span::styled("   STABILITY ", theme::label()),
        Span::styled(format!("{}%", state.stability_percent), theme::telemetry()),
    ];

    if !matches!(screen_class, ScreenClass::Compact) {
        spans.extend([
            Span::styled("   NOISE ", theme::label()),
            Span::styled(format_noise(state.noise_percent), theme::telemetry()),
            Span::styled("   CONF ", theme::label()),
            Span::styled(format_confidence(state.confidence), theme::telemetry()),
        ]);
    }

    Line::from(spans)
}

fn secondary_metrics(state: &TunerUiState, screen_class: ScreenClass) -> Line<'static> {
    let mut spans = vec![
        Span::styled("HISTORY ", theme::label()),
        Span::styled(history::sparkline(&state.history), theme::telemetry()),
        Span::styled("   ACTION ", theme::label()),
        Span::styled(state.direction_label(), theme::emphasis(state.phase)),
    ];

    if !matches!(screen_class, ScreenClass::Compact) {
        spans.extend([
            Span::styled("   PHASE ", theme::label()),
            Span::styled(
                state.badge_label(),
                theme::badge(state.phase, state.animation_tick),
            ),
        ]);
    }

    Line::from(spans)
}

fn format_noise(noise_percent: Option<u8>) -> String {
    noise_percent
        .map(|value| format!("{value}%"))
        .unwrap_or_else(|| "--%".to_string())
}

fn format_frequency(frequency_hz: Option<f32>) -> String {
    frequency_hz
        .map(|value| format!("{value:>6.2}"))
        .unwrap_or_else(|| "--.--".to_string())
}

fn format_confidence(confidence: Option<f32>) -> String {
    confidence
        .map(|value| format!("{:>3.0}%", value * 100.0))
        .unwrap_or_else(|| "--%".to_string())
}
