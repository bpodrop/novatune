use crate::tui::state::SignalPhase;
use ratatui::style::{Color, Modifier, Style};

pub fn background() -> Style {
    Style::default().bg(Color::Black)
}

pub fn chrome() -> Style {
    Style::default().fg(Color::Rgb(56, 62, 70)).bg(Color::Black)
}

pub fn title() -> Style {
    Style::default()
        .fg(Color::Rgb(255, 211, 0))
        .bg(Color::Black)
        .add_modifier(Modifier::BOLD)
}

pub fn label() -> Style {
    Style::default()
        .fg(Color::Rgb(140, 152, 170))
        .bg(Color::Black)
}

pub fn muted() -> Style {
    Style::default()
        .fg(Color::Rgb(92, 98, 106))
        .bg(Color::Black)
}

pub fn telemetry() -> Style {
    Style::default()
        .fg(Color::Rgb(0, 229, 255))
        .bg(Color::Black)
}

pub fn overlay() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Rgb(255, 211, 0))
        .add_modifier(Modifier::BOLD)
}

pub fn selected_item() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Rgb(0, 229, 255))
        .add_modifier(Modifier::BOLD)
}

pub fn note(phase: SignalPhase) -> Style {
    let color = match phase {
        SignalPhase::NoSignal => Color::Rgb(92, 98, 106),
        _ => Color::Rgb(255, 211, 0),
    };

    Style::default()
        .fg(color)
        .bg(Color::Black)
        .add_modifier(Modifier::BOLD)
}

pub fn emphasis(phase: SignalPhase) -> Style {
    phase_style(phase).add_modifier(Modifier::BOLD)
}

pub fn phase_style(phase: SignalPhase) -> Style {
    let color = match phase {
        SignalPhase::NoSignal => Color::Rgb(92, 98, 106),
        SignalPhase::Searching | SignalPhase::Unstable => Color::Rgb(0, 229, 255),
        SignalPhase::TooLow => Color::Rgb(255, 211, 0),
        SignalPhase::TooHigh => Color::Rgb(255, 42, 109),
        SignalPhase::InTune => Color::Green,
    };

    Style::default().fg(color).bg(Color::Black)
}

pub fn badge(phase: SignalPhase, animation_tick: u64) -> Style {
    let mut style = phase_style(phase).add_modifier(Modifier::BOLD);
    if should_blink(phase, animation_tick) {
        style = style.add_modifier(Modifier::RAPID_BLINK);
    }
    style
}

pub fn should_blink(phase: SignalPhase, animation_tick: u64) -> bool {
    match phase {
        SignalPhase::InTune => animation_tick % 10 < 2,
        SignalPhase::TooLow | SignalPhase::TooHigh => animation_tick % 8 < 2,
        SignalPhase::Searching | SignalPhase::Unstable => animation_tick % 12 < 1,
        SignalPhase::NoSignal => false,
    }
}
