use std::{sync::Arc, time::Duration};

use candle_core::Tensor;
use egui::{emath, epaint, pos2, vec2, Color32, Pos2, Rect, Spinner, Stroke};
use log::{info, warn};
use poll_promise::Promise;

use crate::{
    audio, code_ui,
    compute::{self, Compute},
    synth,
};

#[derive(Default)]
enum ComputeState {
    #[default]
    Uninitialized,
    Loading(Promise<anyhow::Result<Compute>>),
    Loaded(Compute),
}

pub struct EncodecExplorer {
    codes: Vec<u32>,
    compute: ComputeState,
    audio: Option<audio::AudioManager>,
    synth: Option<Arc<synth::SamplePlayer>>,
    samples: Vec<f32>,
}

impl Default for EncodecExplorer {
    fn default() -> Self {
        Self {
            codes: Vec::new(),
            compute: ComputeState::Uninitialized,
            audio: None,
            synth: None,
            samples: vec![0.0; 320],
        }
    }
}

impl EncodecExplorer {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.style_mut(|s| {
            s.spacing.slider_width = 300.0;
        });
        Self {
            synth: Some(Arc::new(synth::SamplePlayer::new())),
            ..Default::default()
        }
    }
}

impl eframe::App for EncodecExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("encodec-explorer");
            match self.audio {
                Some(_) => {
                    self.compute = match std::mem::take(&mut self.compute) {
                        ComputeState::Uninitialized => {
                            ui.label("uninitialized");
                            ComputeState::Loading(Promise::spawn_local(compute::Compute::new()))
                        }
                        ComputeState::Loading(p) => {
                            ui.add(Spinner::new());
                            match p.try_take() {
                                Ok(c) => ComputeState::Loaded(c.unwrap()),
                                Err(p) => ComputeState::Loading(p),
                            }
                        }
                        ComputeState::Loaded(c) => {
                            draw_buffer(ui, &self.samples);
                            let mut new_codes = self.codes.clone();
                            code_ui::draw(ui, &mut new_codes);
                            if new_codes != self.codes {
                                self.codes = new_codes;
                                // TODO: do the computation on a separate worker instead
                                self.samples = c
                                    .decode_codes(
                                        &Tensor::from_vec(
                                            self.codes.clone(),
                                            (self.codes.len(), 1),
                                            c.device(),
                                        )
                                        .unwrap(),
                                    )
                                    .unwrap();
                                self.synth
                                    .as_ref()
                                    .unwrap()
                                    .update_samples(self.samples.clone());
                            }
                            ComputeState::Loaded(c)
                        }
                    };
                }
                None => {
                    // need to wait with audio until a button is clicked
                    if ui.button("▶").clicked() {
                        self.audio = Some(audio::AudioManager::new(
                            self.synth.as_ref().unwrap().clone(),
                            |e| warn!("synth error: {e}"),
                        ));
                        info!(
                            "audio device: {:?}",
                            self.audio.as_ref().unwrap().get_name()
                        );
                    }
                }
            }
        });
        // TODO: only repaint if something has happened
        ctx.request_repaint_after(Duration::from_secs(1));
    }
}

fn draw_buffer(ui: &mut egui::Ui, buffer: &[f32]) {
    let plot_width = ui.available_width().min(300.);
    let (_, rect) = ui.allocate_space(vec2(plot_width, plot_width * 0.5));
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 10f32, Color32::BLACK);
    let to_rect = emath::RectTransform::from_to(
        Rect::from_x_y_ranges(0.0..=(buffer.len() - 1) as f32, -1.0..=1.0),
        rect,
    );
    let line: Vec<Pos2> = buffer
        .iter()
        .copied()
        .enumerate()
        .map(|(x, y)| to_rect * pos2(x as f32, y))
        .collect();
    p.add(epaint::Shape::line(line, Stroke::new(1f32, Color32::GRAY)));
}
