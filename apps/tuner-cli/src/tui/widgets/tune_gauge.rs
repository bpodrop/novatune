use crate::tui::gauge::build_gauge_rows;
use crate::tui::layout::ScreenClass;
use crate::tui::state::{DEFAULT_IN_TUNE_THRESHOLD_CENTS, TunerUiState};
use crate::tui::theme;
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, area: Rect, state: &TunerUiState, screen_class: ScreenClass) {
    let compact = matches!(screen_class, ScreenClass::Compact);
    let gauge_width = gauge_width(area.width, compact);
    let rows = build_gauge_rows(
        state.pitch.cents_off,
        state.pitch.phase,
        gauge_width,
        DEFAULT_IN_TUNE_THRESHOLD_CENTS,
        compact,
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::chrome())
        .title(Line::styled(" TUNE VECTOR ", theme::label()));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(rows.scale, theme::muted()),
            Line::styled(rows.marker, theme::emphasis(state.pitch.phase)),
            Line::styled(rows.legend, theme::telemetry()),
        ])
        .alignment(Alignment::Center),
        inner,
    );
}

fn gauge_width(area_width: u16, compact: bool) -> u16 {
    let margin = if compact { 12 } else { 18 };
    let raw = area_width.saturating_sub(margin).clamp(17, 33);
    if raw % 2 == 0 { raw - 1 } else { raw }
}
