mod tui;
mod ui;

use tui::{run_demo_tui, run_live_tui};
use tuner_core::{Note, PresetId, default_preset, preset_by_id};
use tuner_engine::SessionMode;
use ui::{SplashOutcome, run_splashscreen};

const DEFAULT_A4_HZ: f32 = 440.0;

fn main() {
    match run_splashscreen() {
        Ok(SplashOutcome::Finished) => {}
        Ok(SplashOutcome::Aborted) => return,
        Err(error) => {
            eprintln!("Splash screen error: {error}");
            std::process::exit(1);
        }
    }

    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse_cli(&args) {
        Ok(Command::Strings { preset, a4_hz }) => print_tuning(preset, a4_hz),
        Ok(Command::Analyze {
            mode,
            preset,
            a4_hz,
        }) => {
            let session_mode = match mode {
                AnalyzeMode::Chromatic => SessionMode::chromatic(),
                AnalyzeMode::Preset => SessionMode::preset(preset.unwrap_or(default_preset().id)),
            };
            run_with_error_handling(run_live_tui(session_mode, a4_hz));
        }
        Ok(Command::Tune { note }) => {
            run_with_error_handling(run_live_tui(SessionMode::target(note), DEFAULT_A4_HZ))
        }
        Ok(Command::Demo) => run_with_error_handling(run_demo_tui()),
        Ok(Command::Help) => print_help(),
        Err(error) => {
            eprintln!("{error}");
            print_help();
            std::process::exit(2);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnalyzeMode {
    Chromatic,
    Preset,
}

#[derive(Debug, Clone, PartialEq)]
enum Command {
    Strings {
        preset: PresetId,
        a4_hz: f32,
    },
    Analyze {
        mode: AnalyzeMode,
        preset: Option<PresetId>,
        a4_hz: f32,
    },
    Tune {
        note: Note,
    },
    Demo,
    Help,
}

fn parse_cli(args: &[String]) -> Result<Command, String> {
    let Some(command) = args.first().map(|value| value.as_str()) else {
        return Ok(Command::Strings {
            preset: default_preset().id,
            a4_hz: DEFAULT_A4_HZ,
        });
    };

    match command {
        "strings" => parse_strings(&args[1..]),
        "analyze" => parse_analyze(&args[1..]),
        "tune" => parse_tune(&args[1..]),
        "demo" => Ok(Command::Demo),
        "help" | "--help" | "-h" => Ok(Command::Help),
        other => Err(format!("Unknown command: {other}")),
    }
}

fn parse_strings(args: &[String]) -> Result<Command, String> {
    let mut preset = default_preset().id;
    let mut a4_hz = DEFAULT_A4_HZ;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--preset" => {
                let value = args.get(index + 1).ok_or("--preset requires a value")?;
                preset =
                    PresetId::parse(value).ok_or_else(|| format!("Unknown preset: {value}"))?;
                index += 2;
            }
            "--a4" => {
                let value = args.get(index + 1).ok_or("--a4 requires a value")?;
                a4_hz = parse_a4_hz(value)?;
                index += 2;
            }
            other => return Err(format!("Unknown option for strings: {other}")),
        }
    }

    Ok(Command::Strings { preset, a4_hz })
}

fn parse_analyze(args: &[String]) -> Result<Command, String> {
    let mut mode = AnalyzeMode::Chromatic;
    let mut preset = None;
    let mut a4_hz = DEFAULT_A4_HZ;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--mode" => {
                let value = args.get(index + 1).ok_or("--mode requires a value")?;
                mode = match value.as_str() {
                    "chromatic" => AnalyzeMode::Chromatic,
                    "preset" => AnalyzeMode::Preset,
                    _ => return Err(format!("Unknown mode: {value}")),
                };
                index += 2;
            }
            "--preset" => {
                let value = args.get(index + 1).ok_or("--preset requires a value")?;
                preset =
                    Some(PresetId::parse(value).ok_or_else(|| format!("Unknown preset: {value}"))?);
                index += 2;
            }
            "--a4" => {
                let value = args.get(index + 1).ok_or("--a4 requires a value")?;
                a4_hz = parse_a4_hz(value)?;
                index += 2;
            }
            other => return Err(format!("Unknown option for analyze: {other}")),
        }
    }

    Ok(Command::Analyze {
        mode,
        preset,
        a4_hz,
    })
}

fn parse_tune(args: &[String]) -> Result<Command, String> {
    let note = args
        .first()
        .ok_or("Missing target note. Usage: tuner-cli tune <NOTE>")?;
    let note = Note::from_label(note)
        .ok_or_else(|| format!("Invalid target note: {note}. Example: E2, A2, D3, G3, B3, E4"))?;

    Ok(Command::Tune { note })
}

fn parse_a4_hz(raw: &str) -> Result<f32, String> {
    let value = raw
        .parse::<f32>()
        .map_err(|_| format!("Invalid --a4 value: {raw}"))?;
    if !value.is_finite() || value <= 0.0 {
        return Err(format!("Invalid --a4 value: {raw}"));
    }
    Ok(value)
}

fn print_tuning(preset_id: PresetId, a4_hz: f32) {
    let preset = preset_by_id(preset_id);
    let ratio = a4_hz / DEFAULT_A4_HZ;
    println!("{} (A4 = {:.2} Hz):", preset.display_name, a4_hz);
    for string in preset.strings {
        println!("{} -> {:.2} Hz", string.label, string.frequency_hz * ratio);
    }
}

fn print_help() {
    println!("tuner-cli");
    println!();
    println!("Commands:");
    println!("  strings [--preset <id>] [--a4 <hz>]");
    println!("  analyze [--mode chromatic|preset] [--preset <id>] [--a4 <hz>]");
    println!("  tune <NOTE>");
    println!("  demo");
    println!();
    println!("Preset ids:");
    println!("  e-standard  drop-d  eb-standard  d-standard");
    println!("  drop-c  open-g  open-d  dadgad");
    println!();
    println!("Examples:");
    println!("  tuner-cli strings --preset drop-d --a4 432");
    println!("  tuner-cli analyze --mode chromatic");
    println!("  tuner-cli analyze --mode preset --preset e-standard --a4 442");
    println!("  tuner-cli tune E2");
    println!();
    println!("Live TUI hotkeys:");
    println!("  q / Esc    quit");
    println!("  m          toggle Chromatic / Preset in analyze");
    println!("  p          open preset picker");
    println!("  up/down    navigate preset picker");
    println!("  enter      apply selected preset");
    println!("  h / ?      toggle in-app help");
}

fn run_with_error_handling(result: std::io::Result<()>) {
    if let Err(error) = result {
        eprintln!("Terminal UI error: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, parse_cli};

    #[test]
    fn parses_analyze_preset_command() {
        let args = vec![
            "analyze".to_string(),
            "--mode".to_string(),
            "preset".to_string(),
            "--preset".to_string(),
            "drop-d".to_string(),
        ];

        let command = parse_cli(&args).unwrap();
        assert!(matches!(command, Command::Analyze { .. }));
    }

    #[test]
    fn parses_a4_argument_for_analyze() {
        let args = vec!["analyze".to_string(), "--a4".to_string(), "432".to_string()];

        let command = parse_cli(&args).unwrap();
        match command {
            Command::Analyze { a4_hz, .. } => assert!((a4_hz - 432.0).abs() < 0.001),
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
