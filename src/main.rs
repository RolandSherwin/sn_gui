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
        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Top, "top").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                if ui.button(RichText::new("Network").heading()).clicked() {
                    self.state = SafeGuiState::Network;
                }
                if ui.button(RichText::new("Node").heading()).clicked() {
                    self.state = SafeGuiState::Node;
                }
            });
            ui.add_space(10.0);
        });
        self.node_runner.current_network_name = self.network.current_network_name.clone();
        match &mut self.state {
            SafeGuiState::Network => self.network.ui(ctx.clone()),
            SafeGuiState::Node => self.node_runner.ui(ctx.clone()),
        }
    }
}
