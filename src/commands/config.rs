use crate::cli::{ConfigArgs, ConfigCommands};
use crate::config::Config;
use crate::output::print_success;
use anyhow::Result;
use colored::*;

pub fn execute(args: ConfigArgs, config: &mut Config) -> Result<()> {
    match args.command {
        ConfigCommands::Show => show_config(config),
        ConfigCommands::Set { key, value } => set_config(config, &key, &value),
        ConfigCommands::Unset { key } => unset_config(config, &key),
    }
}

fn show_config(config: &Config) -> Result<()> {
    println!("{}", "Current Configuration:".bold().underline());

    let config_path = Config::config_path()?;
    println!(
        "  Config file: {}",
        config_path.display().to_string().dimmed()
    );

    println!("\n  Values:");
    println!(
        "    api_key: {}",
        if config.api_key.is_some() {
            "[set]".green().to_string()
        } else {
            "[not set]".red().to_string()
        }
    );

    println!(
        "    default_voice: {}",
        config.default_voice.as_deref().unwrap_or("[not set]")
    );
    println!(
        "    default_speed: {}",
        config
            .default_speed
            .map(|s| s.to_string())
            .unwrap_or_else(|| "[not set]".to_string())
    );

    println!(
        "    default_model: {}",
        config.default_model.as_deref().unwrap_or("[not set]")
    );

    println!(
        "    default_output_format: {}",
        config
            .default_output_format
            .as_deref()
            .unwrap_or("[not set]")
    );

    Ok(())
}

fn set_config(config: &mut Config, key: &str, value: &str) -> Result<()> {
    match key {
        "api_key" => {
            config.set(key, value)?;
            print_success(&format!("Set '{}' to [hidden]", key));
        }
        _ => {
            config.set(key, value)?;
            print_success(&format!("Set '{}' to '{}'", key, value.cyan()));
        }
    }
    Ok(())
}

fn unset_config(config: &mut Config, key: &str) -> Result<()> {
    config.unset(key)?;
    print_success(&format!("Unset '{}'", key));
    Ok(())
}
