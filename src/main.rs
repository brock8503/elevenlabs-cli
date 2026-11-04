use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use colored::*;
use std::io;

#[cfg(feature = "audio")]
mod audio;
mod cli;
mod client;
mod commands;
mod config;
mod errors;
mod output;
mod playback;
mod utils;
mod validation;

#[cfg(feature = "mcp")]
mod mcp;
#[cfg(feature = "mcp")]
mod mcp_handlers;

// Import CLI types from the new modular structure
use cli::{
    Commands, ModelsArgs, ModelsCommands, TextToSpeechArgs, UserArgs, UserCommands, VoiceArgs,
    VoiceCommands,
};
use config::Config;
use output::print_error;

#[derive(Parser)]
#[command(
    name = "elevenlabs",
    about = "A comprehensive CLI for ElevenLabs AI audio platform",
    version = "0.1.0",
    author = "ElevenLabs CLI Contributors"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// API key for ElevenLabs (or set ELEVENLABS_API_KEY environment variable)
    #[arg(short, long, global = true, env = "ELEVENLABS_API_KEY")]
    api_key: Option<String>,

    /// Output format for audio (mp3_44100_128, mp3_44100_192, pcm_16000, etc.)
    #[arg(short, long, global = true, default_value = "mp3_44100_128")]
    format: String,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Skip confirmation prompts
    #[arg(short = 'y', long, global = true)]
    yes: bool,

    /// Output as JSON (for scripting and MCP integration)
    #[arg(short = 'j', long, global = true)]
    json: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle MCP mode (feature-gated)
    #[cfg(feature = "mcp")]
    if let Some(Commands::Mcp {
        enable_tools,
        disable_tools,
        disable_admin,
        disable_destructive,
        read_only,
    }) = &cli.command
    {
        return mcp::run_server(
            enable_tools.as_deref(),
            disable_tools.as_deref(),
            *disable_admin,
            *disable_destructive,
            *read_only,
        )
        .await;
    }

    // Handle no command - show help
    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            Cli::command().print_help()?;
            std::process::exit(0);
        }
    };

    // Handle completions command first (doesn't need API key)
    if let Commands::Completions { shell } = &command {
        let mut cmd = Cli::command();
        let name = cmd.get_name().to_string();
        generate(*shell, &mut cmd, name, &mut io::stdout());
        return Ok(());
    }

    // Handle update command (doesn't need API key)
    if let Commands::Update(args) = command {
        return commands::update::execute(args).await;
    }

    // Load or create config
    let mut config = Config::load()?;

    // Play doesn't need an API key
    if let Commands::Play(args) = command {
        let opts = playback::PlayOptions {
            local: args.local,
            remote_host: args.remote,
            remote_user: args.remote_user.or(config.remote_user),
        };
        return playback::play_file(std::path::Path::new(&args.file), &opts);
    }

    // Override config with CLI args if provided
    if let Some(api_key) = cli.api_key {
        config.api_key = Some(api_key);
    }

    // Ensure we have an API key
    let api_key = match config.api_key.as_ref() {
        Some(key) if !key.is_empty() => key.clone(),
        _ => {
            errors::print_api_error(&anyhow::anyhow!("API key is required"));
            std::process::exit(1);
        }
    };
    let assume_yes = cli.yes;
    let json_output = cli.json;

    // Apply config defaults
    let output_format_owned = config
        .default_output_format
        .clone()
        .unwrap_or_else(|| cli.format.clone());
    let output_format = output_format_owned.as_str();

    if cli.verbose {
        println!("{} Using ElevenLabs API", "ℹ".blue());
    }

    // Initialize JSON output mode in the output module
    output::set_json_mode(json_output);

    let result: Result<()> = async {
        match command {
            Commands::TextToSpeech(mut args) => {
                args.voice = args.voice.or_else(|| config.default_voice.clone());
                args.speed = args.speed.or(config.default_speed);
                args.remote_user = args.remote_user.or_else(|| config.remote_user.clone());
                commands::tts::execute(args, &api_key, output_format, assume_yes).await?
            }
            Commands::SpeechToText(args) => commands::stt::execute(args, &api_key).await?,
            Commands::Voice(args) => commands::voice::execute(args, &api_key, assume_yes).await?,
            Commands::AudioIsolation(args) => {
                commands::isolation::execute(args, &api_key, assume_yes).await?
            }
            Commands::SoundEffects(args) => {
                commands::sfx::execute(args, &api_key, assume_yes).await?
            }
            Commands::VoiceChanger(args) => {
                commands::voice_changer::execute(args, &api_key, output_format, assume_yes).await?
            }
            Commands::Dubbing(args) => {
                commands::dubbing::execute(args, &api_key, assume_yes).await?
            }
            Commands::History(args) => {
                commands::history::execute(args, &api_key, assume_yes).await?
            }
            Commands::User(args) => commands::user::execute(args, &api_key).await?,
            Commands::Models(args) => commands::models::execute(args, &api_key).await?,
            Commands::Config(args) => {
                // Edit the on-disk config, not the in-memory one: the latter has
                // ELEVENLABS_API_KEY / --api-key merged in and would persist it.
                let mut on_disk = Config::load()?;
                commands::config::execute(args, &mut on_disk)?
            }
            Commands::VoiceLibrary(args) => {
                commands::voice_library::execute(args, &api_key).await?
            }
            Commands::Pronunciation(args) => {
                commands::pronunciation::execute(args, &api_key).await?
            }
            Commands::Usage(args) => commands::usage::execute(args, &api_key).await?,
            Commands::VoiceDesign(args) => {
                commands::voice_design::execute(args, &api_key, assume_yes).await?
            }
            Commands::AudioNative(args) => commands::audio_native::execute(args, &api_key).await?,
            Commands::Samples(args) => {
                commands::samples::execute(args, &api_key, assume_yes).await?
            }
            Commands::Workspace(args) => commands::workspace::execute(args, &api_key).await?,
            Commands::TtsWithTimestamps(args) => {
                commands::tts_timestamps::execute(args, &api_key, output_format, assume_yes).await?
            }
            Commands::TtsStream(args) => {
                commands::tts_stream::execute(args, &api_key, assume_yes).await?
            }
            Commands::RealtimeTts(args) => {
                commands::realtime_tts::execute(args, &api_key, assume_yes).await?
            }
            Commands::Agent(args) => commands::agent::execute(args, &api_key).await?,
            Commands::Conversation(args) => {
                commands::conversation::execute(args, &api_key, assume_yes).await?
            }
            Commands::Knowledge(args) => commands::knowledge::execute(args, &api_key).await?,
            Commands::Rag(args) => commands::rag::execute(args, &api_key).await?,
            Commands::Webhook(args) => commands::webhook::execute(args, &api_key).await?,
            Commands::Dialogue(args) => {
                commands::dialogue::execute(args, &api_key, output_format, assume_yes).await?
            }
            Commands::Tools(args) => commands::tools::execute(args, &api_key).await?,
            Commands::Projects(args) => {
                commands::projects::execute(args, &api_key, assume_yes).await?
            }
            Commands::Music(args) => commands::music::execute(args, &api_key, assume_yes).await?,
            Commands::Phone(args) => commands::phone::execute(args, &api_key, assume_yes).await?,
            Commands::Completions { .. } => unreachable!(),
            Commands::Play(_) => unreachable!(),
            Commands::Update { .. } => unreachable!(),
            Commands::Interactive => {
                run_interactive_mode(&api_key, output_format, assume_yes).await?
            }
            #[cfg(feature = "mcp")]
            Commands::Mcp { .. } => unreachable!(),
        }
        Ok(())
    }
    .await;

    if let Err(e) = result {
        eprintln!("{} {:#}", "Error:".red(), e);
        errors::print_auth_hint(&e);
        std::process::exit(1);
    }

    Ok(())
}

async fn run_interactive_mode(api_key: &str, default_format: &str, assume_yes: bool) -> Result<()> {
    println!("{}", "ElevenLabs Interactive Mode".bold().underline());
    println!("Type 'help' for available commands, 'exit' to quit.\n");

    loop {
        let input = dialoguer::Input::<String>::new()
            .with_prompt("elevenlabs")
            .interact_text()?;

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "exit" | "quit" => {
                println!("Goodbye!");
                break;
            }
            "help" => {
                println!("Available commands:");
                println!("  tts <text>          - Text to speech");
                println!("  stt <file>          - Speech to text");
                println!("  voices              - List voices");
                println!("  models              - List models");
                println!("  user                - User info");
                println!("  exit                - Exit interactive mode");
            }
            "voices" => {
                let args = VoiceArgs {
                    command: VoiceCommands::List { detailed: false },
                };
                if let Err(e) = commands::voice::execute(args, api_key, assume_yes).await {
                    print_error(&format!("Failed to list voices: {}", e));
                }
            }
            "models" => {
                let args = ModelsArgs {
                    command: ModelsCommands::List,
                };
                if let Err(e) = commands::models::execute(args, api_key).await {
                    print_error(&format!("Failed to list models: {}", e));
                }
            }
            "user" => {
                let args = UserArgs {
                    command: UserCommands::Info,
                };
                if let Err(e) = commands::user::execute(args, api_key).await {
                    print_error(&format!("Failed to get user info: {}", e));
                }
            }
            "tts" => {
                if parts.len() < 2 {
                    println!("Usage: tts <text>");
                    continue;
                }
                let text = parts[1..].join(" ");
                let args = TextToSpeechArgs {
                    text: Some(text),
                    file: None,
                    voice: None,
                    model: "eleven_multilingual_v2".to_string(),
                    output: None,
                    play: false,
                    local: false,
                    remote: None,
                    remote_user: None,
                    output_device: None,
                    list_output_devices: false,
                    stability: None,
                    similarity_boost: None,
                    style: None,
                    speaker_boost: false,
                    speed: None,
                    language: None,
                    seed: None,
                };
                if let Err(e) =
                    commands::tts::execute(args, api_key, default_format, assume_yes).await
                {
                    print_error(&format!("TTS failed: {}", e));
                }
            }
            _ => {
                println!("{} Unknown command: {}", "✗".red(), cmd);
            }
        }
    }

    Ok(())
}
