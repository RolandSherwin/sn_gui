use eframe::{
    egui::{self, RichText, ScrollArea},
    epaint::Color32,
};
use eyre::Result;
use std::process::{Child, Command, Stdio};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    sync::mpsc,
    task::JoinHandle,
    time::Duration,
};

enum NodeState {
    Running,
    Idle,
}

impl Default for NodeState {
    fn default() -> Self {
        Self::Idle
    }
}

struct Log {
    handle: JoinHandle<()>,
    log_reciever: mpsc::Receiver<String>,
    logs: Vec<String>,
}

#[derive(Default)]
pub struct NodeRunner {
    pub current_network_name: Option<String>,
    node_state: NodeState,
    // Cannot be inside node_state as we need mut ref to the things inside while still needing
    // shared ref to self.
    running_node_handle: Option<Child>,
    log: Option<Log>,
    pub status_sender: Option<mpsc::Sender<RichText>>,
}

impl NodeRunner {
    pub fn ui(&mut self, ctx: egui::Context) {
        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    let button_rect = egui::Rect::from_min_size(
                        ui.min_rect().min + egui::vec2(-100.0, 00.0),
                        egui::vec2(200.0, 100.0),
                    );
                    match self.node_state {
                        NodeState::Idle => {
                            if ui
                                .put(
                                    button_rect,
                                    egui::Button::new(RichText::new("Start node").heading()),
                                )
                                .clicked()
                            {
                                if let Some(network_name) = &self.current_network_name {
                                    match Self::run_node(network_name) {
                                        Ok(command_handle) => {
                                            let (log_handle, log_reciever) = self.read_log();
                                            self.send_status(RichText::new(
                                                "Node is running!".to_string(),
                                            ));
                                            let log = Log {
                                                handle: log_handle,
                                                logs: Vec::new(),
                                                log_reciever,
                                            };
                                            self.running_node_handle = Some(command_handle);
                                            self.log = Some(log);
                                            self.node_state = NodeState::Running;
                                        }
                                        Err(err) => self.send_status(
                                            RichText::new(format!("Error: {err}"))
                                                .color(Color32::RED),
                                        ),
                                    }
                                } else {
                                    self.send_status(
                                        RichText::new("Error: No default network set")
                                            .color(Color32::RED),
                                    );
                                }
                            }
                        }

                        NodeState::Running => {
                            if ui
                                .put(
                                    button_rect,
                                    egui::Button::new(RichText::new("Stop node").heading()),
                                )
                                .clicked()
                            {
                                if let Some(log) = &mut self.log {
                                    log.handle.abort();
                                }
                                if let Some(node_handle) = &mut self.running_node_handle {
                                    if node_handle.kill().is_err() {
                                        self.send_status(
                                            RichText::new("Error: Failed to kill node")
                                                .color(Color32::RED),
                                        );
                                    };
                                    self.send_status(RichText::new(
                                        "Node has been stopped!".to_string(),
                                    ));
                                }
                                self.node_state = NodeState::Idle
                            }
                        }
                    }

                    if let Some(log) = &mut self.log {
                        ui.add_space(20.0);
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                            ScrollArea::vertical()
                                .max_width(800.0)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        for line in &log.logs {
                                            ui.label(line);
                                        }
                                    });
                                });
                            if let Ok(line) = log.log_reciever.try_recv() {
                                log.logs.push(line);
                            }

                            if ui.button("Log dir").clicked() {
                                Command::new("xdg-open")
                                    .arg(".") // <- Specify the directory you'd like to open.
                                    .spawn()
                                    .unwrap();
                            }
                        });
                    }
                },
            );
        });
    }

    fn read_log(&self) -> (JoinHandle<()>, mpsc::Receiver<String>) {
        let (log_sender, log_reciever) = mpsc::channel(10000);
        let status_sender = self.status_sender.clone();
        let handle = tokio::spawn(async move {
            // allow time for the log file to be created
            tokio::time::sleep(Duration::from_secs(2)).await;
            if let Some(mut path) = dirs::home_dir() {
                path.push(".safe");
                path.push("node");
                path.push("local-node");
                path.push("sn_node.log");
                if let Ok(file) = File::open(path).await {
                    let mut lines = BufReader::new(file).lines();
                    while let Some(line) = lines.next_line().await.unwrap() {
                        if log_sender.send(line).await.is_err() {
                            log::error!("Failed to send logs");
                        }
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                } else {
                    Self::spawn_send_status(
                        status_sender,
                        RichText::new("Error: cannot open log file").color(Color32::RED),
                    );
                }
            } else {
                Self::spawn_send_status(
                    status_sender,
                    RichText::new("Error: cannot find home dir").color(Color32::RED),
                );
            }
        });
        (handle, log_reciever)
    }

    // send status to the footer
    fn send_status(&self, text: RichText) {
        Self::spawn_send_status(self.status_sender.clone(), text);
    }

    fn spawn_send_status(sender: Option<mpsc::Sender<RichText>>, text: RichText) {
        if let Some(sender) = sender {
            tokio::spawn(async move {
                if sender.send(text).await.is_err() {
                    log::error!("Failed to send status");
                };
            });
        }
    }

    fn run_node(netwowrk_name: &str) -> Result<Child> {
        let args_run_node = vec!["node", "join", "--network-name", netwowrk_name];
        // calling .kill() does not kill the child process unless Stdio::piped() is provided
        // Now the child process exists with a panic, "failed printing to stdout"
        let handle = Command::new("safe")
            .args(args_run_node)
            .env("RUST_LOG", "sn_node,sn_interface=trace")
            .stdout(Stdio::piped())
            .spawn()?;
        Ok(handle)
    }
}
