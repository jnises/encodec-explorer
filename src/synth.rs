use std::{borrow::BorrowMut, sync::Mutex};

use log::info;

use crate::audio;

struct State {
    current_sample_rate: u32,
    sample_pos: u64,
    samples: Vec<f32>,
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
        let incoming = self.incoming.lock().unwrap().take();
        let mut state = self.state.lock().unwrap();
        let sref = state.get_or_insert_with(|| State {
            current_sample_rate: sample_rate,
            sample_pos: 0,
            samples: Vec::new(),
        });
        sref.current_sample_rate = sample_rate;
        // TODO: do the resampling on the background thread instead
        for s in out_samples.chunks_exact_mut(channels) {
            // TODO: values from the samples instead
            let value = f64::sin(sref.sample_pos as f64 / sample_rate as f64 * 440f64) as f32;
            for t in s {
                *t = value;
            }
            sref.sample_pos += 1;
        }
    }
}