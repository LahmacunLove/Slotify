use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;
use serde_json::Value;
use crate::api_client::{ApiClient, DjResponse, EventSessionResponse, Timetable};

pub struct AdminMode {
    rt: Arc<Runtime>,
    api_base_url: String,
    api_client: ApiClient,
    is_authenticated: bool,
    admin_password: String,
    dj_pool: Vec<DjResponse>,
    current_queue: Vec<DjResponse>,
    current_event: Option<EventSessionResponse>,
    timetable: Option<Timetable>,
    selected_dj: Option<usize>,
    new_dj_name: String,
    new_dj_email: String,
    session_duration: String,
    event_slot_duration: String,
    event_late_cutoff: String,
    event_start_time: String, // Format: HH:MM (e.g., "20:00")
    lottery_stats: Option<Value>,
    error_message: Option<String>,
    success_message: Option<String>,
    show_stop_confirmation: bool,
    show_clear_confirmation: bool,
    last_refresh: std::time::Instant,
}

impl AdminMode {
    pub fn new(rt: Arc<Runtime>, api_base_url: String) -> Self {
        let api_client = ApiClient::new(api_base_url.clone());
        Self {
            rt,
            api_base_url,
            api_client,
            is_authenticated: false,
            admin_password: String::new(),
            dj_pool: Vec::new(),
            current_queue: Vec::new(),
            current_event: None,
            timetable: None,
            selected_dj: None,
            new_dj_name: String::new(),
            new_dj_email: String::new(),
            session_duration: "60".to_string(),
            event_slot_duration: "60".to_string(),
            event_late_cutoff: "2".to_string(),
            event_start_time: String::new(), // Empty = start immediately
            lottery_stats: None,
            error_message: None,
            success_message: None,
            show_stop_confirmation: false,
            show_clear_confirmation: false,
            last_refresh: std::time::Instant::now(),
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Auto-refresh every 3 seconds (when authenticated)
        if self.is_authenticated && self.last_refresh.elapsed() > std::time::Duration::from_secs(3) {
            self.refresh_event_status();
            self.load_admin_data();
            self.last_refresh = std::time::Instant::now();
        }

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
        // Display event status
        self.render_event_controls(ui);
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
            columns[0].heading("🎭 DJ Pool Management");
            columns[0].add_space(10.0);
            self.render_dj_pool(&mut columns[0]);

            // Right column: Queue Management
            columns[1].heading("📋 Queue Management");
            columns[1].add_space(10.0);
            self.render_queue_management(&mut columns[1]);
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
        
        let mut action: Option<(usize, &str)> = None;

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for (i, dj) in self.dj_pool.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let is_selected = self.selected_dj == Some(i);
                        if ui.selectable_label(is_selected, "").clicked() {
                            self.selected_dj = if is_selected { None } else { Some(i) };
                        }

                        ui.label(&dj.name);

                        ui.separator();

                        if ui.small_button("❌").clicked() {
                            action = Some((i, "remove"));
                        }

                        if ui.small_button("⬆️").clicked() {
                            action = Some((i, "promote"));
                        }
                    });
                }
            });

        // Process action outside the loop
        if let Some((idx, act)) = action {
            match act {
                "remove" => self.remove_dj(idx),
                "promote" => self.promote_dj_to_queue(idx),
                _ => {}
            }
        }
    }

    fn render_queue_management(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("🎵 Queue: {} DJs", self.current_queue.len()));

        let mut queue_action: Option<(usize, &str)> = None;
        let queue_len = self.current_queue.len();

        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                for (i, dj) in self.current_queue.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}.", i + 1));

                        ui.label(&dj.name);

                        ui.separator();

                        // Position controls
                        if i > 0 && ui.small_button("⬆️").clicked() {
                            queue_action = Some((i, "up"));
                        }

                        if i < queue_len - 1 && ui.small_button("⬇️").clicked() {
                            queue_action = Some((i, "down"));
                        }

                        if ui.small_button("❌").clicked() {
                            queue_action = Some((i, "remove"));
                        }
                    });
                }
            });

        // Process action outside the loop
        if let Some((idx, act)) = queue_action {
            match act {
                "up" => self.move_dj_up_in_queue(idx),
                "down" => self.move_dj_down_in_queue(idx),
                "remove" => self.remove_from_queue(idx),
                _ => {}
            }
        }

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
        self.error_message = None;
        self.success_message = None;

        // Load all DJs
        match self.api_client.get_all_djs() {
            Ok(djs) => {
                // Filter out DJs that are already in queue
                self.dj_pool = djs.into_iter()
                    .filter(|dj| dj.position_in_queue.is_none())
                    .collect();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load DJs: {}", e));
            }
        }

        // Load current queue
        match self.api_client.get_lottery_queue() {
            Ok(queue) => {
                self.current_queue = queue;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load queue: {}", e));
            }
        }

        self.load_lottery_stats();
    }

    fn load_lottery_stats(&mut self) {
        match self.api_client.get_lottery_statistics() {
            Ok(stats) => {
                self.lottery_stats = Some(stats);
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load stats: {}", e));
            }
        }
    }

    fn add_dj(&mut self) {
        self.error_message = None;
        self.success_message = None;

        if self.new_dj_name.is_empty() {
            self.error_message = Some("DJ name cannot be empty".to_string());
            return;
        }

        let name = self.new_dj_name.clone();
        let email = self.new_dj_email.clone();

        match self.api_client.register_dj(name, email) {
            Ok(dj) => {
                self.dj_pool.push(dj.clone());
                self.success_message = Some(format!("DJ '{}' registered successfully!", dj.name));
                self.new_dj_name.clear();
                self.new_dj_email.clear();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to register DJ: {}", e));
            }
        }
    }

    fn remove_dj(&mut self, index: usize) {
        self.error_message = None;
        self.success_message = None;

        if index < self.dj_pool.len() {
            let dj = &self.dj_pool[index];
            let dj_id = dj.id.clone();
            let dj_name = dj.name.clone();

            match self.api_client.delete_dj(&dj_id) {
                Ok(_) => {
                    self.dj_pool.remove(index);
                    if self.selected_dj == Some(index) {
                        self.selected_dj = None;
                    }
                    self.success_message = Some(format!("DJ '{}' removed successfully!", dj_name));
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to remove DJ: {}", e));
                }
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
        self.error_message = None;
        self.success_message = None;

        match self.api_client.draw_next_dj() {
            Ok(draw_result) => {
                self.success_message = Some(format!("Drew DJ: {}!", draw_result.winner.name));
                // Reload data to reflect changes
                self.load_admin_data();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to draw DJ: {}", e));
            }
        }
    }

    fn reset_lottery(&mut self) {
        self.error_message = None;
        self.success_message = None;

        match self.api_client.reset_lottery() {
            Ok(_) => {
                self.success_message = Some("Lottery reset successfully!".to_string());
                self.load_admin_data();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to reset lottery: {}", e));
            }
        }
    }

    fn render_event_controls(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("🎉 Event Session Control");
            ui.add_space(5.0);

            if let Some(event) = &self.current_event {
                // Event is running
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
                    if ui.button("🛑 Stop Event").clicked() {
                        self.show_stop_confirmation = true;
                    }

                    ui.separator();

                    if ui.button("📋 View Timetable").clicked() {
                        self.load_timetable();
                    }

                    if ui.button("🔄 Refresh").clicked() {
                        self.refresh_event_status();
                    }
                });

                // Stop Event Confirmation Dialog
                if self.show_stop_confirmation {
                    ui.add_space(10.0);
                    ui.group(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "⚠️ Stop Event?");
                        ui.label("This will end the current event session.");
                        ui.horizontal(|ui| {
                            if ui.button("✅ Yes, Stop Event").clicked() {
                                self.end_event();
                                self.show_stop_confirmation = false;
                            }
                            if ui.button("❌ Cancel").clicked() {
                                self.show_stop_confirmation = false;
                            }
                        });
                    });
                }

                // Show timetable if loaded
                if let Some(timetable) = &self.timetable {
                    ui.add_space(10.0);
                    ui.separator();
                    ui.heading("📋 Timetable");
                    ui.label(format!("Total DJs: {} | Completed sets: {}",
                        timetable.total_djs, timetable.completed_sets));

                    // Fixed height timetable
                    egui::ScrollArea::vertical()
                        .min_scrolled_height(250.0)
                        .max_height(250.0)
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
                }

            } else {
                // No event running
                ui.colored_label(egui::Color32::from_rgb(200, 0, 0), "🔴 No event running");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Slot duration (minutes):");
                    ui.text_edit_singleline(&mut self.event_slot_duration);
                });

                ui.horizontal(|ui| {
                    ui.label("Late penalty cutoff (hours):");
                    ui.text_edit_singleline(&mut self.event_late_cutoff);
                });

                ui.horizontal(|ui| {
                    ui.label("Start time (HH:MM, leave empty for now):");
                    ui.text_edit_singleline(&mut self.event_start_time);
                });
                ui.label("💡 Examples: '20:00' for 8 PM today, or leave empty to start immediately");

                ui.add_space(10.0);

                if ui.button("🎉 Start Event").clicked() {
                    self.start_event();
                }
            }

            // Clear All button (always visible)
            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            if ui.button("🗑️ Clear All Data").clicked() {
                self.show_clear_confirmation = true;
            }

            // Clear All Confirmation Dialog
            if self.show_clear_confirmation {
                ui.add_space(10.0);
                ui.group(|ui| {
                    ui.colored_label(egui::Color32::RED, "⚠️ WARNING: Clear All Data?");
                    ui.label("This will DELETE:");
                    ui.label("  • All DJ registrations");
                    ui.label("  • All sessions");
                    ui.label("  • Current event");
                    ui.label("  • Lottery queue");
                    ui.colored_label(egui::Color32::RED, "This action CANNOT be undone!");
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("✅ Yes, Clear Everything").clicked() {
                            self.clear_all_data();
                            self.show_clear_confirmation = false;
                        }
                        if ui.button("❌ Cancel").clicked() {
                            self.show_clear_confirmation = false;
                        }
                    });
                });
            }
        });
    }

    fn start_event(&mut self) {
        self.error_message = None;
        self.success_message = None;

        let slot_duration = self.event_slot_duration.parse::<i32>().ok();
        let late_cutoff = self.event_late_cutoff.parse::<i32>().ok();

        // Parse custom start time if provided (format: HH:MM)
        let started_at = if !self.event_start_time.trim().is_empty() {
            // Parse HH:MM format
            let parts: Vec<&str> = self.event_start_time.split(':').collect();
            if parts.len() == 2 {
                if let (Ok(hour), Ok(minute)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    if hour < 24 && minute < 60 {
                        // Get current date and set the time
                        let now = chrono::Utc::now();
                        let date = now.date_naive();
                        let time = chrono::NaiveTime::from_hms_opt(hour, minute, 0).unwrap();
                        let datetime = chrono::NaiveDateTime::new(date, time);
                        let datetime_utc = datetime.and_utc();
                        Some(datetime_utc.to_rfc3339())
                    } else {
                        self.error_message = Some("Invalid time format. Use HH:MM (e.g., 20:00)".to_string());
                        return;
                    }
                } else {
                    self.error_message = Some("Invalid time format. Use HH:MM (e.g., 20:00)".to_string());
                    return;
                }
            } else {
                self.error_message = Some("Invalid time format. Use HH:MM (e.g., 20:00)".to_string());
                return;
            }
        } else {
            None
        };

        match self.api_client.start_event(slot_duration, late_cutoff, started_at) {
            Ok(event) => {
                self.current_event = Some(event);
                self.success_message = Some("Event started successfully!".to_string());
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to start event: {}", e));
            }
        }
    }

    fn end_event(&mut self) {
        self.error_message = None;
        self.success_message = None;

        match self.api_client.end_event() {
            Ok(event) => {
                self.current_event = None;
                self.timetable = None;
                self.success_message = Some("Event ended successfully!".to_string());
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to end event: {}", e));
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

    fn load_timetable(&mut self) {
        match self.api_client.get_timetable() {
            Ok(timetable) => {
                self.timetable = timetable;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load timetable: {}", e));
            }
        }
    }

    fn clear_all_data(&mut self) {
        self.error_message = None;
        self.success_message = None;

        match self.api_client.clear_all_data() {
            Ok(_) => {
                // Clear local state
                self.current_event = None;
                self.timetable = None;
                self.dj_pool.clear();
                self.current_queue.clear();
                self.lottery_stats = None;
                self.success_message = Some("All data cleared successfully!".to_string());
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to clear data: {}", e));
            }
        }
    }
}