use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;
use serde_json::Value;

pub struct GuestMode {
    rt: Arc<Runtime>,
    api_base_url: String,
    guest_name: String,
    guest_email: String,
    message: String,
    current_dj: Option<Value>,
    previous_dj: Option<Value>,
    request_status: RequestStatus,
    qr_code_visible: bool,
}

#[derive(Debug, Clone)]
enum RequestStatus {
    None,
    Sending,
    Sent,
    Error(String),
}

impl GuestMode {
    pub fn new(rt: Arc<Runtime>, api_base_url: String) -> Self {
        Self {
            rt,
            api_base_url,
            guest_name: String::new(),
            guest_email: String::new(),
            message: String::new(),
            current_dj: None,
            previous_dj: None,
            request_status: RequestStatus::None,
            qr_code_visible: false,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("👥 Guest Mode");
        ui.add_space(10.0);

        // Show current DJ information
        self.render_current_dj_info(ui);
        ui.add_space(20.0);

        // QR Code section
        self.render_qr_section(ui);
        ui.add_space(20.0);

        // Request form (only show if QR code is "scanned")
        if self.qr_code_visible {
            self.render_request_form(ui);
        }
    }

    fn render_current_dj_info(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("🎵 Now Playing");
            ui.add_space(10.0);

            if let Some(current_dj) = &self.current_dj {
                ui.horizontal(|ui| {
                    ui.strong("Current DJ:");
                    ui.label(current_dj.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown"));
                });

                // Show session duration if available
                ui.horizontal(|ui| {
                    ui.label("⏱️ Playing for:");
                    ui.label("45:30"); // TODO: Calculate actual duration
                });
            } else {
                ui.label("No DJ currently playing");
            }

            ui.add_space(10.0);

            if let Some(previous_dj) = &self.previous_dj {
                ui.horizontal(|ui| {
                    ui.label("Previous DJ:");
                    ui.label(previous_dj.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown"));
                });
            }
        });
    }

    fn render_qr_section(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("📱 Request DJ Set");
            ui.add_space(10.0);

            if !self.qr_code_visible {
                // Show QR code placeholder
                ui.vertical_centered(|ui| {
                    // Draw a simple QR code placeholder
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(200.0, 200.0),
                        egui::Sense::click()
                    );

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

                    // Draw QR code pattern (simplified)
                    let cell_size = 8.0;
                    for x in 0..25 {
                        for y in 0..25 {
                            if (x + y) % 3 == 0 {
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

                    ui.add_space(10.0);
                    ui.label("📱 Scan QR Code to Request Set");
                    
                    // Simulate QR code scan with button click
                    if ui.button("🔍 Simulate QR Scan").clicked() {
                        self.qr_code_visible = true;
                    }
                });
            } else {
                ui.colored_label(egui::Color32::GREEN, "✅ QR Code Scanned!");
                ui.label("Fill out the form below to request the current or previous DJ's set.");
                
                if ui.button("🔄 Reset").clicked() {
                    self.qr_code_visible = false;
                    self.reset_form();
                }
            }
        });
    }

    fn render_request_form(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("📝 Request Form");
            ui.add_space(10.0);

            match &self.request_status {
                RequestStatus::Sent => {
                    ui.colored_label(egui::Color32::GREEN, "✅ Request sent successfully!");
                    ui.label("The DJ will receive an email and can respond directly to you.");
                    ui.add_space(10.0);
                    if ui.button("🆕 New Request").clicked() {
                        self.reset_form();
                    }
                    return;
                }
                RequestStatus::Sending => {
                    ui.label("⏳ Sending request...");
                    return;
                }
                RequestStatus::Error(error) => {
                    ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
                    ui.add_space(10.0);
                }
                RequestStatus::None => {}
            }

            // Target DJ selection
            ui.horizontal(|ui| {
                ui.label("Request set from:");
                ui.radio_value(&mut true, true, "🎵 Current DJ");
                ui.radio_value(&mut false, true, "⏮️ Previous DJ");
            });

            ui.add_space(10.0);

            // Contact information
            ui.horizontal(|ui| {
                ui.label("Your Name:");
                ui.text_edit_singleline(&mut self.guest_name);
            });

            ui.horizontal(|ui| {
                ui.label("Your Email:");
                ui.text_edit_singleline(&mut self.guest_email);
            });

            ui.add_space(10.0);

            // Optional message
            ui.label("Message (optional):");
            ui.text_edit_multiline(&mut self.message);

            ui.add_space(10.0);

            // Send button
            let can_send = !self.guest_name.trim().is_empty() 
                && !self.guest_email.trim().is_empty()
                && self.guest_email.contains('@');

            if ui.add_enabled(can_send, egui::Button::new("📤 Send Request"))
                .clicked() && can_send {
                self.send_request();
            }

            if !can_send {
                ui.colored_label(egui::Color32::YELLOW, "⚠️ Please fill in name and valid email");
            }
        });
    }

    fn send_request(&mut self) {
        self.request_status = RequestStatus::Sending;

        // TODO: Implement actual HTTP request
        // For now, simulate sending
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
        });

        // Simulate successful send
        self.request_status = RequestStatus::Sent;
    }

    fn reset_form(&mut self) {
        self.guest_name.clear();
        self.guest_email.clear();
        self.message.clear();
        self.request_status = RequestStatus::None;
    }
}