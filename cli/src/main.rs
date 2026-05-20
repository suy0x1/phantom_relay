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

            let mut table = Table::new();
            table.load_preset(UTF8_FULL);

            table.set_header(vec!["Service", "State"]);

            println!("\n{}\n", "▣ PhantomRelay Runtime Status".bold().cyan());

            for service in services {
                let (icon, state_text) = if service.active {
                    ("●", "RUNNING")
                } else {
                    ("○", "INACTIVE")
                };

                let name = service.name;

                table.add_row(vec![format!("{} {}", icon, name), state_text.to_string()]);
            }

            let output = table.to_string();

            let output = output
                .replace("RUNNING", &"RUNNING".green().bold().to_string())
                .replace("INACTIVE", &"INACTIVE".red().to_string())
                .replace("●", &"●".green().to_string())
                .replace("○", &"○".red().to_string());

            println!("{}", output);

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
