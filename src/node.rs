use eframe::{
    egui::{self, RichText},
    epaint::Color32,
};
use eyre::{eyre, Result};
use std::process::{Child, Command, Stdio};

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
    error: Option<String>,
}

impl NodeRunner {
    pub fn ui(&mut self, ctx: egui::Context) {
        self.error = None;
        egui::CentralPanel::default().show(&ctx, |ui| {
            match &mut self.node_state {
                NodeState::Idle => {
                    if ui.button(RichText::new("Start node").heading()).clicked() {
                        if self.current_network_name.is_some() {
                            match Self::run_node() {
                                Ok(handle) => self.node_state = NodeState::Running(handle),
                                Err(err) => self.error = Some(err.to_string()),
                            }
                        } else {
                            self.error = Some("No default network set".to_string());
                        }
                    }
                }
                NodeState::Running(handle) => {
                    if ui.button(RichText::new("Stop node").heading()).clicked() {
                        if handle.kill().is_err() {
                            self.error = Some("Failed to kill node".to_string());
                        };
                        self.node_state = NodeState::Idle
                    }
                }
            }

            if let Some(error) = &self.error {
                ui.colored_label(Color32::RED, error);
            }
        });
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
