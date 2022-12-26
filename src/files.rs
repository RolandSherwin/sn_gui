use eframe::{
    egui::{self, RichText},
    epaint::Color32,
};
use eyre::{eyre, Result};
use std::process::Command;
use tokio::sync::mpsc;

#[derive(Default)]
pub struct FilesView {
    files_state: FilesState,
    put_state: PutState,
    get_state: GetState,
    cmd_output: String,
    pub status_sender: Option<mpsc::Sender<RichText>>,
}

enum FilesState {
    Put,
    Get,
}

#[derive(Default)]
struct PutState {
    path: String,
    path_error: bool,
}

struct GetState {
    url: String,
    url_error: bool,
    dst: String,
}

impl Default for GetState {
    fn default() -> Self {
        Self {
            url: Default::default(),
            url_error: Default::default(),
            dst: ".".to_string(),
        }
    }
}

impl Default for FilesState {
    fn default() -> Self {
        Self::Put
    }
}

impl FilesView {
    pub fn ui(&mut self, ctx: egui::Context) {
        egui::SidePanel::new(egui::panel::Side::Left, "files_panel").show(&ctx, |ui| {
            if ui.button(RichText::new("Put").heading()).clicked() {
                self.files_state = FilesState::Put;
            };
            ui.add_space(10.0);
            if ui.button(RichText::new("Get").heading()).clicked() {
                self.files_state = FilesState::Get;
            };
        });
        egui::CentralPanel::default().show(&ctx, |ui| {
            match self.files_state {
                FilesState::Put => {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        let path_lable = ui.label("Put file location: ");
                        ui.text_edit_singleline(&mut self.put_state.path)
                            .labelled_by(path_lable.id);
                        ui.add_space(1.0);

                        if ui.button("Open file…").clicked() {
                            // file picker dependencies https://docs.rs/rfd/latest/rfd/#linux--bsd-backends
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.put_state.path = path.display().to_string();
                            }
                        }
                    });
                    if self.put_state.path_error {
                        ui.colored_label(egui::Color32::RED, "Path cannot be empty");
                    }
                    if ui.button("Put!").clicked() {
                        self.put_state.path_error = self.put_state.path == String::default();
                        if !self.put_state.path_error {
                            match Self::put_files(&self.put_state.path) {
                                Ok(cmd_output) => {
                                    self.send_status(RichText::new(
                                        "Successfully added file to the network".to_string(),
                                    ));
                                    self.put_state.path = String::default();
                                    self.cmd_output = cmd_output;
                                }
                                Err(err) => self.send_status(
                                    RichText::new(err.to_string()).color(Color32::RED),
                                ),
                            }
                        }
                    }
                }
                FilesState::Get => {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        let url_lable = ui.label("Safe url: ");
                        ui.text_edit_singleline(&mut self.get_state.url)
                            .labelled_by(url_lable.id);
                        ui.add_space(1.0);
                    });
                    if self.get_state.url_error {
                        ui.colored_label(egui::Color32::RED, "Safe URL cannot be empty");
                    }

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        let dst_lable = ui.label("File dest: ");
                        ui.text_edit_singleline(&mut self.get_state.dst)
                            .labelled_by(dst_lable.id);
                        if ui.button("Select folder…").clicked() {
                            // file picker dependencies https://docs.rs/rfd/latest/rfd/#linux--bsd-backends
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.get_state.dst = path.display().to_string();
                            }
                        }
                        ui.add_space(1.0);
                    });

                    if ui.button("Get!").clicked() {
                        self.get_state.url_error = self.get_state.url == String::default();
                        if !self.get_state.url_error {
                            match Self::get_files(&self.get_state.url, &self.get_state.dst) {
                                Ok(_) => {
                                    self.send_status(RichText::new(
                                        "Successfully retrieved files from the network".to_string(),
                                    ));
                                    self.get_state.url = String::default();
                                }
                                Err(err) => self.send_status(
                                    RichText::new(err.to_string()).color(Color32::RED),
                                ),
                            }
                        }
                    }
                }
            }

            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::BottomUp),
                |ui| {
                    ui.text_edit_multiline(&mut self.cmd_output);
                },
            )
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

    fn put_files(path: &str) -> Result<String> {
        let args_put_files = vec!["networks", "files", "put", path];
        let result = Command::new("safe").args(args_put_files).output()?;

        if !result.status.success() {
            return Err(eyre!("Error: failed to put file"));
        }

        Ok(String::from_utf8(result.stdout)?)
    }

    fn get_files(url: &str, dst: &str) -> Result<()> {
        let args_put_files = vec!["networks", "files", "get", url, dst];
        let result = Command::new("safe").args(args_put_files).output()?;

        if !result.status.success() {
            return Err(eyre!("Error: failed to put file"));
        }

        Ok(())
    }
}
