use crate::tui::state::{GAUGE_CLAMP_CENTS, SignalPhase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GaugeRows {
    pub scale: String,
    pub marker: String,
    pub legend: String,
}

pub fn marker_position(cents_off: f32, gauge_width: u16, phase: SignalPhase) -> u16 {
    let center = center_position(gauge_width);
    if gauge_width == 0
        || matches!(
            phase,
            SignalPhase::InTune | SignalPhase::NoSignal | SignalPhase::Searching
        )
    {
        return center;
    }

    let last_index = gauge_width - 1;
    let normalized = (cents_off / GAUGE_CLAMP_CENTS).clamp(-1.0, 1.0);
    let span = center as f32;
    let offset = (normalized * span).round() as i16;
    let centered = center as i16 + offset;

    centered.clamp(0, last_index as i16) as u16
}

pub fn center_position(gauge_width: u16) -> u16 {
    gauge_width.saturating_sub(1) / 2
}

pub fn build_gauge_rows(
    cents_off: f32,
    phase: SignalPhase,
    gauge_width: u16,
    in_tune_threshold_cents: f32,
    compact: bool,
) -> GaugeRows {
    let center = center_position(gauge_width);
    let marker = marker_position(cents_off, gauge_width, phase);

    GaugeRows {
        scale: build_scale_row(center, gauge_width, compact),
        marker: build_marker_row(center, marker, phase, gauge_width),
        legend: build_legend_row(phase, in_tune_threshold_cents, compact),
    }
}

fn build_scale_row(center: u16, gauge_width: u16, compact: bool) -> String {
    let mut cells = vec![' '; gauge_width as usize];
    if gauge_width == 0 {
        return String::new();
    }

    let quarter = gauge_width / 4;
    cells.fill('─');
    cells[quarter as usize] = '┬';
    cells[center as usize] = '◎';
    cells[(gauge_width - quarter - 1) as usize] = '┬';

    let track: String = cells.into_iter().collect();
    if compact {
        track
    } else {
        format!("LOW {track} HIGH")
    }
}

fn build_marker_row(center: u16, marker: u16, phase: SignalPhase, gauge_width: u16) -> String {
    let mut cells = vec![' '; gauge_width as usize];
    if gauge_width == 0 {
        return String::new();
    }

    cells[center as usize] = '│';
    for index in marker.saturating_sub(1)..=(marker + 1).min(gauge_width - 1) {
        cells[index as usize] = marker_char(phase);
    }

    if matches!(phase, SignalPhase::NoSignal) {
        cells[center as usize] = '·';
    }

    cells.into_iter().collect()
}

fn build_legend_row(phase: SignalPhase, in_tune_threshold_cents: f32, compact: bool) -> String {
    let label = match phase {
        SignalPhase::NoSignal => "INPUT OFFLINE",
        SignalPhase::Searching => "SCAN VECTOR",
        SignalPhase::Unstable => "HOLD STRING",
        SignalPhase::TooLow => "MOVE RIGHT",
        SignalPhase::TooHigh => "MOVE LEFT",
        SignalPhase::InTune => "CENTER LOCK",
    };

    if compact {
        label.to_string()
    } else {
        format!("{label}   WINDOW ±{in_tune_threshold_cents:.0}c")
    }
}

fn marker_char(phase: SignalPhase) -> char {
    match phase {
        SignalPhase::NoSignal => '·',
        SignalPhase::Searching => '▒',
        SignalPhase::Unstable => '▓',
        SignalPhase::TooLow | SignalPhase::TooHigh | SignalPhase::InTune => '█',
    }
}

#[cfg(test)]
mod tests {
    use super::{build_gauge_rows, center_position, marker_position};
    use crate::tui::state::SignalPhase;

    #[test]
    fn marker_position_clamps_to_range_edges() {
        assert_eq!(marker_position(-50.0, 25, SignalPhase::TooLow), 0);
        assert_eq!(marker_position(0.0, 25, SignalPhase::InTune), 12);
        assert_eq!(marker_position(50.0, 25, SignalPhase::TooHigh), 24);
        assert_eq!(marker_position(100.0, 25, SignalPhase::TooHigh), 24);
    }

    #[test]
    fn center_position_is_exact_for_odd_width() {
        assert_eq!(center_position(25), 12);
    }

    #[test]
    fn in_tune_keeps_marker_centered() {
        let rows = build_gauge_rows(0.4, SignalPhase::InTune, 25, 3.0, false);
        assert!(rows.marker.contains("███"));
        assert!(rows.scale.contains('◎'));
    }
}
