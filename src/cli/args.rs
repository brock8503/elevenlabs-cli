//! Global CLI arguments and command definitions

use clap::Subcommand;

// Import subcommand types from other modules
use super::agent::*;
use super::audio::*;
use super::config::*;
use super::conversation::*;
use super::design::*;
use super::dialogue::*;
use super::dubbing::*;
use super::history::*;
use super::knowledge::*;
use super::library::*;
use super::models::*;
use super::music::*;
use super::native::*;
use super::phone::*;
use super::projects::*;
use super::pronunciation::*;
use super::rag::*;
use super::samples::*;
use super::stt::*;
use super::tools::*;
use super::tts::*;
use super::update::*;
use super::usage::*;
use super::user::*;
use super::voice::*;
use super::webhook::*;
use super::workspace::*;

/// Main command enum for all CLI subcommands
#[derive(Subcommand)]
pub enum Commands {
    /// Text-to-Speech operations
    #[command(name = "tts", alias = "speak")]
    TextToSpeech(TextToSpeechArgs),

    /// Speech-to-Text operations
    #[command(name = "stt", alias = "transcribe")]
    SpeechToText(SpeechToTextArgs),

    /// Voice management operations
    #[command(name = "voice")]
    Voice(VoiceArgs),

    /// Audio isolation (remove background noise)
    #[command(name = "isolate")]
    AudioIsolation(AudioIsolationArgs),

    /// Generate sound effects from text
    #[command(name = "sfx")]
    SoundEffects(SoundEffectsArgs),

    /// Voice changer (speech-to-speech)
    #[command(name = "voice-changer", alias = "vc")]
    VoiceChanger(VoiceChangerArgs),

    /// Dubbing operations
    #[command(name = "dub")]
    Dubbing(DubbingArgs),

    /// History operations
    #[command(name = "history")]
    History(HistoryArgs),

    /// User information
    #[command(name = "user")]
    User(UserArgs),

    /// Model information
    #[command(name = "models")]
    Models(ModelsArgs),

    /// Configuration management
    #[command(name = "config")]
    Config(ConfigArgs),

    /// Voice library (shared voices)
    #[command(name = "library", alias = "lib")]
    VoiceLibrary(VoiceLibraryArgs),

    /// Pronunciation dictionaries
    #[command(name = "pronunciation", alias = "pronounce")]
    Pronunciation(PronunciationArgs),

    /// Usage statistics
    #[command(name = "usage")]
    Usage(UsageArgs),

    /// Voice design (create voices from text)
    #[command(name = "voice-design", alias = "design")]
    VoiceDesign(VoiceDesignArgs),

    /// Audio Native (embeddable audio player)
    #[command(name = "audio-native")]
    AudioNative(AudioNativeArgs),

    /// Sample management
    #[command(name = "samples")]
    Samples(SamplesArgs),

    /// Workspace management
    #[command(name = "workspace", alias = "ws")]
    Workspace(WorkspaceArgs),

    /// Play an audio file where you are (remote client over ssh/mosh, else locally)
    #[command(name = "play")]
    Play(PlayArgs),

    /// Text-to-Speech with character-level timestamps
    #[command(name = "tts-timestamps", alias = "tts-ts")]
    TtsWithTimestamps(TtsTimestampsArgs),

    /// Streaming Text-to-Speech with timestamps
    #[command(name = "tts-stream")]
    TtsStream(TtsStreamArgs),

    /// Real-time Text-to-Speech via WebSocket (ultra-low latency)
    #[command(name = "realtime-tts", alias = "rtts")]
    RealtimeTts(RealtimeTtsArgs),

    /// Agent management
    #[command(name = "agent")]
    Agent(AgentArgs),

    /// Conversation with agent (WebSocket)
    #[command(name = "converse", alias = "chat")]
    Conversation(ConversationArgs),

    /// Knowledge base management
    #[command(name = "knowledge", alias = "kb")]
    Knowledge(KnowledgeArgs),

    /// RAG index management
    #[command(name = "rag")]
    Rag(RagArgs),

    /// Webhook management
    #[command(name = "webhook")]
    Webhook(WebhookArgs),

    /// Text-to-Dialogue (multi-voice)
    #[command(name = "dialogue")]
    Dialogue(DialogueArgs),

    /// Tools management (agent tools)
    #[command(name = "tools")]
    Tools(ToolsArgs),

    /// Projects management
    #[command(name = "projects")]
    Projects(ProjectsArgs),

    /// Music generation
    #[command(name = "music")]
    Music(MusicArgs),

    /// Phone number management (Twilio/SIP)
    #[command(name = "phone")]
    Phone(PhoneArgs),

    /// Generate shell completions
    #[command(name = "completions")]
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// Update the CLI to the latest version
    #[command(name = "update")]
    Update(UpdateArgs),

    /// Interactive mode (REPL)
    #[command(name = "interactive", alias = "repl")]
    Interactive,

    /// Run as MCP (Model Context Protocol) server for AI assistants
    #[cfg(feature = "mcp")]
    #[command(name = "mcp")]
    Mcp {
        /// Comma-separated list of tools to enable (e.g., "tts,stt,voice")
        #[arg(long)]
        enable_tools: Option<String>,

        /// Comma-separated list of tools to disable (e.g., "agents,phone")
        #[arg(long)]
        disable_tools: Option<String>,

        /// Disable all administrative operations (delete, create, update)
        #[arg(long)]
        disable_admin: bool,

        /// Disable only destructive operations (delete only)
        #[arg(long)]
        disable_destructive: bool,

        /// Read-only mode: disable all write operations (same as --disable-admin)
        #[arg(long)]
        read_only: bool,
    },
}
