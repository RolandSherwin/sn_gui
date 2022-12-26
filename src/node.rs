use eframe::{
    egui::{self, RichText},
    epaint::Color32,
};
use eyre::Result;
use std::process::{Child, Command, Stdio};
use tokio::sync::mpsc;

enum NodeState {
    Running(Child),
    Idle,
}

impl Default for NodeState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Default)]
pub struct NodeRunner {
    pub current_network_name: Option<String>,
    node_state: NodeState,
    pub status_sender: Option<mpsc::Sender<RichText>>,
}

impl NodeRunner {
    pub fn ui(&mut self, ctx: egui::Context) {
        egui::CentralPanel::default().show(&ctx, |ui| match &mut self.node_state {
            NodeState::Idle => {
                if ui.button(RichText::new("Start node").heading()).clicked() {
                    if self.current_network_name.is_some() {
                        match Self::run_node() {
                            Ok(handle) => {
                                self.send_status(RichText::new("Node is running!".to_string()));
                                self.node_state = NodeState::Running(handle)
                            }
                            Err(err) => self.send_status(
                                RichText::new(format!("Error: {err}")).color(Color32::RED),
                            ),
                        }
                    } else {
                        self.send_status(
                            RichText::new("Error: No default network set").color(Color32::RED),
                        );
                    }
                }
            }
            NodeState::Running(handle) => {
                if ui.button(RichText::new("Stop node").heading()).clicked() {
                    if handle.kill().is_err() {
                        self.send_status(
                            RichText::new("Error: Failed to kill node").color(Color32::RED),
                        );
                    };
                    self.send_status(RichText::new("Node has been stopped!".to_string()));
                    self.node_state = NodeState::Idle
                }
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

    fn run_node() -> Result<Child> {
        let args_run_node = vec!["node", "join", "--network-name", "main"];
        // calling .kill() does not kill the child process unless Stdio::piped() is provided
        // Now the child process exists with a panic, "failed printing to stdout"
        let handle = Command::new("safe")
            .args(args_run_node)
            .stdout(Stdio::piped())
            .spawn()?;
        Ok(handle)
    }
}
