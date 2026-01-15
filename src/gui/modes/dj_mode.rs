use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;
use crate::api_client::{ApiClient, DjResponse, EventSessionResponse};

pub struct DjMode {
    rt: Arc<Runtime>,
    api_base_url: String,
    api_client: ApiClient,
    dj_name: String,
    dj_email: String,
    registration_status: RegistrationStatus,
    current_queue: Vec<DjResponse>,
    next_dj: Option<DjResponse>,
    current_event: Option<EventSessionResponse>,
    error_message: Option<String>,
    success_message: Option<String>,
}

#[derive(Debug, Clone)]
enum RegistrationStatus {
    NotRegistered,
    Registering,
    Registered(String), // DJ ID
    Error(String),
}

impl DjMode {
    pub fn new(rt: Arc<Runtime>, api_base_url: String) -> Self {
        let api_client = ApiClient::new(api_base_url.clone());
        Self {
            rt,
            api_base_url,
            api_client,
            dj_name: String::new(),
            dj_email: String::new(),
            registration_status: RegistrationStatus::NotRegistered,
            current_queue: Vec::new(),
            next_dj: None,
            current_event: None,
            error_message: None,
            success_message: None,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("📝 DJ Registration");
        ui.add_space(10.0);

        // Display error/success messages
        if let Some(error) = &self.error_message {
            ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
            ui.add_space(5.0);
        }
        if let Some(success) = &self.success_message {
            ui.colored_label(egui::Color32::GREEN, format!("✅ {}", success));
            ui.add_space(5.0);
        }

        // Two-column layout
        ui.columns(2, |columns| {
            // LEFT COLUMN: Registration
            match &self.registration_status {
                RegistrationStatus::NotRegistered => {
                    self.render_registration_form(&mut columns[0]);
                }
                RegistrationStatus::Registering => {
                    columns[0].label("⏳ Registering for lottery...");
                }
                RegistrationStatus::Registered(dj_id) => {
                    let dj_id_clone = dj_id.clone();
                    self.render_registered_interface(&mut columns[0], &dj_id_clone);
                }
                RegistrationStatus::Error(error) => {
                    columns[0].colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
                    columns[0].add_space(10.0);
                    if columns[0].button("🔄 Try Again").clicked() {
                        self.registration_status = RegistrationStatus::NotRegistered;
                        self.error_message = None;
                    }
                }
            }

            // RIGHT COLUMN: Current Queue
            self.render_current_queue(&mut columns[1]);
        });
    }

    fn render_event_status(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("📅 Event Status");
            ui.add_space(5.0);

            if let Some(event) = &self.current_event {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "🟢 Event Active");
                    ui.label(format!("| Running for {} minutes", event.elapsed_minutes));
                });

                ui.add_space(5.0);

                if let Some(current_dj_name) = &event.current_dj_name {
                    ui.horizontal(|ui| {
                        ui.strong("🎵 Currently playing:");
                        ui.label(current_dj_name);
                    });

                    if let Some(progress) = event.current_slot_progress_percent {
                        ui.add_space(3.0);
                        ui.horizontal(|ui| {
                            ui.label(format!("Slot progress: {:.0}%", progress));
                            let progress_bar = egui::ProgressBar::new(progress / 100.0)
                                .text(format!("{:.0}%", progress));
                            ui.add(progress_bar);
                        });
                    }
                } else {
                    ui.label("⏳ Waiting for first DJ to start...");
                }

                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label(format!("⏱️  Slot duration: {} minutes", event.slot_duration_minutes));
                    ui.separator();
                    ui.label(format!("⚠️  Late penalty after: {} hours", event.late_arrival_cutoff_hours));
                });
            } else {
                ui.colored_label(egui::Color32::from_rgb(200, 0, 0), "🔴 No event running");
                ui.label("Waiting for admin to start the event...");
            }
        });

        if ui.button("🔄 Refresh Event Status").clicked() {
            self.refresh_event_status();
        }
    }

    fn render_registration_form(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("Register for DJ Lottery");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.dj_name);
            });

            ui.horizontal(|ui| {
                ui.label("Email (optional):");
                ui.text_edit_singleline(&mut self.dj_email);
            });

            ui.add_space(10.0);

            let can_register = !self.dj_name.trim().is_empty();
            
            if ui.add_enabled(can_register, egui::Button::new("🎲 Register for Lottery"))
                .clicked() && can_register {
                self.register_dj();
            }

            if !can_register {
                ui.colored_label(egui::Color32::YELLOW, "⚠️ Please enter your name");
            }
        });
    }

    fn render_registered_interface(&mut self, ui: &mut egui::Ui, dj_id: &str) {
        ui.group(|ui| {
            ui.heading(format!("✅ Registered: {}", self.dj_name));
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("🗑️ Remove from Lottery").clicked() {
                    self.remove_from_lottery(dj_id);
                }

                ui.separator();

                if ui.button("➕ Register Another DJ").clicked() {
                    // Reset to registration form but keep current DJ registered
                    self.registration_status = RegistrationStatus::NotRegistered;
                    self.dj_name.clear();
                    self.dj_email.clear();
                    self.error_message = None;
                    self.success_message = Some("Previous DJ still registered. Add another!".to_string());
                }
            });

            ui.add_space(10.0);

            // Show position in queue if available
            if let Some(next_dj) = &self.next_dj {
                if next_dj.id == dj_id {
                    ui.colored_label(egui::Color32::GREEN, "🎉 You're next up!");
                } else {
                    ui.label("⏳ Waiting in lottery pool...");
                }
            } else {
                ui.label("⏳ Waiting in lottery pool...");
            }
        });
    }

    fn render_current_queue(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("Current Queue");
            ui.add_space(10.0);

            if let Some(next_dj) = &self.next_dj {
                ui.horizontal(|ui| {
                    ui.strong("🎵 Next DJ:");
                    ui.label(&next_dj.name);
                });
                ui.add_space(10.0);
            }

            if self.current_queue.is_empty() {
                ui.label("No DJs in queue yet");
            } else {
                ui.label(format!("📋 {} DJs in queue", self.current_queue.len()));

                for (i, dj) in self.current_queue.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}.", dj.position_in_queue.unwrap_or(i as i32 + 1)));
                        ui.label(&dj.name);

                        let registered_date = dj.registered_at.split('T').next().unwrap_or("");
                        ui.label(format!("(registered: {})", registered_date));
                    });
                }
            }
        });

        // Auto-refresh queue
        if ui.button("🔄 Refresh").clicked() {
            self.refresh_queue();
        }
    }

    fn register_dj(&mut self) {
        self.error_message = None;
        self.success_message = None;

        if self.dj_name.trim().is_empty() {
            self.error_message = Some("DJ name cannot be empty".to_string());
            return;
        }

        let name = self.dj_name.clone();
        let email = if self.dj_email.trim().is_empty() {
            "".to_string()
        } else {
            self.dj_email.clone()
        };

        match self.api_client.register_dj(name, email) {
            Ok(dj) => {
                self.registration_status = RegistrationStatus::Registered(dj.id.clone());
                self.success_message = Some(format!("Successfully registered as '{}'!", dj.name));
                self.refresh_queue();
            }
            Err(e) => {
                self.registration_status = RegistrationStatus::Error(e.clone());
                self.error_message = Some(format!("Failed to register: {}", e));
            }
        }
    }

    fn remove_from_lottery(&mut self, dj_id: &str) {
        self.error_message = None;
        self.success_message = None;

        match self.api_client.delete_dj(dj_id) {
            Ok(_) => {
                self.registration_status = RegistrationStatus::NotRegistered;
                let name = self.dj_name.clone();
                self.dj_name.clear();
                self.dj_email.clear();
                self.success_message = Some(format!("'{}' removed from lottery", name));
                self.refresh_queue();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to remove from lottery: {}", e));
            }
        }
    }

    fn refresh_queue(&mut self) {
        // Get lottery queue (DJs that have been drawn)
        match self.api_client.get_lottery_queue() {
            Ok(queue) => {
                self.current_queue = queue.clone();
                // Next DJ is the first in queue
                self.next_dj = queue.first().cloned();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load queue: {}", e));
            }
        }
    }

    fn refresh_event_status(&mut self) {
        match self.api_client.get_current_event() {
            Ok(event) => {
                self.current_event = event;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load event status: {}", e));
            }
        }
    }
}