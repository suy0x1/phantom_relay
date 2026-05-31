mod cli;
mod ipc;
mod runtime;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use comfy_table::{Table, presets::UTF8_FULL};

use crate::{
    cli::{args::Cli, client::send_command, parser::to_runtime_command},
    ipc::protocol::IPCResponse,
};

fn side_by_side(left: &str, right: &str, gap: usize) -> String {
    let left_lines: Vec<&str> = left.lines().collect();
    let right_lines: Vec<&str> = right.lines().collect();

    let left_width = left_lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);

    let total_lines = left_lines.len().max(right_lines.len());
    let mut out = String::new();

    for i in 0..total_lines {
        let l = left_lines.get(i).copied().unwrap_or("");
        let r = right_lines.get(i).copied().unwrap_or("");

        out.push_str(l);

        let pad = left_width.saturating_sub(l.chars().count()) + gap;
        out.push_str(&" ".repeat(pad));

        out.push_str(r);
        out.push('\n');
    }

    out
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let runtime_cmd = to_runtime_command(cli.command)?;
    let response = send_command(runtime_cmd).await?;

    match response {
        IPCResponse::Success { message } => {
            println!("{} {}", "[ok]".green(), message);
        }

        IPCResponse::Status { services } => {
            use colored::control::set_override;
            set_override(true);

            let mut service_table = Table::new();
            service_table.load_preset(UTF8_FULL);
            service_table.set_header(vec!["Service", "State"]);

            let mut mode_table = Table::new();
            mode_table.load_preset(UTF8_FULL);
            mode_table.set_header(vec!["Mode", "State"]);

            let mut has_services = false;
            let mut has_modes = false;

            for service in services {
                let (icon, state_text) = if service.is_mode {
                    if service.active {
                        ("●", "ENABLED")
                    } else {
                        ("○", "DISABLED")
                    }
                } else {
                    if service.active {
                        ("●", "RUNNING")
                    } else {
                        ("○", "INACTIVE")
                    }
                };

                let row = vec![format!("{} {}", icon, service.name), state_text.to_string()];

                if service.is_mode {
                    has_modes = true;
                    mode_table.add_row(row);
                } else {
                    has_services = true;
                    service_table.add_row(row);
                }
            }

            println!("\n{}\n", "▣ PhantomRelay Runtime Status".bold().cyan());

            let services_output = if has_services {
                Some(service_table.to_string())
            } else {
                None
            };

            let modes_output = if has_modes {
                Some(mode_table.to_string())
            } else {
                None
            };

            match (services_output, modes_output) {
                (Some(left), Some(right)) => {
                    let combined = side_by_side(&left, &right, 6);

                    let colored = combined
                        .replace("RUNNING", &"RUNNING".green().bold().to_string())
                        .replace("INACTIVE", &"INACTIVE".red().to_string())
                        .replace("ENABLED", &"ENABLED".green().bold().to_string())
                        .replace("DISABLED", &"DISABLED".red().to_string())
                        .replace("●", &"●".green().to_string())
                        .replace("○", &"○".red().to_string());

                    println!("{}", colored);
                }

                (Some(left), None) => {
                    let colored = left
                        .replace("RUNNING", &"RUNNING".green().bold().to_string())
                        .replace("INACTIVE", &"INACTIVE".red().to_string())
                        .replace("ENABLED", &"ENABLED".green().bold().to_string())
                        .replace("DISABLED", &"DISABLED".red().to_string())
                        .replace("●", &"●".green().to_string())
                        .replace("○", &"○".red().to_string());

                    println!("{}", colored);
                }

                (None, Some(right)) => {
                    let colored = right
                        .replace("RUNNING", &"RUNNING".green().bold().to_string())
                        .replace("INACTIVE", &"INACTIVE".red().to_string())
                        .replace("ENABLED", &"ENABLED".green().bold().to_string())
                        .replace("DISABLED", &"DISABLED".red().to_string())
                        .replace("●", &"●".green().to_string())
                        .replace("○", &"○".red().to_string());

                    println!("{}", colored);
                }

                (None, None) => {
                    println!("{}", "no runtime entries found".dimmed());
                }
            }

            println!(
                "\n{} {}\n",
                "hint:".dimmed(),
                "use `prctl start dns` / `prctl enable dns-turbo`".dimmed()
            );
        }

        IPCResponse::Error { message } => {
            println!("{} {}", "[error]".red(), message);
        }
    }

    Ok(())
}
