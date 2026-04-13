use ratatui::style::{Color, Modifier, Style};

pub fn background() -> Style {
    Style::default().bg(Color::Black)
}

pub fn chrome() -> Style {
    Style::default().fg(Color::Rgb(40, 48, 56)).bg(Color::Black)
}

pub fn cyan() -> Style {
    Style::default()
        .fg(Color::Rgb(0, 229, 255))
        .bg(Color::Black)
}

pub fn magenta() -> Style {
    Style::default()
        .fg(Color::Rgb(255, 42, 109))
        .bg(Color::Black)
}

pub fn amber() -> Style {
    Style::default()
        .fg(Color::Rgb(255, 191, 0))
        .bg(Color::Black)
        .add_modifier(Modifier::BOLD)
}

pub fn subtitle() -> Style {
    cyan().add_modifier(Modifier::BOLD)
}

pub fn muted() -> Style {
    Style::default()
        .fg(Color::Rgb(108, 116, 124))
        .bg(Color::Black)
}

pub fn ok() -> Style {
    Style::default()
        .fg(Color::Rgb(0, 229, 255))
        .bg(Color::Black)
        .add_modifier(Modifier::BOLD)
}

pub fn pending() -> Style {
    Style::default()
        .fg(Color::Rgb(255, 191, 0))
        .bg(Color::Black)
}

pub fn accent(is_even_tick: bool) -> Style {
    if is_even_tick {
        magenta().add_modifier(Modifier::BOLD)
    } else {
        cyan().add_modifier(Modifier::BOLD)
    }
}

pub fn handoff(is_even_tick: bool) -> Style {
    if is_even_tick {
        Style::default()
            .fg(Color::Rgb(255, 191, 0))
            .bg(Color::Rgb(255, 42, 109))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(0, 229, 255))
            .add_modifier(Modifier::BOLD)
    }
}
