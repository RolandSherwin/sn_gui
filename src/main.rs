mod network;
mod node;
use crate::{network::Network, node::NodeRunner};

use eframe::{
    egui::{self, Layout, RichText},
    emath::Align,
};
use eyre::Result;

fn main() -> Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(900.0, 400.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Safe Network",
        options,
        Box::new(|_cc| Box::<SafeGui>::default()),
    );
    Ok(())
}

enum SafeGuiState {
    Network,
    Node,
}
struct SafeGui {
    network: Network,
    node_runner: NodeRunner,
    state: SafeGuiState,
}

impl Default for SafeGui {
    fn default() -> Self {
        Self {
            network: Network::default(),
            node_runner: NodeRunner::default(),
            state: SafeGuiState::Network,
        }
    }
}

impl eframe::App for SafeGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.group(|ui| {
                    if ui.button(RichText::new("Network").heading()).clicked() {
                        self.state = SafeGuiState::Network;
                    }
                    if ui.button(RichText::new("Node").heading()).clicked() {
                        self.state = SafeGuiState::Node;
                    }
                });
            });
            ui.add_space(15.0);
            match &mut self.state {
                SafeGuiState::Network => self.network.ui(ui),
                SafeGuiState::Node => self.node_runner.ui(ui),
            }
        });
    }
}
