use eframe::egui::{
    self, Align, CentralPanel, Color32, Grid, Layout, RichText, ScrollArea, Ui, Window,
};
use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tokio::sync::mpsc;

#[derive(Default)]
pub struct Network {
    open_window: Option<OpenWindows>,
    add_network: AddNetwork,
    switch_network: SwitchNetwork,
    remove_network: RemoveNetwork,
    networks: Option<Vec<NetworkPrinter>>,
    pub current_network_name: Option<String>,
    pub status_sender: Option<mpsc::Sender<RichText>>,
}

#[derive(Default)]
struct AddNetwork {
    initial_focus_done: bool,
    name: String,
    name_error: bool,
    path: String,
    path_error: bool,
}

#[derive(Default)]
struct SwitchNetwork {
    name: String,
    name_error: bool,
}

#[derive(Default)]
struct RemoveNetwork {
    name: String,
    name_error: bool,
}

enum OpenWindows {
    Add,
    Switch,
    Remove,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct NetworkPrinter {
    current: bool,
    name: String,
    genesis_key: String,
    network_info: String,
}

impl Network {
    pub fn ui(&mut self, ctx: egui::Context) {
        if self.networks.is_none() {
            self.get_networks();
        }

        // let pos = ctx.input().screen_rect().size();
        if let Some(open_window) = &self.open_window {
            match open_window {
                OpenWindows::Add => {
                    Window::new("Add Network").show(&ctx, |ui| {
                        self.add_network.ui_name_row(ui);
                        self.add_network.ui_path_row(ui);

                        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                            if ui.button("Close").clicked() {
                                self.open_window = None;
                            }
                            if ui.button("Add").clicked() {
                                self.add_network.path_error =
                                    self.add_network.path == String::default();
                                self.add_network.name_error =
                                    self.add_network.name == String::default();

                                if !self.add_network.path_error && !self.add_network.name_error {
                                    if let Err(err) = Self::add_networks_cmd(
                                        &self.add_network.name,
                                        &self.add_network.path,
                                    ) {
                                        self.send_status(
                                            RichText::new(err.to_string()).color(Color32::RED),
                                        );
                                    } else {
                                        // sucessfully added a network
                                        self.send_status(RichText::new(format!(
                                            "Success: Added a network with name {}",
                                            self.add_network.name
                                        )));
                                        self.add_network.path = String::default();
                                        self.add_network.path_error = false;
                                        self.add_network.name = String::default();
                                        self.add_network.name_error = false;
                                        self.open_window = None;
                                        self.get_networks();
                                    }
                                }
                            }
                        });
                    });
                }
                OpenWindows::Switch => {
                    Window::new("Switch Network")
                        // .pivot(egui::Align2::LEFT_CENTER)
                        // .fixed_pos(egui::pos2(pos.x + 000.0, pos.y + 0.0))
                        .show(&ctx, |ui| {
                            self.switch_network.ui_name_row(ui);

                            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                                if ui.button("Close").clicked() {
                                    self.open_window = None;
                                }
                                if ui.button("Switch").clicked() {
                                    self.switch_network.name_error =
                                        self.switch_network.name == String::default();
                                    if !self.switch_network.name_error {
                                        if let Err(err) =
                                            Self::switch_networks_cmd(&self.switch_network.name)
                                        {
                                            self.send_status(
                                                RichText::new(err.to_string()).color(Color32::RED),
                                            );
                                        } else {
                                            // sucessfully swtiched newtork
                                            self.send_status(RichText::new(format!(
                                                "Success: Switched to network: {}",
                                                self.switch_network.name
                                            )));
                                            self.switch_network.name = String::default();
                                            self.open_window = None;
                                            self.get_networks();
                                        }
                                    }
                                }
                            });
                        });
                }
                OpenWindows::Remove => {
                    Window::new("Remove Network")
                        // .pivot(egui::Align2::LEFT_CENTER)
                        // .fixed_pos(egui::pos2(pos.x + 000.0, pos.y + 0.0))
                        .show(&ctx, |ui| {
                            self.remove_network.ui_name_row(ui);

                            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                                if ui.button("Close").clicked() {
                                    self.open_window = None;
                                }
                                if ui.button("Remove").clicked() {
                                    self.remove_network.name_error =
                                        self.remove_network.name == String::default();
                                    if !self.remove_network.name_error {
                                        if let Err(err) =
                                            Self::remove_network_cmd(&self.remove_network.name)
                                        {
                                            self.send_status(
                                                RichText::new(err.to_string()).color(Color32::RED),
                                            );
                                        } else {
                                            // sucessfully removed network
                                            self.send_status(RichText::new(format!(
                                                "Success: Removed network: {}",
                                                self.remove_network.name
                                            )));
                                            self.remove_network.name = String::default();
                                            self.open_window = None;
                                            self.get_networks();
                                        }
                                    }
                                }
                            });
                        });
                }
            }
        }

        CentralPanel::default().show(&ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                if ui.button("⟳").clicked() {
                    self.get_networks();
                };
                if ui.button("Remove").clicked() {
                    if self.open_window.is_none() {
                        self.open_window = Some(OpenWindows::Remove)
                    } else {
                        self.open_window = None;
                    }
                };
                if ui.button("Switch").clicked() {
                    // toggle window
                    if self.open_window.is_none() {
                        self.open_window = Some(OpenWindows::Switch);
                    } else {
                        self.open_window = None;
                    }
                };
                if ui.button("Add").clicked() {
                    if self.open_window.is_none() {
                        // to set focus to the first text input
                        self.add_network.initial_focus_done = false;
                        self.open_window = Some(OpenWindows::Add)
                    } else {
                        self.open_window = None;
                    }
                };
            });

            match &self.networks {
                Some(networks) => {
                    ScrollArea::vertical().show(ui, |ui| {
                        Grid::new("network_grid").striped(true).show(ui, |ui| {
                            ui.label(RichText::new("Current").strong());
                            ui.label(RichText::new("Network Name").strong());
                            ui.label(RichText::new("Genesis Key").strong());
                            ui.label(RichText::new("Network Contact Info").strong());
                            ui.end_row();
                            self.current_network_name = None;
                            for network in networks {
                                // used to start node
                                if network.current {
                                    self.current_network_name = Some(network.name.clone());
                                }
                                let current = if network.current { "✅" } else { "" };
                                ui.label(current);
                                ui.label(&network.name);
                                ui.label(&network.genesis_key);
                                ui.label(&network.network_info);
                                ui.end_row();
                            }
                        });
                    });
                }
                None => {}
            }
        });
    }

    // Send status to the footer
    fn send_status(&self, text: RichText) {
        let sender = self.status_sender.clone();
        if let Some(sender) = sender {
            tokio::spawn(async move {
                if sender.send(text).await.is_err() {
                    log::error!("Failed to send status");
                };
            });
        }
    }

    // Get the current networks and store it inside self
    fn get_networks(&mut self) {
        match Self::get_networks_cmd() {
            Ok(networks) => self.networks = Some(networks),
            Err(err) => self.send_status(RichText::new(err.to_string()).color(Color32::RED)),
        }
    }

    // Execute safe Command to get the networks
    fn get_networks_cmd() -> Result<Vec<NetworkPrinter>> {
        let cmd = Command::new("safe")
            .args(vec!["networks", "--json"])
            .output()?;
        if !cmd.status.success() {
            return Err(eyre!("Error: failed to fetch networks"));
        }
        let networks = String::from_utf8(cmd.stdout)?;
        Ok(serde_json::from_str(networks.as_str())?)
    }

    // Execute safe Command to add a network
    fn add_networks_cmd(name: &str, path: &str) -> Result<()> {
        let args_add_network = vec!["networks", "add", name, path];
        if !Command::new("safe")
            .args(args_add_network)
            .output()?
            .status
            .success()
        {
            return Err(eyre!("Error: failed to add network"));
        }

        Ok(())
    }

    // Execute safe Command to switch network
    fn switch_networks_cmd(name: &str) -> Result<()> {
        let args_switch_network = vec!["networks", "switch", name];

        let result = Command::new("safe").args(args_switch_network).output()?;

        if !result.status.success() {
            let stderr = String::from_utf8(result.stderr)?;
            if stderr.contains("with name") {
                return Err(eyre!(
                    "Error: failed to switch network. No network with name {name} found!"
                ));
            }
            return Err(eyre!("Error: failed to switch network"));
        }
        Ok(())
    }

    // Execute safe Command to remove a network
    fn remove_network_cmd(name: &str) -> Result<()> {
        let args_remove_network = vec!["networks", "remove", name];

        let result = Command::new("safe").args(args_remove_network).output()?;
        let stdout = String::from_utf8(result.stdout)?;
        if stdout.contains("with name") {
            return Err(eyre!(
                "Error: failed to remove network. No network with name {name} found!"
            ));
        }

        if !result.status.success() {
            return Err(eyre!("Error: failed to remove network"));
        }
        Ok(())
    }
}

impl AddNetwork {
    // UI for network name lable and text input
    fn ui_name_row(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            let name_lable = ui.label("Network name: ");
            let name = ui
                .text_edit_singleline(&mut self.name)
                .labelled_by(name_lable.id);
            if !self.initial_focus_done {
                name.request_focus();
                self.initial_focus_done = true
            }
        });
        if self.name_error {
            ui.colored_label(Color32::RED, "Name cannot be empty");
        }
    }

    // UI for network path lable, text input and file browser
    fn ui_path_row(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            let path_lable = ui.label("Network path: ");
            ui.text_edit_singleline(&mut self.path)
                .labelled_by(path_lable.id);
            ui.add_space(1.0);

            if ui.button("Open file…").clicked() {
                // file picker dependencies https://docs.rs/rfd/latest/rfd/#linux--bsd-backends
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.path = path.display().to_string();
                }
            }
        });
        if self.path_error {
            ui.colored_label(Color32::RED, "Path cannot be empty");
        }
    }
}

impl SwitchNetwork {
    fn ui_name_row(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            let switch_to_lable = ui.label("Switch to: ");
            ui.text_edit_singleline(&mut self.name)
                .labelled_by(switch_to_lable.id)
                .request_focus();
        });
        if self.name_error {
            ui.colored_label(Color32::RED, "Name cannot be empty");
        }
    }
}

impl RemoveNetwork {
    fn ui_name_row(&mut self, ui: &mut Ui) {
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            let remove_network_lable = ui.label("Remove network: ");
            ui.text_edit_singleline(&mut self.name)
                .labelled_by(remove_network_lable.id)
                .request_focus();
        });
        if self.name_error {
            ui.colored_label(Color32::RED, "Name cannot be empty");
        }
    }
}
