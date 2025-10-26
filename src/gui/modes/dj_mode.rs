use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;
use serde_json::Value;

pub struct DjMode {
    rt: Arc<Runtime>,
    api_base_url: String,
    dj_name: String,
    dj_email: String,
    registration_status: RegistrationStatus,
    current_queue: Vec<Value>,
    next_dj: Option<Value>,
    error_message: Option<String>,
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
        Self {
            rt,
            api_base_url,
            dj_name: String::new(),
            dj_email: String::new(),
            registration_status: RegistrationStatus::NotRegistered,
            current_queue: Vec::new(),
            next_dj: None,
            error_message: None,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎧 DJ Mode");
        ui.add_space(10.0);

        match &self.registration_status {
            RegistrationStatus::NotRegistered => {
                self.render_registration_form(ui);
            }
            RegistrationStatus::Registering => {
                ui.label("⏳ Registering for lottery...");
            }
            RegistrationStatus::Registered(dj_id) => {
                self.render_registered_interface(ui, dj_id);
            }
            RegistrationStatus::Error(error) => {
                ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
                ui.add_space(10.0);
                if ui.button("🔄 Try Again").clicked() {
                    self.registration_status = RegistrationStatus::NotRegistered;
                    self.error_message = None;
                }
            }
        }

        ui.add_space(20.0);
        self.render_current_queue(ui);
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

                if ui.button("🎵 Start Session").clicked() {
                    self.start_session(dj_id);
                }
            });

            ui.add_space(10.0);

            // Show position in queue if available
            if let Some(next_dj) = &self.next_dj {
                if next_dj.get("id").and_then(|v| v.as_str()) == Some(dj_id) {
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
                    ui.label(next_dj.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown"));
                });
                ui.add_space(10.0);
            }

            if self.current_queue.is_empty() {
                ui.label("No DJs in queue yet");
            } else {
                ui.label(format!("📋 {} DJs in lottery pool", self.current_queue.len()));
                
                for (i, dj) in self.current_queue.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}.", i + 1));
                        ui.label(dj.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown"));
                        
                        if let Some(registered_at) = dj.get("registered_at")
                            .and_then(|v| v.as_str()) {
                            ui.label(format!("(registered: {})", 
                                registered_at.split('T').next().unwrap_or("")));
                        }
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
        self.registration_status = RegistrationStatus::Registering;
        
        let rt = self.rt.clone();
        let api_url = format!("{}/djs/register", self.api_base_url);
        let name = self.dj_name.clone();
        let email = if self.dj_email.trim().is_empty() {
            None
        } else {
            Some(self.dj_email.clone())
        };

        // TODO: Implement actual HTTP request
        // For now, simulate registration
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
            // Simulate successful registration
            // In real implementation, make HTTP request here
        });

        // Simulate successful registration for demo
        self.registration_status = RegistrationStatus::Registered("demo-dj-id".to_string());
    }

    fn remove_from_lottery(&mut self, dj_id: &str) {
        // TODO: Implement HTTP request to remove DJ
        self.registration_status = RegistrationStatus::NotRegistered;
        self.dj_name.clear();
        self.dj_email.clear();
    }

    fn start_session(&mut self, dj_id: &str) {
        // TODO: Implement HTTP request to start session
        // For now, just show a message
        self.error_message = Some("Session starting feature will be implemented".to_string());
    }

    fn refresh_queue(&mut self) {
        // TODO: Implement HTTP request to get current queue
        // For now, simulate some data
        self.current_queue = vec![
            serde_json::json!({
                "id": "dj1",
                "name": "DJ Example",
                "registered_at": "2024-01-01T12:00:00Z"
            }),
            serde_json::json!({
                "id": "dj2", 
                "name": "Another DJ",
                "registered_at": "2024-01-01T12:30:00Z"
            }),
        ];

        self.next_dj = Some(serde_json::json!({
            "id": "next-dj",
            "name": "Next Up DJ"
        }));
    }
}