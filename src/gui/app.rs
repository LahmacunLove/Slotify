use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::gui::modes::{Mode, DjMode, GuestMode, AdminMode};

pub struct DjSystemApp {
    rt: Arc<Runtime>,
    current_mode: Mode,
    dj_mode: DjMode,
    guest_mode: GuestMode,
    admin_mode: AdminMode,
    api_base_url: String,
}

impl DjSystemApp {
    pub fn new(rt: Arc<Runtime>) -> Self {
        let api_base_url = "http://localhost:3000/api".to_string();
        
        Self {
            rt: rt.clone(),
            current_mode: Mode::DJ,
            dj_mode: DjMode::new(rt.clone(), api_base_url.clone()),
            guest_mode: GuestMode::new(rt.clone(), api_base_url.clone()),
            admin_mode: AdminMode::new(rt.clone(), api_base_url.clone()),
            api_base_url,
        }
    }

    fn render_mode_selector(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().button_padding = egui::vec2(20.0, 10.0);
            
            if ui.selectable_label(
                matches!(self.current_mode, Mode::DJ),
                "🎧 DJ Mode"
            ).clicked() {
                self.current_mode = Mode::DJ;
            }

            if ui.selectable_label(
                matches!(self.current_mode, Mode::Guest),
                "👥 Guest Mode"
            ).clicked() {
                self.current_mode = Mode::Guest;
            }

            if ui.selectable_label(
                matches!(self.current_mode, Mode::Admin),
                "⚙️ Admin Mode"
            ).clicked() {
                self.current_mode = Mode::Admin;
            }
        });
    }

    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("🔗 API Status:");
            ui.colored_label(egui::Color32::GREEN, "Connected");
            
            ui.separator();
            
            ui.label("🕐 Current Time:");
            ui.label(chrono::Utc::now().format("%H:%M:%S").to_string());
        });
    }
}

impl eframe::App for DjSystemApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Title
            ui.vertical_centered(|ui| {
                ui.heading("🎵 DJ Session Recorder & Lottery System");
                ui.add_space(10.0);
            });

            // Mode selector
            self.render_mode_selector(ui);
            ui.add_space(20.0);

            // Main content area
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.current_mode {
                    Mode::DJ => self.dj_mode.render(ui),
                    Mode::Guest => self.guest_mode.render(ui),
                    Mode::Admin => self.admin_mode.render(ui),
                }
            });

            // Status bar at the bottom
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                self.render_status_bar(ui);
            });
        });

        // Request repaint for real-time updates
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}