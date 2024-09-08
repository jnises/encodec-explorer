use std::sync::Mutex;

use rubato::Resampler;

use crate::audio;

struct State {
    current_sample_rate: u32,
    play_pos: usize,
    raw_samples: Option<Vec<f32>>,
    resampled_samples: Option<Vec<f32>>,
}

pub struct SamplePlayer {
    incoming: Mutex<Option<Vec<f32>>>,
    // TODO: don't share the player between threads, so we can avoid this mutex
    state: Mutex<Option<State>>,
}

impl SamplePlayer {
    pub fn new() -> Self {
        Self {
            incoming: Mutex::new(None),
            state: Mutex::new(None),
        }
    }

    pub fn update_samples(&self, samples: Vec<f32>) {
        *self.incoming.lock().unwrap() = Some(samples);
    }
}

impl audio::Synth for SamplePlayer {
    fn play(&self, sample_rate: u32, channels: usize, out_samples: &mut [f32]) {
        let mut state = self.state.lock().unwrap();
        let sref = state.get_or_insert_with(|| State {
            current_sample_rate: 0,
            play_pos: 0,
            raw_samples: None,
            resampled_samples: None,
        });
        if let Some(incoming) = self.incoming.lock().unwrap().take() {
            sref.raw_samples = Some(incoming);
            sref.resampled_samples = None;
        }
        if sref.current_sample_rate != sample_rate {
            log::info!("sample rate changed to: {sample_rate}");
            sref.resampled_samples = None;
            sref.current_sample_rate = sample_rate;
        }
        if sref.resampled_samples.is_none() {
            sref.play_pos = 0;
            const ENCODEC_SAMPLE_RATE: usize = 24000;
            if let Some(raw) = &sref.raw_samples {
                let mut resampler = rubato::FftFixedInOut::new(
                    ENCODEC_SAMPLE_RATE,
                    sref.current_sample_rate as usize,
                    raw.len(),
                    1,
                )
                .unwrap();
                // TODO: handle looping better?
                let mut resampled = sref.resampled_samples.take().unwrap_or_default();
                loop {
                    match resampler.process_into_buffer(&[raw], &mut [&mut resampled], None) {
                        Ok((in_samples, out_samples)) => {
                            debug_assert_eq!(in_samples, raw.len());
                            resampled.truncate(out_samples);
                            break;
                        }
                        Err(rubato::ResampleError::InsufficientOutputBufferSize { channel, expected, .. }) => {
                            debug_assert_eq!(channel, 0);
                            resampled.resize(expected, 0f32);
                        },
                        Err(e) => panic!("{e}"),
                    }
                }
                sref.resampled_samples = Some(resampled);
                debug_assert!(
                    sref.resampled_samples.as_ref().unwrap().len().abs_diff(
                        raw.len() * sref.current_sample_rate as usize / ENCODEC_SAMPLE_RATE
                    ) < 1
                );
            }
        }
        // TODO: do the resampling on the background thread instead
        for s in out_samples.chunks_exact_mut(channels) {
            let value = if let Some(data) = &sref.resampled_samples {
                //let value = if let Some(data) = &sref.raw_samples {
                sref.play_pos = (sref.play_pos + 1) % data.len();
                data[sref.play_pos]
            } else {
                0.0
            };
            for t in s {
                *t = value;
            }
        }
    }
}
