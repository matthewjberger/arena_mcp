#![windows_subsystem = "windows"]

use std::sync::mpsc;

use arena_app_protocol::{AgentStatus, BackendEvent, FrontendCommand};
use include_dir::{Dir, include_dir};
use nightshade::claude::{CliCommand, CliEvent, ClaudeConfig, McpConfig, spawn_cli_worker};
use nightshade::prelude::*;
use nightshade::webview::{WebviewContext, serve_embedded_dir};

static DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/site/dist");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cli_cmd_tx, cli_cmd_rx) = mpsc::channel::<CliCommand>();
    let (cli_event_tx, cli_event_rx) = mpsc::channel::<CliEvent>();

    let config = ClaudeConfig {
        system_prompt: Some(
            "You have access to Arena PLM tools via the arena MCP server. You can search items, view BOMs, look up changes/ECOs, check where-used references, view revisions, find suppliers, and explore quality processes. Use these tools to help answer questions about hardware parts, assemblies, and change orders in Arena.".to_string(),
        ),
        mcp_config: McpConfig::Custom(serde_json::json!({
            "mcpServers": {
                "arena": {
                    "command": "arena",
                    "args": []
                }
            }
        }).to_string()),
        ..Default::default()
    };

    spawn_cli_worker(cli_cmd_rx, cli_event_tx, config);

    let port = serve_embedded_dir(&DIST);

    launch(ArenaApp {
        port,
        ctx: WebviewContext::default(),
        connected: false,
        cli_cmd_tx,
        cli_event_rx,
    })?;

    Ok(())
}

struct ArenaApp {
    port: u16,
    ctx: WebviewContext<FrontendCommand, BackendEvent>,
    connected: bool,
    cli_cmd_tx: mpsc::Sender<CliCommand>,
    cli_event_rx: mpsc::Receiver<CliEvent>,
}

impl State for ArenaApp {
    fn title(&self) -> &str {
        "Arena PLM"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;
    }

    fn ui(&mut self, world: &mut World, ctx: &egui::Context) {
        self.process_frontend_commands();
        self.forward_cli_events();

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                if let Some(handle) = &world.resources.window.handle {
                    self.ctx.ensure_webview(
                        handle.clone(),
                        self.port,
                        ui.available_rect_before_wrap(),
                    );
                    handle.request_redraw();
                }
            });
    }
}

impl ArenaApp {
    fn process_frontend_commands(&mut self) {
        let commands: Vec<FrontendCommand> = self.ctx.drain_messages().collect();
        for command in commands {
            match command {
                FrontendCommand::Ready => {
                    if !self.connected {
                        self.ctx.send(BackendEvent::Connected);
                        self.ctx.send(BackendEvent::StatusUpdate {
                            status: AgentStatus::Idle,
                        });
                        self.connected = true;
                    }
                }
                FrontendCommand::SendPrompt {
                    prompt,
                    session_id,
                    model,
                } => {
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Thinking,
                    });
                    let _ = self.cli_cmd_tx.send(CliCommand::StartQuery {
                        prompt,
                        session_id,
                        model,
                    });
                }
                FrontendCommand::CancelRequest => {
                    let _ = self.cli_cmd_tx.send(CliCommand::Cancel);
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Idle,
                    });
                }
            }
        }
    }

    fn forward_cli_events(&mut self) {
        for event in self.cli_event_rx.try_iter() {
            match event {
                CliEvent::SessionStarted { session_id } => {
                    self.ctx.send(BackendEvent::StreamingStarted { session_id });
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Streaming,
                    });
                }
                CliEvent::TextDelta { text } => {
                    self.ctx.send(BackendEvent::TextDelta { text });
                }
                CliEvent::ThinkingDelta { text } => {
                    self.ctx.send(BackendEvent::ThinkingDelta { text });
                }
                CliEvent::ToolUseStarted { tool_name, tool_id } => {
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::UsingTool {
                            tool_name: tool_name.clone(),
                        },
                    });
                    self.ctx.send(BackendEvent::ToolUseStarted {
                        tool_name,
                        tool_id,
                    });
                }
                CliEvent::ToolUseInputDelta {
                    tool_id,
                    partial_json,
                } => {
                    self.ctx.send(BackendEvent::ToolUseInputDelta {
                        tool_id,
                        partial_json,
                    });
                }
                CliEvent::ToolUseFinished { tool_id } => {
                    self.ctx.send(BackendEvent::ToolUseFinished { tool_id });
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Streaming,
                    });
                }
                CliEvent::TurnComplete { session_id } => {
                    self.ctx.send(BackendEvent::TurnComplete { session_id });
                }
                CliEvent::Complete {
                    session_id,
                    total_cost_usd,
                    num_turns,
                } => {
                    self.ctx.send(BackendEvent::RequestComplete {
                        session_id,
                        total_cost_usd,
                        num_turns,
                    });
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Idle,
                    });
                }
                CliEvent::Error { message } => {
                    self.ctx.send(BackendEvent::Error { message });
                    self.ctx.send(BackendEvent::StatusUpdate {
                        status: AgentStatus::Idle,
                    });
                }
            }
        }
    }
}
