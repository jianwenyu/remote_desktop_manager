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
            KeyStatus::FirstRun => {
                egui::Window::new("Create Master Key")
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label("Please create a master key to encrypt your data.");
                        ui.add(egui::TextEdit::singleline(&mut self.master_key_input).password(true));
                        ui.label("Confirm Master Key");
                        ui.add(egui::TextEdit::singleline(&mut self.confirm_master_key_input).password(true));
                        if ui.button("Create").clicked() {
                            if self.master_key_input == self.confirm_master_key_input {
                                let key = crate::encryption::generate_key_from_password(self.master_key_input.as_bytes());
                                self.encryption_key = key;
                                if let Err(e) = self.save_clients() {
                                    self.error_message = Some(format!("Failed to save clients: {}", e));
                                } else {
                                    self.key_status = KeyStatus::Submitted;
                                }
                            } else {
                                self.error_message = Some("Master keys do not match.".to_string());
                            }
                        }
                    });
            }
            KeyStatus::Missing => {
                egui::Window::new("Enter Master Key")
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label("Please enter the master key to decrypt your data.");
                        ui.add(egui::TextEdit::singleline(&mut self.master_key_input).password(true));
                        if ui.button("Submit").clicked() {
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
                    });
            }
            KeyStatus::Incorrect => {
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
            KeyStatus::Submitted => {
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

                egui::CentralPanel::default().show(ctx, |ui| {
                    let clients: Vec<(usize, String)> = self
                        .clients
                        .iter()
                        .enumerate()
                        .map(|(index, client)| (index, client.name.clone()))
                        .collect();

                    for (index, client_name) in clients {
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.selected_client, Some(index), egui::RichText::new(client_name).heading());
                            if ui.button("Connect").clicked() {
                                if let Some(client) = self.clients.get(index) {
                                    if let Err(e) = self.connect_to_client(client) {
                                        self.error_message = Some(format!("Failed to connect to client: {}", e));
                                    }
                                }
                            }
                        });
                    }

                    ui.separator();

                    match self.mode {
                        AppMode::Adding => {
                            ui.label("Add New Client:");

                            ui.horizontal(|ui| {
                                ui.label("Name:");
                                ui.text_edit_singleline(&mut self.new_client_name);
                            });
                            ui.horizontal(|ui| {
                                ui.label("IP:");
                                ui.text_edit_singleline(&mut self.new_client_ip);
                            });
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
                        }
                        AppMode::Editing => {
                            if let Some(index) = self.selected_client {
                                if index < self.clients.len() {
                                    ui.label("Edit Client:");

                                    ui.horizontal(|ui| {
                                        ui.label("Name:");
                                        ui.text_edit_singleline(&mut self.new_client_name);
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("IP:");
                                        ui.text_edit_singleline(&mut self.new_client_ip);
                                    });
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

                                    if ui.button("Save").clicked() {
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

                                    if ui.button("Cancel").clicked() {
                                        self.clear_new_client_fields();
                                        self.mode = AppMode::Normal;
                                    }
                                }
                            }
                        }
                        AppMode::Removing => {
                            if let Some(index) = self.selected_client {
                                if index < self.clients.len() {
                                    ui.label(format!("Remove Client: {}", self.clients[index].name));

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
                                        self.clear_new_client_fields();
                                        self.mode = AppMode::Normal;
                                    }
                                }
                            }
                        }
                        AppMode::About => {
                            ui.label("Powered By Jerry Yu");
                            if ui.button("Back").clicked() {
                                self.mode = AppMode::Normal;
                            }
                        }
                        AppMode::Normal => {
                            // Do nothing
                        }
                    }
                });
            }
        }
    }
}