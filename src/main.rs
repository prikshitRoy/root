//! Entry point for the `root` desktop application.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::process;

use root::installer::{execute_cli_command, parse_cli_args, CliCommand};

/// Launches the native desktop window by default, or handles installer CLI flags if provided.
fn main() -> eframe::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    match parse_cli_args(&args) {
        CliCommand::RunDefault => root::run_desktop_app(),
        CliCommand::OpenInstallerGui => root::run_installer_app(),
        cli_cmd => {
            if let Err(err) = execute_cli_command(cli_cmd) {
                eprintln!("Installer error: {err}");
                process::exit(1);
            }
            Ok(())
        }
    }
}
