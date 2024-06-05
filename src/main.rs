use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

#[derive(Serialize, Deserialize, Clone)]
struct Client {
    name: String,
    ip: String,
    password: String,
}

enum AppMode {
    Normal,
    Adding,
    Editing,
    Removing,
    About,
}

struct AppState {
    clients: Vec<Client>,
    selected_client: Option<usize>,
    new_client_name: String,
    new_client_ip: String,
    new_client_password: String,
    mode: AppMode,
    show_password: bool,
    error_message: Option<String>,
}

impl AppState {
    fn new() -> Self {
        let clients = if let Ok(data) = fs::read_to_string("clients.json") {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };
        Self {
            clients,
            selected_client: None,
            new_client_name: String::new(),
            new_client_ip: String::new(),
            new_client_password: String::new(),
            mode: AppMode::Normal,
            show_password: false,
            error_message: None,
        }
    }

    fn save_clients(&self) {
        let data = serde_json::to_string(&self.clients).unwrap();
        fs::write("clients.json", data).unwrap();
    }

    fn connect_to_client(&self, client: &Client) {
        Command::new("mstsc")
            .arg("/v")
            .arg(&client.ip)
            .arg("/prompt")
            .spawn()
            .expect("Failed to launch Remote Desktop");
    }

    fn clear_new_client_fields(&mut self) {
        self.new_client_name.clear();
        self.new_client_ip.clear();
        self.new_client_password.clear();
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
                ui.menu_button("About", |ui| {
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
                            self.connect_to_client(client);
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
                        self.save_clients();
                        self.mode = AppMode::Normal;
                    }

                    if ui.button("Cancel").clicked() {
                        self.clear_new_client_fields();
                        self.mode = AppMode::Normal;
                    }
                }
                AppMode::Editing => {
                    if let Some(index) = self.selected_client {
                        if index < self.clients.len() {
                            let client = &self.clients[index];
                            self.new_client_name = client.name.clone();
                            self.new_client_ip = client.ip.clone();
                            self.new_client_password = client.password.clone();

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
                                self.save_clients();
                                self.mode = AppMode::Normal;
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
                                self.save_clients();
                                self.mode = AppMode::Normal;
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

fn main() {
    let native_options = eframe::NativeOptions {
        window_builder: Some(Box::new(|builder| {
            builder
                .with_inner_size(eframe::epaint::Vec2::new(600.0, 400.0))
                .with_title("Remote Desktop Manager")
        })),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Remote Desktop Manager",
        native_options,
        Box::new(|_cc| Box::new(AppState::new())),
    );
}
