use egui::DragValue;

pub fn draw(ui: &mut egui::Ui, codes: &mut Vec<u32>) {
    if codes.is_empty() {
        codes.push(0);
    }
    ui.horizontal(|ui| {
        for c in codes.iter_mut() {
            const MAX_CODE: usize = 1023;
            ui.add(DragValue::new(c).range(0..=MAX_CODE));
        }
        const MAX_BOOKS: usize = 255;
        ui.vertical(|ui| {
            if ui
                .add_enabled(codes.len() <= MAX_BOOKS, egui::Button::new("+").small())
                .clicked()
            {
                codes.push(0);
            }
            if ui
                .add_enabled(codes.len() > 1, egui::Button::new("-").small())
                .clicked()
            {
                codes.pop();
            }
        });
    });
}
