//! Dedicated cross-platform GUI and CLI Setup Wizard binary (`root-installer`).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::process;

use root::installer::{
    execute_cli_command, is_invoked_as_installed_app, parse_cli_args, CliCommand,
};

/// Launches the Setup Wizard GUI, executes headless CLI install/uninstall, or runs `root` if invoked as the installed app.
fn main() -> eframe::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let current_exe = env::current_exe().ok();

    match parse_cli_args(&args) {
        CliCommand::RunDefault => {
            if is_invoked_as_installed_app(current_exe.as_deref()) {
                root::run_desktop_app()
            } else {
                root::run_installer_app()
            }
        }
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
