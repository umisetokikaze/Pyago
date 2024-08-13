use anyhow::Result;
use clap::Parser;
use log::{error, info};
use env_logger;

mod cli;
mod config;
mod commands;
mod utils;

use cli::{Cli, Commands};
use utils::DefaultPackageManager;

use crate::config::Config;
use std::path::Path;


fn main() {
    env_logger::init();

    if let Err(e) = run() {
        error!("エラーが発生しました: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    info!("Pyagoを起動しました");


    let _package_manager = DefaultPackageManager::new();

    match cli.command {
        Commands::Init { name } => commands::init(&name)?,
        Commands::Add { packages, file } => {
            let mut add_command = commands::AddCommand::new(packages, file);
                let mut config = Config::from_file(Path::new("pyago.toml"))?;
                add_command.execute(&mut config)?;
        },
        Commands::Run { command, args } => commands::run(&command, &args)?,
        Commands::Remove { package, file } => commands::remove_package(package.as_deref(), file.as_deref())?,
        Commands::Build { optimize, obfuscate, cross_platform } => commands::build(optimize, obfuscate, cross_platform)?,
    }

    info!("Pyagoが正常に終了しました");
    Ok(())
}
