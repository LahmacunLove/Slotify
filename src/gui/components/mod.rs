// Common UI components for the DJ System GUI

use eframe::egui;

pub struct TouchButton;

impl TouchButton {
    pub fn new(text: &str) -> egui::Button {
        egui::Button::new(text)
            .min_size(egui::vec2(120.0, 50.0))
    }

    pub fn large(text: &str) -> egui::Button {
        egui::Button::new(text)
            .min_size(egui::vec2(200.0, 80.0))
    }
}

pub struct QrCodeWidget;

impl QrCodeWidget {
    pub fn show(ui: &mut egui::Ui, size: f32, data: &str) {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(size, size),
            egui::Sense::hover()
        );

        // Draw QR code background
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(5.0),
            egui::Color32::WHITE
        );

        ui.painter().rect_stroke(
            rect,
            egui::Rounding::same(5.0),
            egui::Stroke::new(2.0, egui::Color32::BLACK)
        );

        // Draw simplified QR pattern
        let cell_size = size / 25.0;
        let data_hash = simple_hash(data);
        
        for x in 0..25 {
            for y in 0..25 {
                // Create pseudo-random pattern based on data
                if (x + y + data_hash) % 3 == 0 {
                    let cell_rect = egui::Rect::from_min_size(
                        rect.min + egui::vec2(x as f32 * cell_size, y as f32 * cell_size),
                        egui::vec2(cell_size, cell_size)
                    );
                    ui.painter().rect_filled(
                        cell_rect,
                        egui::Rounding::ZERO,
                        egui::Color32::BLACK
                    );
                }
            }
        }
    }
}

fn simple_hash(data: &str) -> usize {
    data.chars().map(|c| c as usize).sum()
}

pub struct StatusIndicator;

impl StatusIndicator {
    pub fn show(ui: &mut egui::Ui, status: &str, is_good: bool) {
        let color = if is_good {
            egui::Color32::GREEN
        } else {
            egui::Color32::RED
        };

        let icon = if is_good { "✅" } else { "❌" };
        
        ui.horizontal(|ui| {
            ui.label(icon);
            ui.colored_label(color, status);
        });
    }
}