use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;
use serde_json::Value;

pub struct AdminMode {
    rt: Arc<Runtime>,
    api_base_url: String,
    is_authenticated: bool,
    admin_password: String,
    dj_pool: Vec<Value>,
    current_queue: Vec<Value>,
    selected_dj: Option<usize>,
    new_dj_name: String,
    new_dj_email: String,
    session_duration: String,
    lottery_stats: Option<Value>,
}

impl AdminMode {
    pub fn new(rt: Arc<Runtime>, api_base_url: String) -> Self {
        Self {
            rt,
            api_base_url,
            is_authenticated: false,
            admin_password: String::new(),
            dj_pool: Vec::new(),
            current_queue: Vec::new(),
            selected_dj: None,
            new_dj_name: String::new(),
            new_dj_email: String::new(),
            session_duration: "60".to_string(),
            lottery_stats: None,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ Admin Mode");
        ui.add_space(10.0);

        if !self.is_authenticated {
            self.render_login(ui);
        } else {
            self.render_admin_interface(ui);
        }
    }

    fn render_login(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("🔐 Admin Authentication");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Password:");
                ui.text_edit_singleline(&mut self.admin_password);
            });

            ui.add_space(10.0);

            if ui.button("🔓 Login").clicked() {
                // Simple password check (in production, use proper authentication)
                if self.admin_password == "admin123" {
                    self.is_authenticated = true;
                    self.load_admin_data();
                } else {
                    self.admin_password.clear();
                }
            }

            if !self.admin_password.is_empty() && self.admin_password != "admin123" {
                ui.colored_label(egui::Color32::RED, "❌ Invalid password");
            }

            ui.add_space(10.0);
            ui.colored_label(egui::Color32::GRAY, "Demo password: admin123");
        });
    }

    fn render_admin_interface(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("🚪 Logout").clicked() {
                self.is_authenticated = false;
                self.admin_password.clear();
            }

            ui.separator();

            if ui.button("🔄 Refresh Data").clicked() {
                self.load_admin_data();
            }

            ui.separator();

            if ui.button("🎲 Draw Next DJ").clicked() {
                self.draw_next_dj();
            }
        });

        ui.add_space(20.0);

        // Main admin content in columns
        ui.columns(2, |columns| {
            // Left column: DJ Pool Management
            columns[0].group(|ui| {
                ui.heading("🎭 DJ Pool Management");
                ui.add_space(10.0);

                self.render_dj_pool(&mut columns[0]);
            });

            // Right column: Queue Management
            columns[1].group(|ui| {
                ui.heading("📋 Queue Management");
                ui.add_space(10.0);

                self.render_queue_management(&mut columns[1]);
            });
        });

        ui.add_space(20.0);

        // Statistics and controls
        self.render_statistics_panel(ui);
    }

    fn render_dj_pool(&mut self, ui: &mut egui::Ui) {
        // Add new DJ form
        ui.group(|ui| {
            ui.label("➕ Add New DJ");
            
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.new_dj_name);
            });

            ui.horizontal(|ui| {
                ui.label("Email:");
                ui.text_edit_singleline(&mut self.new_dj_email);
            });

            let can_add = !self.new_dj_name.trim().is_empty();
            if ui.add_enabled(can_add, egui::Button::new("➕ Add DJ"))
                .clicked() && can_add {
                self.add_dj();
            }
        });

        ui.add_space(10.0);

        // DJ Pool List
        ui.label(format!("📊 Pool: {} DJs", self.dj_pool.len()));
        
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for (i, dj) in self.dj_pool.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let is_selected = self.selected_dj == Some(i);
                        if ui.selectable_label(is_selected, "").clicked() {
                            self.selected_dj = if is_selected { None } else { Some(i) };
                        }

                        ui.label(dj.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown"));

                        ui.separator();

                        if ui.small_button("❌").clicked() {
                            self.remove_dj(i);
                        }

                        if ui.small_button("⬆️").clicked() {
                            self.promote_dj_to_queue(i);
                        }
                    });
                }
            });
    }

    fn render_queue_management(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("🎵 Queue: {} DJs", self.current_queue.len()));

        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                for (i, dj) in self.current_queue.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}.", i + 1));
                        
                        ui.label(dj.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown"));

                        ui.separator();

                        // Position controls
                        if i > 0 && ui.small_button("⬆️").clicked() {
                            self.move_dj_up_in_queue(i);
                        }

                        if i < self.current_queue.len() - 1 && ui.small_button("⬇️").clicked() {
                            self.move_dj_down_in_queue(i);
                        }

                        if ui.small_button("❌").clicked() {
                            self.remove_from_queue(i);
                        }
                    });
                }
            });

        ui.add_space(10.0);

        // B2B Session controls
        ui.group(|ui| {
            ui.label("🤝 B2B Session");
            
            ui.horizontal(|ui| {
                ui.label("Duration (min):");
                ui.text_edit_singleline(&mut self.session_duration);
            });

            if ui.button("🎧 Create B2B (Top 2)").clicked() {
                self.create_b2b_session();
            }
        });
    }

    fn render_statistics_panel(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("📊 System Statistics");
            ui.add_space(10.0);

            if let Some(stats) = &self.lottery_stats {
                ui.horizontal(|ui| {
                    ui.label("Total Draws:");
                    ui.label(stats.get("total_draws")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                        .to_string());
                });

                ui.horizontal(|ui| {
                    ui.label("Unique Winners:");
                    ui.label(stats.get("unique_winners")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                        .to_string());
                });

                ui.horizontal(|ui| {
                    ui.label("Fairness Score:");
                    ui.label(format!("{:.2}%", 
                        stats.get("fairness_score")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) * 100.0));
                });
            } else {
                ui.label("Loading statistics...");
            }

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("🔄 Reset Lottery").clicked() {
                    self.reset_lottery();
                }

                if ui.button("📊 Refresh Stats").clicked() {
                    self.load_lottery_stats();
                }
            });
        });
    }

    fn load_admin_data(&mut self) {
        // TODO: Implement actual HTTP requests
        // For now, simulate data
        self.dj_pool = vec![
            serde_json::json!({"id": "dj1", "name": "DJ Alpha", "email": "alpha@example.com"}),
            serde_json::json!({"id": "dj2", "name": "DJ Beta", "email": "beta@example.com"}),
            serde_json::json!({"id": "dj3", "name": "DJ Gamma", "email": "gamma@example.com"}),
        ];

        self.current_queue = vec![
            serde_json::json!({"id": "dj4", "name": "DJ Queue First", "position": 1}),
            serde_json::json!({"id": "dj5", "name": "DJ Queue Second", "position": 2}),
        ];

        self.load_lottery_stats();
    }

    fn load_lottery_stats(&mut self) {
        // TODO: Implement actual HTTP request
        self.lottery_stats = Some(serde_json::json!({
            "total_draws": 15,
            "unique_winners": 8,
            "fairness_score": 0.73
        }));
    }

    fn add_dj(&mut self) {
        // TODO: Implement HTTP request
        let new_dj = serde_json::json!({
            "id": format!("dj-{}", uuid::Uuid::new_v4()),
            "name": self.new_dj_name.clone(),
            "email": self.new_dj_email.clone()
        });

        self.dj_pool.push(new_dj);
        self.new_dj_name.clear();
        self.new_dj_email.clear();
    }

    fn remove_dj(&mut self, index: usize) {
        if index < self.dj_pool.len() {
            self.dj_pool.remove(index);
            if self.selected_dj == Some(index) {
                self.selected_dj = None;
            }
        }
    }

    fn promote_dj_to_queue(&mut self, index: usize) {
        if index < self.dj_pool.len() {
            let dj = self.dj_pool.remove(index);
            self.current_queue.push(dj);
        }
    }

    fn move_dj_up_in_queue(&mut self, index: usize) {
        if index > 0 && index < self.current_queue.len() {
            self.current_queue.swap(index - 1, index);
        }
    }

    fn move_dj_down_in_queue(&mut self, index: usize) {
        if index < self.current_queue.len() - 1 {
            self.current_queue.swap(index, index + 1);
        }
    }

    fn remove_from_queue(&mut self, index: usize) {
        if index < self.current_queue.len() {
            let dj = self.current_queue.remove(index);
            self.dj_pool.push(dj);
        }
    }

    fn create_b2b_session(&mut self) {
        // TODO: Implement B2B session creation
        if self.current_queue.len() >= 2 {
            // Remove top 2 DJs and create B2B session
            // For now, just show a message
        }
    }

    fn draw_next_dj(&mut self) {
        // TODO: Implement lottery draw API call
        if !self.dj_pool.is_empty() {
            let selected = rand::random::<usize>() % self.dj_pool.len();
            let dj = self.dj_pool.remove(selected);
            self.current_queue.push(dj);
        }
    }

    fn reset_lottery(&mut self) {
        // TODO: Implement lottery reset API call
        self.current_queue.clear();
        self.load_admin_data();
    }
}