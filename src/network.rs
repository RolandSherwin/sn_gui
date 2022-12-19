use eframe::{
    egui::{self, RichText, Ui},
    epaint::Color32,
};
use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Default)]
pub struct Network {
    networks: Option<Vec<NetworkPrinter>>,
    error: Option<String>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct NetworkPrinter {
    current: bool,
    name: String,
    genesis_key: String,
    network_info: String,
}

impl Network {
    pub fn ui(&mut self, ui: &mut Ui) {
        self.error = None;
        if self.networks.is_none() {
            match Self::get_networks() {
                Ok(networks) => self.networks = Some(networks),
                Err(err) => self.error = Some(err.to_string()),
            }
        }

        match &self.networks {
            Some(networks) => {
                egui::Grid::new("network_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Current").strong());
                        ui.label(RichText::new("Network Name").strong());
                        ui.label(RichText::new("Genesis Key").strong());
                        ui.label(RichText::new("Network Contact Info").strong());
                        ui.end_row();
                        for network in networks {
                            let current = if network.current { "✅" } else { "" };
                            ui.label(current);
                            ui.label(&network.name);
                            ui.label(&network.genesis_key);
                            ui.label(&network.network_info);
                            ui.end_row();
                        }
                    });
            }
            None => {}
        }
        if let Some(error) = &self.error {
            ui.colored_label(Color32::RED, error);
        }
    }

    fn get_networks() -> Result<Vec<NetworkPrinter>> {
        let cmd = Command::new("safe")
            .args(vec!["networks", "--json"])
            .output()?;
        if !cmd.status.success() {
            return Err(eyre!("Failed to fetch networks"));
        }
        let networks = String::from_utf8(cmd.stdout)?;
        Ok(serde_json::from_str(networks.as_str())?)
    }
}
