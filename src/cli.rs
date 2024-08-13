use clap::{Parser, Subcommand};



#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init { name: String },
    Add {
        packages: Vec<String>,
        #[arg(short, long)]
        file: Option<String>,
    },
    Run { command: String, args: Vec<String> },
    Remove { package: Option<String>, file: Option<String> },
    Build { optimize: bool, obfuscate: bool, cross_platform: bool },
}
