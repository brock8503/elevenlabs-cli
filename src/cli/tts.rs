//! Text-to-Speech CLI arguments

use clap::Args;

/// Text-to-Speech arguments
#[derive(Args)]
pub struct TextToSpeechArgs {
    /// Text to convert to speech (or use --file)
    #[arg(value_name = "TEXT")]
    pub text: Option<String>,

    /// Read text from file
    #[arg(short = 'i', long, value_name = "FILE")]
    pub file: Option<String>,

    /// Voice ID to use (default: config default_voice, else Brian)
    #[arg(long)]
    pub voice: Option<String>,

    /// Model to use
    #[arg(short, long, default_value = "eleven_multilingual_v2")]
    pub model: String,

    /// Output file path
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<String>,

    /// Play audio after generation
    #[arg(long)]
    pub play: bool,

    /// With --play: always play on this machine, even over ssh/mosh
    #[arg(long, requires = "play")]
    pub local: bool,

    /// With --play: play on this ssh host instead of the detected one
    #[arg(long, value_name = "HOST", requires = "play", conflicts_with = "local")]
    pub remote: Option<String>,

    /// ssh user for remote playback (config: remote_user)
    #[arg(long, value_name = "USER")]
    pub remote_user: Option<String>,

    /// Output audio device name
    /// Use --list-output-devices to see available devices
    #[arg(long, value_name = "DEVICE")]
    pub output_device: Option<String>,

    /// List available output audio devices
    #[arg(long)]
    pub list_output_devices: bool,

    /// Stability (0.0-1.0)
    #[arg(long, value_name = "0.0-1.0")]
    pub stability: Option<f32>,

    /// Similarity boost (0.0-1.0)
    #[arg(long, value_name = "0.0-1.0")]
    pub similarity_boost: Option<f32>,

    /// Style (0.0-1.0)
    #[arg(long, value_name = "0.0-1.0")]
    pub style: Option<f32>,

    /// Use speaker boost
    #[arg(long)]
    pub speaker_boost: bool,

    /// Speaking speed, 0.7 (slower) to 1.2 (faster); 1.0 is normal (default: config default_speed)
    #[arg(long, value_name = "0.7-1.2")]
    pub speed: Option<f32>,

    /// Language code (e.g., en, es, fr)
    #[arg(long, value_name = "CODE")]
    pub language: Option<String>,

    /// Seed for deterministic generation
    #[arg(long, value_name = "INT")]
    pub seed: Option<u32>,
}

/// TTS with Timestamps arguments
#[derive(Args)]
pub struct TtsTimestampsArgs {
    /// Text to convert to speech (or use --file)
    #[arg(value_name = "TEXT")]
    pub text: Option<String>,

    /// Read text from file
    #[arg(short = 'i', long, value_name = "FILE")]
    pub file: Option<String>,

    /// Voice ID to use
    #[arg(long, default_value = "Brian")]
    pub voice: String,

    /// Model to use
    #[arg(long, default_value = "eleven_multilingual_v2")]
    pub model: String,

    /// Output file path
    #[arg(long, value_name = "OUTPUT")]
    pub output: Option<String>,

    /// Output format
    #[arg(long, default_value = "mp3_44100_128")]
    pub output_format: String,

    /// Output subtitle file (SRT/VTT)
    #[arg(long, value_name = "FILE")]
    pub subtitles: Option<String>,

    /// Enable logging (default: true)
    #[arg(long)]
    pub enable_logging: Option<bool>,

    /// Latency optimization (0-4)
    #[arg(long, value_name = "0-4")]
    pub latency: Option<u8>,
}

/// TTS Streaming arguments
#[derive(Args)]
pub struct TtsStreamArgs {
    /// Text to convert to speech
    #[arg(value_name = "TEXT")]
    pub text: String,

    /// Voice ID to use
    #[arg(long, default_value = "Brian")]
    pub voice: String,

    /// Model to use
    #[arg(long, default_value = "eleven_multilingual_v2")]
    pub model: String,

    /// Output file path
    #[arg(long, value_name = "OUTPUT")]
    pub output: Option<String>,

    /// Play audio after generation
    #[arg(long)]
    pub play: bool,

    /// Output audio device name
    /// Use --list-output-devices to see available devices
    #[arg(long, value_name = "DEVICE")]
    pub output_device: Option<String>,

    /// List available output audio devices
    #[arg(long)]
    pub list_output_devices: bool,

    /// Latency optimization (0-4)
    #[arg(long, value_name = "0-4")]
    pub latency: Option<u8>,

    /// Output format
    #[arg(long, default_value = "mp3_44100_128")]
    pub output_format: String,

    /// Stability (0.0-1.0)
    #[arg(long, value_name = "FLOAT")]
    pub stability: Option<f32>,

    /// Similarity boost (0.0-1.0)
    #[arg(long, value_name = "FLOAT")]
    pub similarity_boost: Option<f32>,
}

/// Play an audio file, on the remote client when connected over ssh or mosh
#[derive(Args)]
pub struct PlayArgs {
    /// Audio file to play
    #[arg(value_name = "FILE")]
    pub file: String,

    /// Always play on this machine, even over ssh/mosh
    #[arg(long)]
    pub local: bool,

    /// Play on this ssh host instead of the detected one
    #[arg(long, value_name = "HOST", conflicts_with = "local")]
    pub remote: Option<String>,

    /// ssh user for remote playback (config: remote_user)
    #[arg(long, value_name = "USER")]
    pub remote_user: Option<String>,
}
