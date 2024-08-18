use std::sync::Mutex;

use rubato::Resampler;

use crate::audio;

struct State {
    current_sample_rate: u32,
    play_pos: usize,
    raw_samples: Option<Vec<f32>>,
    resampled_samples: Option<Vec<f32>>,
    resampler: Option<rubato::FftFixedIn<f32>>,
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
            current_sample_rate: sample_rate,
            play_pos: 0,
            raw_samples: None,
            resampled_samples: None,
            resampler: None,
        });
        if let Some(incoming) = self.incoming.lock().unwrap().take() {
            sref.raw_samples = Some(incoming);
            sref.resampled_samples = None;
        }
        if sref.current_sample_rate != sample_rate {
            sref.resampled_samples = None;
            sref.resampler = None;
            sref.current_sample_rate = sample_rate;
        }
        if sref.resampled_samples.is_none() {
            sref.play_pos = 0;
            const ENCODEC_SAMPLE_RATE: usize = 24000;
            const ENCODEC_FRAGMENT_SIZE: usize = 320;
            if let Some(raw) = &sref.raw_samples {
                let resampler = sref.resampler.get_or_insert_with(|| {
                    rubato::FftFixedIn::new(
                        ENCODEC_SAMPLE_RATE,
                        sref.current_sample_rate as usize,
                        ENCODEC_FRAGMENT_SIZE,
                        1,
                        1,
                    )
                    .unwrap()
                });
                resampler.reset();
                sref.resampled_samples = Some(
                    resampler
                        .process(&[raw], None)
                        .unwrap()
                        .into_iter()
                        .next()
                        .unwrap(),
                );
            }
        }
        // TODO: do the resampling on the background thread instead
        for s in out_samples.chunks_exact_mut(channels) {
            let value = if let Some(data) = &sref.resampled_samples {
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
