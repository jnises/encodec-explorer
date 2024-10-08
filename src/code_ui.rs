use candle_core::{Device, Tensor};
use egui::Slider;
use num::Integer as _;

// TODO: use some existing type for 2d arrays instead?
#[derive(Clone)]
pub struct Codes {
    codes: Vec<u32>,
    width: usize,
}

impl Default for Codes {
    fn default() -> Self {
        Self::new()
    }
}

impl Codes {
    pub fn new() -> Self {
        Self {
            codes: vec![0],
            width: 1,
        }
    }

    pub fn to_tensor(&self, device: &Device) -> anyhow::Result<Tensor> {
        Ok(if self.width == 0 {
            Tensor::zeros((0, 0), candle_core::DType::U32, device)?
        } else {
            debug_assert!(self.codes.len() % self.width == 0);
            let height = self.codes.len() / self.width;
            Tensor::from_vec(self.codes.clone(), (height, self.width), device)?
        })
    }

    fn height(&self) -> usize {
        debug_assert!(self.width > 0);
        debug_assert!(!self.codes.is_empty());
        if self.width == 0 {
            0
        } else {
            self.codes.len() / self.width
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<u32> {
        if x < self.width && y < self.height() {
            Some(self.codes[y * self.width + x])
        } else {
            None
        }
    }

    fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut u32> {
        if x < self.width && y < self.height() {
            Some(&mut self.codes[y * self.width + x])
        } else {
            None
        }
    }

    fn reshape(&mut self, width: usize, height: usize) {
        assert!(width >= 1);
        assert!(height >= 1);
        self.codes = (0..height * width)
            .map(|i| {
                let (y, x) = i.div_rem(&width);
                self.get(x, y).unwrap_or(0)
            })
            .collect();
        self.width = width;
    }
}

impl PartialEq for Codes {
    fn eq(&self, other: &Self) -> bool {
        self.codes == other.codes && self.width == other.width
    }
}

pub fn draw(ui: &mut egui::Ui, codes: &mut Codes) {
    const MAX_FRAGMENTS: usize = 4;
    const MAX_LAYERS: usize = 32;
    ui.group(|ui| {
        egui::ScrollArea::vertical()
            .max_height(500.0)
            .show(ui, |ui| {
                // TODO: use a table?
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(codes.width > 1, egui::Button::new("⬅").small())
                            .clicked()
                        {
                            codes.reshape(codes.width - 1, codes.height());
                        }
                        if ui
                            .add_enabled(
                                codes.width < MAX_FRAGMENTS,
                                egui::Button::new("➡").small(),
                            )
                            .clicked()
                        {
                            codes.reshape(codes.width + 1, codes.height());
                        }
                        for x in 0..codes.width {
                            ui.separator();
                            ui.vertical(|ui| {
                                for y in 0..codes.height() {
                                    const MAX_CODE: u32 = 1023;
                                    let value = codes.get_mut(x, y).unwrap();
                                    ui.horizontal(|ui| {
                                        ui.add(Slider::new(value, 0..=MAX_CODE));
                                        if ui
                                            .add_enabled(*value > 0, egui::Button::new("-").small())
                                            .clicked()
                                        {
                                            *value -= 1;
                                        }
                                        if ui
                                            .add_enabled(
                                                *value < MAX_CODE,
                                                egui::Button::new("+").small(),
                                            )
                                            .clicked()
                                        {
                                            *value += 1;
                                        };
                                    });
                                }
                            });
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(codes.height() > 1, egui::Button::new("⬆").small())
                            .clicked()
                        {
                            codes.reshape(codes.width, codes.height() - 1);
                        }
                        if ui
                            .add_enabled(
                                codes.height() < MAX_LAYERS,
                                egui::Button::new("⬇").small(),
                            )
                            .clicked()
                        {
                            codes.reshape(codes.width, codes.height() + 1);
                        }
                    });
                });
            });
    });
}
