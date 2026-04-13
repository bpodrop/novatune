use std::f32::consts::PI;

use tuner_dsp_web::{
    new_detector, next_output, push_samples, reset, set_calibration_hz, set_min_clarity,
    set_min_rms, set_mode, set_preset, shutdown,
};

fn sine_wave(frequency_hz: f32, sample_rate: u32, frame_size: usize, amplitude: f32) -> Vec<f32> {
    (0..frame_size)
        .map(|index| {
            let t = index as f32 / sample_rate as f32;
            amplitude * (2.0 * PI * frequency_hz * t).sin()
        })
        .collect()
}

#[test]
fn detector_lifecycle_and_outputs_follow_contract() {
    let detector_id = new_detector(44_100, 4096, 1024);

    let frame = sine_wave(110.0, 44_100, 4096, 0.8);
    let produced = push_samples(detector_id, &frame);

    assert_eq!(produced, 1);

    let output = next_output(detector_id).expect("expected a pitch output");
    assert!(output.frequency_hz > 80.0 && output.frequency_hz < 140.0);
    assert!(output.confidence > 0.0);

    assert!(next_output(detector_id).is_none());

    assert!(reset(detector_id));
    assert!(next_output(detector_id).is_none());

    assert!(shutdown(detector_id));
    assert!(!shutdown(detector_id));
}

#[test]
fn unknown_detector_operations_are_safe() {
    let unknown_detector_id = u32::MAX;

    assert_eq!(push_samples(unknown_detector_id, &[0.0, 1.0, 0.0]), 0);
    assert!(next_output(unknown_detector_id).is_none());
    assert!(!reset(unknown_detector_id));
    assert!(!shutdown(unknown_detector_id));
}

#[test]
fn invalid_detector_configs_are_rejected_and_safe() {
    let invalid_frame_id = new_detector(44_100, 0, 1024);
    let invalid_hop_id = new_detector(44_100, 2048, 0);
    let invalid_rate_id = new_detector(0, 2048, 256);
    let invalid_shape_id = new_detector(44_100, 512, 1024);

    assert_eq!(invalid_frame_id, 0);
    assert_eq!(invalid_hop_id, 0);
    assert_eq!(invalid_rate_id, 0);
    assert_eq!(invalid_shape_id, 0);

    assert_eq!(push_samples(invalid_frame_id, &[0.1, -0.1]), 0);
    assert!(next_output(invalid_frame_id).is_none());
    assert!(!reset(invalid_frame_id));
    assert!(!shutdown(invalid_frame_id));
}

#[test]
fn tuning_controls_update_output_mapping() {
    let detector_id = new_detector(44_100, 4096, 1024);
    assert_ne!(detector_id, 0);

    assert!(set_preset(detector_id, "drop-d"));
    assert!(set_calibration_hz(detector_id, 432.0));
    assert!(set_mode(detector_id, "preset"));

    let frame = sine_wave(82.41, 44_100, 4096, 0.8);
    let produced = push_samples(detector_id, &frame);
    assert_eq!(produced, 1);

    let output = next_output(detector_id).expect("expected tuned output");
    assert_eq!(output.note_name, "D2");
    assert!(output.cents_off > 150.0);

    assert!(shutdown(detector_id));
}

#[test]
fn tuning_controls_validate_inputs() {
    let detector_id = new_detector(44_100, 4096, 1024);
    assert_ne!(detector_id, 0);

    assert!(!set_preset(detector_id, "not-a-preset"));
    assert!(!set_calibration_hz(detector_id, 0.0));
    assert!(!set_calibration_hz(detector_id, -1.0));
    assert!(!set_min_rms(detector_id, -0.1));
    assert!(!set_min_clarity(detector_id, -0.1));
    assert!(!set_min_clarity(detector_id, 1.1));
    assert!(!set_mode(detector_id, "invalid"));

    assert!(!set_preset(u32::MAX, "drop-d"));
    assert!(!set_calibration_hz(u32::MAX, 440.0));
    assert!(!set_min_rms(u32::MAX, 0.01));
    assert!(!set_min_clarity(u32::MAX, 0.6));
    assert!(!set_mode(u32::MAX, "preset"));

    assert!(shutdown(detector_id));
}
