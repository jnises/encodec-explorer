use std::{sync::Arc, time::Duration};

use egui::{emath, epaint, pos2, vec2, Color32, Pos2, Rect, Stroke};
use log::{info, warn};

use crate::{audio, code_ui, synth, worker};

pub struct EncodecExplorer {
    codes: Vec<u32>,
    worker: Option<worker::Worker>,
    audio: Option<audio::AudioManager>,
    synth: Option<Arc<synth::SamplePlayer>>,
}

impl Default for EncodecExplorer {
    fn default() -> Self {
        Self {
            codes: Vec::new(),
            worker: None,
            audio: None,
            synth: None,
        }
    }
}

impl EncodecExplorer {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.style_mut(|s| {
            s.spacing.slider_width = 300.0;
        });
        let mut s = Self::default();
        s.worker = Some(worker::Worker::new());
        s.synth = Some(Arc::new(synth::SamplePlayer::new()));
        s.audio = Some(audio::AudioManager::new(
            s.synth.as_ref().unwrap().clone(),
            |e| warn!("synth error: {e}"),
        ));
        info!("audio device: {:?}", s.audio.as_ref().unwrap().get_name());
        s
    }
}

impl eframe::App for EncodecExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(new_samples) = self.worker.as_mut().unwrap().update() {
            self.synth
                .as_ref()
                .unwrap()
                .update_samples(new_samples.to_vec());
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            draw_buffer(ui, self.worker.as_ref().unwrap().samples());
            let mut new_codes = self.codes.clone();
            code_ui::draw(ui, &mut new_codes);
            if new_codes != self.codes {
                self.codes = new_codes;
                self.worker
                    .as_mut()
                    .unwrap()
                    .set_codes(self.codes.clone())
                    .unwrap();
            }
            //ui.add(egui::Slider::new(&mut self.code, 0..=1023).text("code").vertical());
        });
        // TODO: only reppaint if something has happened
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
