use eframe::egui;
use crate::client::{Client, AppMode};
use crate::encryption::{decrypt, encrypt, KEY_SIZE};
use std::fs;
use std::process::Command;

pub enum KeyStatus {
    Missing,
    Submitted,
    Incorrect,
    FirstRun,
}

pub struct AppState {
    pub clients: Vec<Client>,
    pub selected_client: Option<usize>,
    pub new_client_name: String,
    pub new_client_ip: String,
    pub new_client_password: String,
    pub mode: AppMode,
    pub show_password: bool,
    pub error_message: Option<String>,
    pub encryption_key: [u8; KEY_SIZE],
    pub master_key_input: String,
    pub confirm_master_key_input: String,
    pub key_status: KeyStatus,
}

impl AppState {
    pub fn new() -> Self {
        let key_status = if fs::metadata("clients.json").is_ok() {
            KeyStatus::Missing
        } else {
            KeyStatus::FirstRun
        };
        Self {
            clients: Vec::new(),
            selected_client: None,
            new_client_name: String::new(),
            new_client_ip: String::new(),
            new_client_password: String::new(),
            mode: AppMode::Normal,
            show_password: false,
            error_message: None,
            encryption_key: [0; KEY_SIZE],
            master_key_input: String::new(),
            confirm_master_key_input: String::new(),
            key_status,
        }
    }

    pub fn save_clients(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(&self.clients)?;
        let encrypted_data = encrypt(&data, &self.encryption_key).map_err(|e| e.to_string())?;
        fs::write("clients.json", encrypted_data)?;
        Ok(())
    }

    pub fn connect_to_client(&self, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.set_text(&client.password)?;

        Command::new("mstsc")
            .arg("/v")
            .arg(&client.ip)
            .arg("/prompt")
            .spawn()?;
        Ok(())
    }

    pub fn clear_new_client_fields(&mut self) {
        self.new_client_name.clear();
        self.new_client_ip.clear();
        self.new_client_password.clear();
    }

    pub fn load_selected_client(&mut self) {
        if let Some(index) = self.selected_client {
            if index < self.clients.len() {
                let client = &self.clients[index];
                self.new_client_name = client.name.clone();
                self.new_client_ip = client.ip.clone();
                self.new_client_password = client.password.clone();
            }
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.key_status {
            KeyStatus::FirstRun => self.show_first_run_dialog(ctx),
            KeyStatus::Missing => self.show_missing_key_dialog(ctx),
            KeyStatus::Incorrect => self.show_incorrect_key_dialog(ctx),
            KeyStatus::Submitted => {
                self.show_menu_bar(ctx);
                self.show_error_dialog(ctx);
                self.show_side_panel(ctx);
                self.show_central_panel(ctx);
            }
        }
    }
}

impl AppState {
    

    fn show_first_run_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Create Master Key")
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Please create a master key to encrypt your data.");
                let response = ui.add(egui::TextEdit::singleline(&mut self.master_key_input).password(true));
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.submit_master_key();
                }
                ui.label("Confirm Master Key");
                let response = ui.add(egui::TextEdit::singleline(&mut self.confirm_master_key_input).password(true));
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.submit_master_key();
                }
                if ui.button("Create").clicked() {
                    self.submit_master_key();
                }
            });
    }

    fn submit_master_key(&mut self) {
        let key = crate::encryption::generate_key_from_password(self.master_key_input.as_bytes());
        self.encryption_key = key;
        if let Ok(data) = fs::read("clients.json") {
            if let Ok(decrypted_data) = decrypt(&data, &self.encryption_key) {
                self.clients = serde_json::from_slice(&decrypted_data).unwrap_or_default();
                self.key_status = KeyStatus::Submitted;
            } else {
                self.key_status = KeyStatus::Incorrect;
            }
        } else {
            self.key_status = KeyStatus::Submitted; // No clients file, so the key is accepted
        }
    }

    fn show_missing_key_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Enter Master Key")
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Please enter the master key to decrypt your data.");
                let response = ui.add(egui::TextEdit::singleline(&mut self.master_key_input).password(true));
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.submit_master_key();
                }
                if ui.button("Submit").clicked() {
                    self.submit_master_key();
                }
            });
    }

    fn show_incorrect_key_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Incorrect Master Key")
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("The master key is incorrect. Please try again.");
                if ui.button("OK").clicked() {
                    self.key_status = KeyStatus::Missing;
                    self.master_key_input.clear();
                }
            });
    }

    fn show_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.mode = AppMode::Adding;
                        self.clear_new_client_fields();
                        ui.close_menu();
                    }
                    if ui.button("Edit").clicked() {
                        if self.selected_client.is_some() {
                            self.mode = AppMode::Editing;
                            self.load_selected_client();
                            ui.close_menu();
                        } else {
                            self.error_message = Some("Please select a target to edit.".to_string());
                        }
                    }
                    if ui.button("Remove").clicked() {
                        if self.selected_client.is_some() {
                            self.mode = AppMode::Removing;
                            ui.close_menu();
                        } else {
                            self.error_message = Some("Please select a target to remove.".to_string());
                        }
                    }
                    if ui.button("Import").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            if let Ok(data) = fs::read(path) {
                                let old_key = [0; KEY_SIZE]; // The old static key
                                if let Ok(decrypted_data) = decrypt(&data, &old_key) {
                                    if let Ok(old_clients) = serde_json::from_slice::<Vec<Client>>(&decrypted_data) {
                                        self.clients.extend(old_clients);
                                        if let Err(e) = self.save_clients() {
                                            self.error_message = Some(format!("Failed to save clients: {}", e));
                                        }
                                    }
                                }
                            }
                        }
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.mode = AppMode::About;
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn show_error_dialog(&mut self, ctx: &egui::Context) {
        if let Some(error_message) = self.error_message.clone() {
            egui::Window::new("Error")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(error_message);
                    if ui.button("OK").clicked() {
                        self.error_message = None;
                    }
                });
        }
    }

    fn show_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Clients");
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                let clients = self.clients.clone();
                for (index, client) in clients.iter().enumerate() {
                                        ui.horizontal(|ui| {
                        // Client name (selectable)
                        if ui.selectable_value(&mut self.selected_client, Some(index), egui::RichText::new(&client.name).heading()).clicked() {
                            self.mode = AppMode::Normal;
                        }
                        // Connect button after name
                        if ui.button("▶Connect").clicked() { 
                            if let Err(e) = self.connect_to_client(&client) {
                                self.error_message = Some(format!("Failed to connect: {}", e));
                            }
                        }

                        // // Right-aligned buttons
                        // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        //     if ui.button("🗑").clicked() {
                        //         self.selected_client = Some(index);
                        //         self.mode = AppMode::Removing;
                        //     }
                        //     if ui.button("✏").clicked() {
                        //         self.selected_client = Some(index);
                        //         self.mode = AppMode::Editing;
                        //         self.load_selected_client();
                        //     }
                        // });
                    });
                }
            });
        });
    }

    fn show_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            match self.mode {
                AppMode::Adding => self.show_add_client_form(ui),
                AppMode::Editing => self.show_edit_client_form(ui),
                AppMode::Removing => self.show_remove_client_form(ui),
                AppMode::About => self.show_about_info(ui),
                AppMode::Normal => {
                    ui.heading("Remote Desktop Manager");
                    ui.label("Select a client from the list to connect, or use the File menu to manage clients.");
                }
            }
        });
    }

    fn show_add_client_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Add New Client");
        self.show_client_form(ui);
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                self.clients.push(Client {
                    name: self.new_client_name.clone(),
                    ip: self.new_client_ip.clone(),
                    password: self.new_client_password.clone(),
                });
                self.clear_new_client_fields();
                if let Err(e) = self.save_clients() {
                    self.error_message = Some(format!("Failed to save clients: {}", e));
                } else {
                    self.mode = AppMode::Normal;
                }
            }
            if ui.button("Cancel").clicked() {
                self.clear_new_client_fields();
                self.mode = AppMode::Normal;
            }
        });
    }

    fn show_edit_client_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Edit Client");
        self.show_client_form(ui);
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                if let Some(index) = self.selected_client {
                    self.clients[index] = Client {
                        name: self.new_client_name.clone(),
                        ip: self.new_client_ip.clone(),
                        password: self.new_client_password.clone(),
                    };
                    self.clear_new_client_fields();
                    if let Err(e) = self.save_clients() {
                        self.error_message = Some(format!("Failed to save clients: {}", e));
                    } else {
                        self.mode = AppMode::Normal;
                    }
                }
            }
            if ui.button("Cancel").clicked() {
                self.clear_new_client_fields();
                self.mode = AppMode::Normal;
            }
        });
    }

    fn show_remove_client_form(&mut self, ui: &mut egui::Ui) {
        if let Some(index) = self.selected_client {
            ui.heading(format!("Remove Client: {}", self.clients[index].name));
            ui.label("Are you sure you want to remove this client?");
            ui.horizontal(|ui| {
                if ui.button("Confirm").clicked() {
                    self.clients.remove(index);
                    self.selected_client = None;
                    self.clear_new_client_fields();
                    if let Err(e) = self.save_clients() {
                        self.error_message = Some(format!("Failed to save clients: {}", e));
                    } else {
                        self.mode = AppMode::Normal;
                    }
                }
                if ui.button("Cancel").clicked() {
                    self.mode = AppMode::Normal;
                }
            });
        }
    }

    fn show_about_info(&mut self, ui: &mut egui::Ui) {
        ui.heading("About");
        ui.label("Powered By Jerry Yu");
        if ui.button("Back").clicked() {
            self.mode = AppMode::Normal;
        }
    }

    fn show_client_form(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.new_client_name);
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.label("IP:");
            ui.text_edit_singleline(&mut self.new_client_ip);
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.label("Password:");
            if self.show_password {
                ui.text_edit_singleline(&mut self.new_client_password);
            } else {
                let masked_password: String = "*".repeat(self.new_client_password.len());
                ui.label(masked_password);
            }
            if ui.button("👁").clicked() {
                self.show_password = !self.show_password;
            }
        });
        ui.add_space(10.0);
    }
}
