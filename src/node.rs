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

pub struct NodeRunner {
    error: Option<String>,
    node_state: NodeState,
}

impl Default for NodeRunner {
    fn default() -> Self {
        Self {
            error: None,
            node_state: NodeState::Idle,
        }
    }
}

impl NodeRunner {
    pub fn ui(&mut self, ctx: egui::Context) {
        self.error = None;
        egui::CentralPanel::default().show(&ctx, |ui| {
            match &mut self.node_state {
                NodeState::Idle => {
                    if ui.button(RichText::new("Start node").size(30.0)).clicked() {
                        if let Err(err) = Self::setup_network() {
                            self.error = Some(err.to_string());
                        } else {
                            match Self::run_node() {
                                Ok(handle) => self.node_state = NodeState::Running(handle),
                                Err(err) => self.error = Some(err.to_string()),
                            }
                        }
                    }
                }
                NodeState::Running(handle) => {
                    if ui.button(RichText::new("Stop node").size(30.0)).clicked() {
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
    fn setup_network() -> Result<()> {
        let args_add_network = vec![
            "networks",
            "add",
            "main",
            "https://sn-node.s3.eu-west-2.amazonaws.com/testnet_tool/main2/network-contacts",
        ];
        let args_switch_network = vec!["networks", "switch", "main"];
        if !Command::new("safe")
            .args(args_add_network)
            .output()?
            .status
            .success()
        {
            return Err(eyre!("Failed to add network"));
        }

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
