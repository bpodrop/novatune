use crate::tui::state::TunerUiState;
use crate::tui::theme;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, area: Rect, state: &TunerUiState) {
    let live_label = if state.animation_tick.is_multiple_of(19) {
        " // LIVE_SCAN "
    } else {
        " // LIVE INPUT "
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::chrome())
        .title(Line::from(vec![
            Span::styled(" ", theme::label()),
            Span::styled(state.header.app_label, theme::title()),
            Span::styled(live_label, theme::label()),
        ]));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Length(18)])
        .split(rows[0]);
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(18),
            Constraint::Length(16),
        ])
        .split(rows[1]);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("AUDIO FEED ", theme::label()),
            Span::styled("CHANNEL_01", theme::telemetry()),
            Span::styled("   DEVICE ", theme::label()),
            Span::styled("JACK IN", theme::telemetry()),
            Span::styled("   HELP ", theme::label()),
            Span::styled("H", theme::telemetry()),
        ])),
        top[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {} ", state.badge_label()),
            theme::badge(state.pitch.phase, state.animation_tick),
        )))
        .alignment(Alignment::Right),
        top[1],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("PROFILE ", theme::label()),
            Span::styled(&state.header.profile_label, theme::telemetry()),
            Span::styled("   MODE ", theme::label()),
            Span::styled(&state.header.mode_label, theme::telemetry()),
        ])),
        bottom[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("SR ", theme::label()),
            Span::styled(
                format!("{:.1}kHz", state.header.sample_rate_hz as f32 / 1000.0),
                theme::telemetry(),
            ),
        ])),
        bottom[1],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("SIGNAL ", theme::label()),
            Span::styled(
                state.header.signal_label,
                theme::badge(state.pitch.phase, state.animation_tick),
            ),
        ]))
        .alignment(Alignment::Right),
        bottom[2],
    );
}
