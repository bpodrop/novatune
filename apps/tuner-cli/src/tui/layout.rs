use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenClass {
    Compact,
    Standard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootLayout {
    pub header: Rect,
    pub hero: Rect,
    pub footer: Rect,
    pub screen_class: ScreenClass,
}

pub fn build(area: Rect) -> RootLayout {
    let screen_class = if area.width < 76 || area.height < 22 {
        ScreenClass::Compact
    } else {
        ScreenClass::Standard
    };

    let chrome = area.inner(Margin {
        vertical: if area.height > 20 { 1 } else { 0 },
        horizontal: if area.width > 60 { 2 } else { 0 },
    });

    let sections = match screen_class {
        ScreenClass::Standard => Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Min(11),
                Constraint::Length(5),
            ])
            .split(chrome),
        ScreenClass::Compact => Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(4),
            ])
            .split(chrome),
    };

    RootLayout {
        header: sections[0],
        hero: sections[1],
        footer: sections[2],
        screen_class,
    }
}
