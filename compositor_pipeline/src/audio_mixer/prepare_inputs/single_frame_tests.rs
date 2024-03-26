use std::time::Duration;

use crate::audio_mixer::InputSamples;

use super::frame_input_samples;

#[test]
fn test_prepare_inputs() {
    // 6 samples at sample rate 48000
    let batch_duration = Duration::from_micros(125);
    let start = Duration::from_millis(20);
    let end = start + batch_duration;
    let sample_rate = 48000;
    let sample_duration = Duration::from_secs_f64(1.0 / sample_rate as f64);
    let small_error = Duration::from_secs_f64(sample_duration.as_secs_f64() * 0.001);
    let half_sample = Duration::from_secs_f64(sample_duration.as_secs_f64() * 0.5);

    assert_eq!(
        frame_input_samples(start, end, vec![], sample_rate),
        vec![(0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0)]
    );

    let first_batch_start = start - small_error;
    let second_batch_start = first_batch_start + (4 * sample_duration);
    assert_eq!(
        frame_input_samples(
            start,
            end,
            vec![
                InputSamples {
                    samples: vec![(1, 1), (2, 2), (3, 3), (4, 4)].into(),
                    start_pts: first_batch_start,
                    end_pts: first_batch_start + (4 * sample_duration)
                },
                InputSamples {
                    samples: vec![(5, 5), (6, 6), (7, 7), (8, 8)].into(),
                    start_pts: second_batch_start,
                    end_pts: second_batch_start + (4 * sample_duration)
                },
            ],
            sample_rate
        ),
        vec![(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)]
    );

    let first_batch_start = start - half_sample;
    let second_batch_start = first_batch_start + (4 * sample_duration);
    assert_eq!(
        frame_input_samples(
            start,
            end,
            vec![
                InputSamples {
                    samples: vec![(1, 1), (2, 2), (3, 3), (4, 4)].into(),
                    start_pts: first_batch_start,
                    end_pts: first_batch_start + (4 * sample_duration)
                },
                InputSamples {
                    samples: vec![(5, 5), (6, 6), (7, 7), (8, 8)].into(),
                    start_pts: second_batch_start,
                    end_pts: second_batch_start + (4 * sample_duration)
                },
            ],
            sample_rate
        ),
        vec![(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)]
    );

    let first_batch_start = start + small_error;
    let second_batch_start = first_batch_start + (4 * sample_duration);
    assert_eq!(
        frame_input_samples(
            start,
            end,
            vec![
                InputSamples {
                    samples: vec![(1, 1), (2, 2), (3, 3), (4, 4)].into(),
                    start_pts: first_batch_start,
                    end_pts: first_batch_start + (4 * sample_duration)
                },
                InputSamples {
                    samples: vec![(5, 5), (6, 6), (7, 7), (8, 8)].into(),
                    start_pts: second_batch_start,
                    end_pts: second_batch_start + (4 * sample_duration)
                },
            ],
            sample_rate
        ),
        vec![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5)]
    );

    let first_batch_start = start - sample_duration + small_error;
    let second_batch_start = first_batch_start + (4 * sample_duration);
    assert_eq!(
        frame_input_samples(
            start,
            end,
            vec![
                InputSamples {
                    samples: vec![(1, 1), (2, 2), (3, 3), (4, 4)].into(),
                    start_pts: first_batch_start,
                    end_pts: first_batch_start + (4 * sample_duration)
                },
                InputSamples {
                    samples: vec![(5, 5), (6, 6), (7, 7), (8, 8)].into(),
                    start_pts: second_batch_start,
                    end_pts: second_batch_start + (4 * sample_duration)
                },
            ],
            sample_rate
        ),
        vec![(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)]
    );

    let first_batch_start = start - sample_duration - small_error;
    let second_batch_start = first_batch_start + (4 * sample_duration);
    assert_eq!(
        frame_input_samples(
            start,
            end,
            vec![
                InputSamples {
                    samples: vec![(1, 1), (2, 2), (3, 3), (4, 4)].into(),
                    start_pts: first_batch_start,
                    end_pts: first_batch_start + (4 * sample_duration)
                },
                InputSamples {
                    samples: vec![(5, 5), (6, 6), (7, 7), (8, 8)].into(),
                    start_pts: second_batch_start,
                    end_pts: second_batch_start + (4 * sample_duration)
                },
            ],
            sample_rate
        ),
        vec![(2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7)]
    );

    //slightly overlapping batches
    let first_batch_start = start - sample_duration + small_error;
    let second_batch_start = first_batch_start + (4 * sample_duration) - small_error;
    assert_eq!(
        frame_input_samples(
            start,
            end,
            vec![
                InputSamples {
                    samples: vec![(1, 1), (2, 2), (3, 3), (4, 4)].into(),
                    start_pts: first_batch_start,
                    end_pts: first_batch_start + (4 * sample_duration)
                },
                InputSamples {
                    samples: vec![(5, 5), (6, 6), (7, 7), (8, 8)].into(),
                    start_pts: second_batch_start,
                    end_pts: second_batch_start + (4 * sample_duration)
                },
            ],
            sample_rate
        ),
        vec![(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)]
    );

    // batches with small gap (small error)
    let first_batch_start = start - sample_duration + small_error;
    let second_batch_start = first_batch_start + (4 * sample_duration) + small_error;
    assert_eq!(
        frame_input_samples(
            start,
            end,
            vec![
                InputSamples {
                    samples: vec![(1, 1), (2, 2), (3, 3), (4, 4)].into(),
                    start_pts: first_batch_start,
                    end_pts: first_batch_start + (4 * sample_duration)
                },
                InputSamples {
                    samples: vec![(5, 5), (6, 6), (7, 7), (8, 8)].into(),
                    start_pts: second_batch_start,
                    end_pts: second_batch_start + (4 * sample_duration)
                },
            ],
            sample_rate
        ),
        vec![(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)]
    );

    //slightly overlapping batches (more than half sample)
    let first_batch_start = start - sample_duration + small_error;
    let second_batch_start = first_batch_start + (4 * sample_duration) - small_error - half_sample;
    assert_eq!(
        frame_input_samples(
            start,
            end,
            vec![
                InputSamples {
                    samples: vec![(1, 1), (2, 2), (3, 3), (4, 4)].into(),
                    start_pts: first_batch_start,
                    end_pts: first_batch_start + (4 * sample_duration)
                },
                InputSamples {
                    samples: vec![(5, 5), (6, 6), (7, 7), (8, 8)].into(),
                    start_pts: second_batch_start,
                    end_pts: second_batch_start + (4 * sample_duration)
                },
            ],
            sample_rate
        ),
        vec![(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)]
    );

    // batches with small gap (more than half sample)
    let first_batch_start = start - sample_duration + small_error;
    let second_batch_start = first_batch_start + (4 * sample_duration) + small_error + half_sample;
    assert_eq!(
        frame_input_samples(
            start,
            end,
            vec![
                InputSamples {
                    samples: vec![(1, 1), (2, 2), (3, 3), (4, 4)].into(),
                    start_pts: first_batch_start,
                    end_pts: first_batch_start + (4 * sample_duration)
                },
                InputSamples {
                    samples: vec![(5, 5), (6, 6), (7, 7), (8, 8)].into(),
                    start_pts: second_batch_start,
                    end_pts: second_batch_start + (4 * sample_duration)
                },
            ],
            sample_rate
        ),
        vec![(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)]
    );

    //slightly overlapping batches (more than a sample)
    let first_batch_start = start - sample_duration + small_error;
    let second_batch_start =
        first_batch_start + (4 * sample_duration) - small_error - sample_duration;
    assert_eq!(
        frame_input_samples(
            start,
            end,
            vec![
                InputSamples {
                    samples: vec![(1, 1), (2, 2), (3, 3), (4, 4)].into(),
                    start_pts: first_batch_start,
                    end_pts: first_batch_start + (4 * sample_duration)
                },
                InputSamples {
                    samples: vec![(5, 5), (6, 6), (7, 7), (8, 8)].into(),
                    start_pts: second_batch_start,
                    end_pts: second_batch_start + (4 * sample_duration)
                },
            ],
            sample_rate
        ),
        vec![(1, 1), (2, 2), (3, 3), (4, 4), (6, 6), (7, 7)]
    );

    // batches with small gap (more than half sample)
    let first_batch_start = start - sample_duration + small_error;
    let second_batch_start =
        first_batch_start + (4 * sample_duration) + small_error + sample_duration;
    assert_eq!(
        frame_input_samples(
            start,
            end,
            vec![
                InputSamples {
                    samples: vec![(1, 1), (2, 2), (3, 3), (4, 4)].into(),
                    start_pts: first_batch_start,
                    end_pts: first_batch_start + (4 * sample_duration)
                },
                InputSamples {
                    samples: vec![(5, 5), (6, 6), (7, 7), (8, 8)].into(),
                    start_pts: second_batch_start,
                    end_pts: second_batch_start + (4 * sample_duration)
                },
            ],
            sample_rate
        ),
        vec![(1, 1), (2, 2), (3, 3), (4, 4), (0, 0), (5, 5)]
    );
}
