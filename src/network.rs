use eframe::{
    egui::{self, RichText, ScrollArea},
    epaint::Color32,
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
                    egui::Window::new("Add Network").show(&ctx, |ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            let name_lable = ui.label("Network name: ");
                            ui.text_edit_singleline(&mut self.add_network.name)
                                .labelled_by(name_lable.id)
                                .request_focus();
                        });
                        if self.add_network.name_error {
                            ui.colored_label(egui::Color32::RED, "Name cannot be empty");
                        }

                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            let path_lable = ui.label("Network path: ");
                            ui.text_edit_singleline(&mut self.add_network.path)
                                .labelled_by(path_lable.id);
                            ui.add_space(1.0);

                            if ui.button("Open file…").clicked() {
                                // file picker dependencies https://docs.rs/rfd/latest/rfd/#linux--bsd-backends
                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                    self.add_network.path = path.display().to_string();
                                }
                            }
                        });
                        if self.add_network.path_error {
                            ui.colored_label(egui::Color32::RED, "Path cannot be empty");
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
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
                                    }
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
                        });
                    });
                }
                OpenWindows::Switch => {
                    egui::Window::new("Switch Network")
                        // .pivot(egui::Align2::LEFT_CENTER)
                        // .fixed_pos(egui::pos2(pos.x + 000.0, pos.y + 0.0))
                        .show(&ctx, |ui| {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                let switch_to_lable = ui.label("Switch to: ");
                                ui.text_edit_singleline(&mut self.switch_network.name)
                                    .labelled_by(switch_to_lable.id)
                                    .request_focus();
                            });
                            if self.switch_network.name_error {
                                ui.colored_label(egui::Color32::RED, "Name cannot be empty");
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
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
                                        }
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
                            });
                        });
                }
                OpenWindows::Remove => {
                    egui::Window::new("Remove Network")
                        // .pivot(egui::Align2::LEFT_CENTER)
                        // .fixed_pos(egui::pos2(pos.x + 000.0, pos.y + 0.0))
                        .show(&ctx, |ui| {
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                                let remove_network_lable = ui.label("Remove network: ");
                                ui.text_edit_singleline(&mut self.remove_network.name)
                                    .labelled_by(remove_network_lable.id)
                                    .request_focus();
                            });
                            if self.remove_network.name_error {
                                ui.colored_label(egui::Color32::RED, "Name cannot be empty");
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
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
                                        }
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
                            });
                        });
                }
            }
        }

        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
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
                        self.open_window = Some(OpenWindows::Add)
                    } else {
                        self.open_window = None;
                    }
                };
            });

            match &self.networks {
                Some(networks) => {
                    ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("network_grid")
                            .striped(true)
                            .show(ui, |ui| {
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

    // send status to the footer
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

    fn get_networks(&mut self) {
        match Self::get_networks_cmd() {
            Ok(networks) => self.networks = Some(networks),
            Err(err) => self.send_status(RichText::new(err.to_string()).color(Color32::RED)),
        }
    }

    fn get_networks_cmd() -> Result<Vec<NetworkPrinter>> {
        let cmd = Command::new("safe")
            .args(vec!["networks", "--json"])
            .output()?;
        if !cmd.status.success() {
            return Err(eyre!("Failed to fetch networks"));
        }
        let networks = String::from_utf8(cmd.stdout)?;
        Ok(serde_json::from_str(networks.as_str())?)
    }

    fn add_networks_cmd(name: &str, path: &str) -> Result<()> {
        let args_add_network = vec!["networks", "add", name, path];
        if !Command::new("safe")
            .args(args_add_network)
            .output()?
            .status
            .success()
        {
            return Err(eyre!("Failed to add network"));
        }

        Ok(())
    }

    fn switch_networks_cmd(name: &str) -> Result<()> {
        let args_switch_network = vec!["networks", "switch", name];

        if !Command::new("safe")
            .args(args_switch_network)
            .output()?
            .status
            .success()
        {
            return Err(eyre!("Failed to switch network"));
        }
        Ok(())
    }

    fn remove_network_cmd(name: &str) -> Result<()> {
        let args_remove_network = vec!["networks", "remove", name];

        if !Command::new("safe")
            .args(args_remove_network)
            .output()?
            .status
            .success()
        {
            return Err(eyre!("Failed to remove network"));
        }
        Ok(())
    }
}
