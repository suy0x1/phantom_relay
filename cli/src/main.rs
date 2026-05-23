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

            println!("\n{}\n", "▣ PhantomRelay Runtime Status".bold().cyan());

            let mut has_services = false;
            let mut has_modes = false;

            for service in services {
                let (icon, state_text) = if service.active {
                    ("●", "RUNNING")
                } else {
                    ("○", "INACTIVE")
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

            if has_services {
                let services_output = service_table
                    .to_string()
                    .replace("RUNNING", &"RUNNING".green().bold().to_string())
                    .replace("INACTIVE", &"INACTIVE".red().to_string())
                    .replace("●", &"●".green().to_string())
                    .replace("○", &"○".red().to_string());

                println!("{}", "◉ Runtime Services".bold().blue());
                println!();
                println!("{}", services_output);
            }

            if has_modes {
                let modes_output = mode_table
                    .to_string()
                    .replace("RUNNING", &"RUNNING".green().bold().to_string())
                    .replace("INACTIVE", &"INACTIVE".red().to_string())
                    .replace("●", &"●".green().to_string())
                    .replace("○", &"○".red().to_string());

                if has_services {
                    println!();
                }

                println!("{}", "▣ Runtime Modes".bold().yellow());
                println!();
                println!("{}", modes_output);
            }

            println!(
                "\n{} {}\n",
                "hint:".dimmed(),
                "use `prctl start dns` / `stop proxy`".dimmed()
            );
        }

        IPCResponse::Error { message } => {
            println!("{} {}", "[error]".red(), message);
        }
    }

    Ok(())
}
