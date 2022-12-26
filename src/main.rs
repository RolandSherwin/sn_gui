mod files;
mod network;
mod node;
use crate::{files::FilesView, network::Network, node::NodeRunner};

use eframe::{
    egui::{self, Layout, RichText},
    emath::Align,
};
use eyre::Result;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
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
    Files,
    Network,
    Node,
}

impl Default for SafeGuiState {
    fn default() -> Self {
        Self::Network
    }
}

#[derive(Default)]
struct SafeGui {
    network: Network,
    node_runner: NodeRunner,
    files_view: FilesView,
    state: SafeGuiState,
    status: Option<RichText>,
    stauts_reciever: Option<mpsc::Receiver<RichText>>,
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
                if ui.button(RichText::new("Files").heading()).clicked() {
                    self.state = SafeGuiState::Files;
                }
                if ui.button(RichText::new("Node").heading()).clicked() {
                    self.state = SafeGuiState::Node;
                }
            });
            ui.add_space(10.0);
        });

        if let Some(status_sender) = self.footer(ctx) {
            self.network.status_sender = Some(status_sender.clone());
            self.node_runner.status_sender = Some(status_sender);
        }
        self.node_runner.current_network_name = self.network.current_network_name.clone();
        match &mut self.state {
            SafeGuiState::Network => self.network.ui(ctx.clone()),
            SafeGuiState::Node => self.node_runner.ui(ctx.clone()),
            SafeGuiState::Files => self.files_view.ui(ctx.clone()),
        }
    }
}

impl SafeGui {
    // Render a footer and store/display the status that we get from the senders
    pub fn footer(&mut self, ctx: &egui::Context) -> Option<mpsc::Sender<RichText>> {
        let tx = if self.stauts_reciever.is_none() {
            let (tx, rx) = mpsc::channel(10000);
            self.stauts_reciever = Some(rx);
            Some(tx)
        } else {
            None
        };

        if let Some(rx) = &mut self.stauts_reciever {
            if let Ok(new_status) = rx.try_recv() {
                self.status = Some(new_status);
            }
        }

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                if let Some(text) = &self.status {
                    ui.label(text.clone());
                }
            })
        });

        tx
    }
}
