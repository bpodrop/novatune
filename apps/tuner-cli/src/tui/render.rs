use crate::tui::layout;
use crate::tui::state::TunerUiState;
use crate::tui::theme;
use crate::tui::widgets::{header, pitch_hero, telemetry};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::widgets::{Block, Borders, Clear};

pub fn render(frame: &mut Frame, ui_state: &TunerUiState) {
    let root = layout::build(frame.area());
    frame.render_widget(Clear, frame.area());
    frame.render_widget(Block::default().style(theme::background()), frame.area());
    header::render(frame, root.header, ui_state);
    pitch_hero::render(frame, root.hero, ui_state, root.screen_class);
    telemetry::render(frame, root.footer, ui_state, root.screen_class);
    render_overlay(frame, ui_state);
    render_help(frame, ui_state);
    render_preset_picker(frame, ui_state);
}

fn render_overlay(frame: &mut Frame, ui_state: &TunerUiState) {
    let Some(message) = ui_state.overlay_notice.as_ref() else {
        return;
    };

    let width = (message.len() as u16 + 4).min(frame.area().width.saturating_sub(2));
    if width < 8 || frame.area().height < 3 {
        return;
    }

    let area = Rect {
        x: frame.area().x + frame.area().width.saturating_sub(width) / 2,
        y: frame.area().y + 1,
        width,
        height: 1,
    };

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {message} ",),
            theme::overlay(),
        )))
        .alignment(Alignment::Center),
        area,
    );
}

fn render_help(frame: &mut Frame, ui_state: &TunerUiState) {
    if !ui_state.help_visible || frame.area().width < 40 || frame.area().height < 12 {
        return;
    }

    let outer = centered_rect(frame.area(), 56, 11);
    frame.render_widget(Clear, outer);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::chrome())
        .title(Line::from(vec![
            Span::styled(" ", theme::label()),
            Span::styled("KEYMAP", theme::title()),
            Span::styled(" ", theme::label()),
        ]));
    let inner = block.inner(outer);
    frame.render_widget(block, outer);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let lines = [
        "H / ?      Toggle this help",
        "Q / Esc    Quit",
        "M          Toggle Chromatic / Preset",
        "P          Open preset picker",
        "Up / Down  Navigate preset picker",
        "Enter      Apply selected preset",
        "Esc        Close help or picker",
    ];

    for (area, line) in rows.iter().zip(lines.iter()) {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(*line, theme::telemetry()))),
            *area,
        );
    }
}

fn render_preset_picker(frame: &mut Frame, ui_state: &TunerUiState) {
    if !ui_state.preset_picker_visible || ui_state.preset_picker_items.is_empty() {
        return;
    }

    let height =
        (ui_state.preset_picker_items.len() as u16 + 4).min(frame.area().height.saturating_sub(2));
    let outer = centered_rect(frame.area(), 40, height);
    frame.render_widget(Clear, outer);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::chrome())
        .title(Line::from(vec![
            Span::styled(" ", theme::label()),
            Span::styled("TUNINGS", theme::title()),
            Span::styled(" ", theme::label()),
        ]));
    let inner = block.inner(outer);
    frame.render_widget(block, outer);

    let constraints = vec![Constraint::Length(1); ui_state.preset_picker_items.len()];
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (idx, ((_, label), area)) in ui_state
        .preset_picker_items
        .iter()
        .zip(rows.iter())
        .enumerate()
    {
        let style = if idx == ui_state.preset_picker_selected {
            theme::selected_item()
        } else {
            theme::telemetry()
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(format!(" {}", label), style))),
            *area,
        );
    }
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width.saturating_sub(2)).max(10);
    let height = height.min(area.height.saturating_sub(2)).max(5);

    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}
