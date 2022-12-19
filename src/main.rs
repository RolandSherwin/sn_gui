use eframe::{
    egui::{self, Layout, RichText},
    epaint::Color32,
};
use eyre::{eyre, Result};
use std::process::{Child, Command, Stdio};

fn main() -> Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Safe Network",
        options,
        Box::new(|_cc| Box::<NodeRunner>::default()),
    );
    Ok(())
}

enum NodeState {
    Running(Child),
    Idle,
}

struct NodeRunner {
    error: Option<String>,
    node_state: NodeState,
}

impl Default for NodeRunner {
    fn default() -> Self {
        Self {
            error: Some("GG".to_owned()),
            node_state: NodeState::Idle,
        }
    }
}

impl eframe::App for NodeRunner {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                Layout::top_down_justified(eframe::emath::Align::Center),
                |ui| {
                    match &mut self.node_state {
                        NodeState::Idle => {
                            if ui.button(RichText::new("Start node").size(30.0)).clicked() {
                                self.error = None;
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
                                self.error = None;
                                handle.kill().unwrap();
                                self.node_state = NodeState::Idle
                            }
                        }
                    }

                    if let Some(error) = &self.error {
                        ui.colored_label(Color32::RED, error);
                    }
                },
            );
        });
    }
}

impl NodeRunner {
    fn setup_network() -> Result<()> {
        return Ok(());
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
