use clap::Parser;
use std::fs;
use tracing::{debug, error, info, trace, warn};

mod config;
mod constants;
mod logging;
mod tools;

/// A lightweight Windows utility that applies modular configuration profiles to customize your system.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long)]
    config: String,

    /// Do not ask for confirmation before applying the configuration
    #[arg(long)]
    noconfirm: bool,

    /// Verbose mode (-v, -vv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Generate an empty configuration file at the specified path
    #[arg(long)]
    generate_config: bool,
}

fn main() {
    let args = Args::parse();
    match args.verbose {
        0 => logging::setup("info", Some("%Y-%m-%d_%H-%M-%S.log")).unwrap(),
        1 => logging::setup("debug", Some("%Y-%m-%d_%H-%M-%S.log")).unwrap(),
        2 => logging::setup("trace", Some("%Y-%m-%d_%H-%M-%S.log")).unwrap(),
        _ => logging::setup("info", Some("%Y-%m-%d_%H-%M-%S.log")).unwrap(),
    }
    info!(
        "trx8 v{} - {}",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_REPOSITORY")
    );
    if args.generate_config {
        let empty_config = config::Config::generate_empty();
        let config_path = args.config;
        let json_string = serde_json::to_string_pretty(&empty_config);
        match json_string {
            Ok(json) => {
                std::fs::write(&config_path, json).unwrap();
                info!("Empty configuration file written to: {}", config_path);
            }
            Err(e) => {
                error!("Failed to generate configuration file: {}", e);
            }
        }
        return;
    }
    info!("Reading configuration file at: {}", args.config);
    let config_text = match fs::read_to_string(args.config) {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read configuration file: {}", e);
            return;
        }
    };
    let config = match serde_json::from_str::<config::Config>(&config_text) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to parse configuration file: {}", e);
            return;
        }
    };
    trace!("Configuration loaded: {:?}", config);
    info!("");
    info!("====/ CONFIGURATION INFORMATION /===");
    info!("Name: {}", config.metadata.name);
    if let Some(description) = &config.metadata.description {
        info!("Description: {}", description);
    }
    info!("Version: {}", config.metadata.version);
    if let Some(authors) = &config.metadata.author {
        info!("Author(s): {}", authors.join(", "));
    }
    info!("====================================");
    warn!("");
    if !args.noconfirm {
        warn!(
            "Before applying, make sure to only use trusted configuration files as they can execute arbitrary code and potentially harm your system."
        );
        warn!("Do you wish to proceed with applying this configuration? (y/N):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.trim().to_lowercase() != "y" {
            info!("Operation cancelled by user.");
            return;
        }
    } else {
        warn!("No confirmation flag detected, proceeding without confirmation :)");
    }

    // Execute the configuration
    // TODO: Move this to a separate function to clean up main.rs
    for (i, script) in config.scripts.iter().enumerate() {
        info!(
            "Running script ({}/{}): {}",
            i + 1,
            config.scripts.len(),
            script.name
        );
        if let Some(description) = &script.description {
            info!("Script description: {}", description);
        }
        // TODO: Implement condition checking for scripts
        for action in &script.actions {
            debug!("Executing action: {}", action.name);
            if let Some(description) = &action.description {
                trace!("Action information: {}", description);
            }
            tools::execute_action(&action.name, &action.parameters);
        }
    }
    info!("Configuration applied successfully.");
}
