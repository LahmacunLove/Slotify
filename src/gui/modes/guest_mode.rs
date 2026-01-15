use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;
use serde_json::Value;
use crate::api_client::{ApiClient, EventSessionResponse, Timetable};

pub struct GuestMode {
    rt: Arc<Runtime>,
    api_base_url: String,
    api_client: ApiClient,
    guest_name: String,
    guest_email: String,
    message: String,
    current_dj: Option<Value>,
    previous_dj: Option<Value>,
    request_status: RequestStatus,
    qr_code_visible: bool,
    current_event: Option<EventSessionResponse>,
    timetable: Option<Timetable>,
    last_refresh: std::time::Instant,
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
        let api_client = ApiClient::new(api_base_url.clone());
        Self {
            rt,
            api_base_url,
            api_client,
            guest_name: String::new(),
            guest_email: String::new(),
            message: String::new(),
            current_dj: None,
            previous_dj: None,
            request_status: RequestStatus::None,
            qr_code_visible: false,
            current_event: None,
            timetable: None,
            last_refresh: std::time::Instant::now(),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Auto-refresh every 2 seconds
        if self.last_refresh.elapsed() > std::time::Duration::from_secs(2) {
            self.refresh_event_status();
            self.refresh_timetable();
            self.last_refresh = std::time::Instant::now();
        }

        // Two-column layout (no main heading)
        ui.columns(2, |columns| {
            // LEFT COLUMN: Timetable (no heading, more space)
            self.render_timetable(&mut columns[0]);

            // RIGHT COLUMN: Guest Request Features (no heading)
            self.render_qr_section(&mut columns[1]);
            columns[1].add_space(20.0);
            if self.qr_code_visible {
                self.render_request_form(&mut columns[1]);
            }
        });

        // Event status in footer
        ui.add_space(20.0);
        ui.separator();
        self.render_event_status(ui);
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
            ui.vertical_centered(|ui| {
                ui.heading("📱 Request DJ Set");
            });
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

    fn render_event_status(&mut self, ui: &mut egui::Ui) {
        // Compact footer-style event status
        ui.horizontal(|ui| {
            if let Some(event) = &self.current_event {
                ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "🟢 Event Active");
                ui.separator();
                ui.label(format!("⏱️ {} min elapsed", event.elapsed_minutes));
                ui.separator();
                ui.label(format!("Slot: {} min", event.slot_duration_minutes));

                if let Some(current_dj_name) = &event.current_dj_name {
                    ui.separator();
                    ui.strong("🎵");
                    ui.label(current_dj_name);

                    if let Some(progress) = event.current_slot_progress_percent {
                        ui.separator();
                        let progress_bar = egui::ProgressBar::new(progress / 100.0)
                            .text(format!("{:.0}%", progress));
                        ui.add(progress_bar);
                    }
                }
            } else {
                ui.colored_label(egui::Color32::from_rgb(200, 0, 0), "🔴 No event running");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Refresh").clicked() {
                    self.refresh_event_status();
                    self.refresh_timetable();
                }
            });
        });
    }

    fn render_timetable(&mut self, ui: &mut egui::Ui) {
        // No heading, more space for content
        ui.group(|ui| {
            if let Some(timetable) = &self.timetable {
                ui.horizontal(|ui| {
                    ui.label(format!("📊 {} DJs", timetable.total_djs));
                    ui.separator();
                    ui.label(format!("✅ {} completed", timetable.completed_sets));
                });
                ui.add_space(5.0);

                // Fixed height timetable
                egui::ScrollArea::vertical()
                    .min_scrolled_height(400.0)
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for entry in &timetable.entries {
                            ui.horizontal(|ui| {
                                // Parse and format start time as HH:MM
                                let start_time = if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(&entry.started_at) {
                                    datetime.format("%H:%M").to_string()
                                } else {
                                    "--:--".to_string()
                                };

                                // Status indicator
                                let status_indicator = match entry.status.as_str() {
                                    "Completed" => "✅",
                                    "InProgress" => "▶️",
                                    _ => "⏳",
                                };

                                ui.label(status_indicator);
                                ui.label(start_time);
                                ui.label("|");
                                ui.label(&entry.dj_name);
                            });
                        }
                    });
            } else {
                // Fixed height even when empty
                ui.vertical_centered(|ui| {
                    ui.add_space(150.0);
                    ui.label("📋 No timetable yet");
                    ui.label("Start an event in Admin mode");
                    ui.add_space(150.0);
                });
            }
        });
    }

    fn refresh_event_status(&mut self) {
        match self.api_client.get_current_event() {
            Ok(event) => {
                self.current_event = event;
            }
            Err(_) => {
                self.current_event = None;
            }
        }
    }

    fn refresh_timetable(&mut self) {
        match self.api_client.get_timetable() {
            Ok(timetable) => {
                self.timetable = timetable;
            }
            Err(_) => {
                self.timetable = None;
            }
        }
    }
}