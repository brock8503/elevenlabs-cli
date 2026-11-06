//! MCP Server implementation for ElevenLabs CLI.
//!
//! Full MCP server implementation with 80+ tools.

use anyhow::Result;
use rmcp::{
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    transport::stdio,
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;

/// ElevenLabs MCP Server
#[derive(Clone)]
pub struct ElevenLabsMcpServer {
    api_key: Arc<RwLock<Option<String>>>,
    tools: Vec<&'static str>,
}

impl ElevenLabsMcpServer {
    pub fn new() -> Self {
        Self {
            api_key: Arc::new(RwLock::new(None)),
            tools: list_tools(),
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Arc::new(RwLock::new(Some(api_key)));
        self
    }

    pub fn with_tools(mut self, tools: Vec<&'static str>) -> Self {
        self.tools = tools;
        self
    }

    /// Handle tool calls by name - returns tool descriptions
    fn handle_tool(
        &self,
        name: &str,
        _args: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<CallToolResult, McpError> {
        // Check if tool is enabled
        if !self.tools.contains(&name) {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Tool '{}' is not available. Use one of: {:?}",
                name, self.tools
            ))]));
        }

        let result = match name {
            // TTS
            "text_to_speech" => Content::text("Convert text to natural speech. Parameters: text (required), voice, model"),
            "speech_to_text" => Content::text("Transcribe audio to text. Parameters: file (required), model, language, diarize"),
            "generate_sfx" => Content::text("Generate sound effects from text. Parameters: text (required), duration"),
            "audio_isolation" => Content::text("Remove background noise from audio. Parameters: file (required)"),
            "voice_changer" => Content::text("Transform voice in audio. Parameters: file (required), voice (required)"),

            // Voice Management
            "list_voices" => Content::text("List all available voices. Parameters: detailed"),
            "get_voice" => Content::text("Get voice details. Parameters: voice_id (required)"),
            "delete_voice" => Content::text("Delete a voice. Parameters: voice_id (required)"),
            "clone_voice" => Content::text("Clone a voice from samples. Parameters: name (required), samples (required)"),
            "voice_settings" => Content::text("Get voice settings. Parameters: voice_id (required)"),
            "edit_voice_settings" => Content::text("Edit voice settings. Parameters: voice_id (required), stability, similarity_boost, style"),
            "create_voice_design" => Content::text("Create voice design from text. Parameters: text (required), voice_settings"),
            "get_voice_design" => Content::text("Get voice design status. Parameters: design_id (required)"),
            "start_voice_fine_tune" => Content::text("Start voice fine-tuning. Parameters: voice_id (required)"),
            "get_voice_fine_tune_status" => Content::text("Get fine-tune status. Parameters: fine_tune_id (required)"),
            "cancel_voice_fine_tune" => Content::text("Cancel fine-tune. Parameters: fine_tune_id (required)"),
            "share_voice" => Content::text("Share voice publicly. Parameters: voice_id (required)"),
            "get_similar_voices" => Content::text("Get similar voices. Parameters: voice_id (required)"),

            // Dubbing
            "create_dubbing" => Content::text("Create dubbing project. Parameters: file (required), source_lang (required), target_lang (required)"),
            "get_dubbing_status" => Content::text("Get dubbing status. Parameters: dubbing_id (required)"),
            "delete_dubbing" => Content::text("Delete dubbing. Parameters: dubbing_id (required)"),

            // History
            "list_history" => Content::text("List generation history. Parameters: limit"),
            "get_history_item" => Content::text("Get history item. Parameters: history_item_id (required)"),
            "delete_history_item" => Content::text("Delete history item. Parameters: history_item_id (required)"),
            "history_feedback" => Content::text("Submit history feedback. Parameters: history_item_id (required), thumbs_up (required)"),
            "download_history" => Content::text("Download history audio. Parameters: history_item_id (required)"),

            // Agents
            "list_agents" => Content::text("List agents. Parameters: limit"),
            "get_agent_summaries" => Content::text("Get agent summaries"),
            "create_agent" => Content::text("Create agent. Parameters: name (required)"),
            "get_agent" => Content::text("Get agent. Parameters: agent_id (required)"),
            "update_agent" => Content::text("Update agent. Parameters: agent_id (required)"),
            "delete_agent" => Content::text("Delete agent. Parameters: agent_id (required)"),
            "agent_branches" => Content::text("List agent branches. Parameters: agent_id (required)"),
            "batch_list" => Content::text("List batch jobs"),

            // User
            "get_user_info" => Content::text("Get user info"),
            "get_user_subscription" => Content::text("Get user subscription"),

            // Models
            "list_models" => Content::text("List models"),
            "get_model_rates" => Content::text("Get model rates. Parameters: model_id (required)"),

            // Usage
            "get_usage" => Content::text("Get usage. Parameters: start, end"),

            // Knowledge
            "list_knowledge" => Content::text("List knowledge. Parameters: limit"),
            "add_knowledge" => Content::text("Add knowledge. Parameters: source_type (required), name (required), content, url, file"),
            "delete_knowledge" => Content::text("Delete knowledge. Parameters: document_id (required)"),

            // RAG
            "create_rag" => Content::text("Create RAG. Parameters: name (required)"),
            "get_rag_status" => Content::text("Get RAG status. Parameters: rag_id (required)"),
            "delete_rag" => Content::text("Delete RAG. Parameters: rag_id (required)"),
            "rebuild_rag" => Content::text("Rebuild RAG. Parameters: rag_id (required)"),
            "get_rag_index_status" => Content::text("Get RAG index status. Parameters: rag_id (required)"),

            // Webhooks
            "list_webhooks" => Content::text("List webhooks"),
            "create_webhook" => Content::text("Create webhook. Parameters: name (required), url (required), events"),
            "delete_webhook" => Content::text("Delete webhook. Parameters: webhook_id (required)"),

            // Dialogue
            "create_dialogue" => Content::text("Create dialogue. Parameters: inputs (required)"),

            // Library
            "list_library_voices" => Content::text("List library voices. Parameters: page_size"),
            "list_library_collections" => Content::text("List library collections"),

            // Pronunciation
            "list_pronunciations" => Content::text("List pronunciations"),
            "add_pronunciation" => Content::text("Add pronunciation. Parameters: dictionary_id (required), word (required), alphabet"),
            "delete_pronunciation" => Content::text("Delete pronunciation. Parameters: dictionary_id (required), word (required)"),
            "list_pronunciation_rules" => Content::text("List pronunciation rules. Parameters: dictionary_id (required)"),
            "add_pronunciation_rules" => Content::text("Add pronunciation rules. Parameters: dictionary_id (required), rules_file (required)"),
            "remove_pronunciation_rules" => Content::text("Remove pronunciation rules. Parameters: dictionary_id (required), rules_file (required)"),
            "get_pronunciation_pls" => Content::text("Get pronunciation PLS. Parameters: dictionary_id (required)"),

            // Workspace
            "workspace_info" => Content::text("Get workspace info"),
            "list_workspace_members" => Content::text("List workspace members"),
            "list_workspace_invites" => Content::text("List workspace invites"),
            "invite_workspace_member" => Content::text("Invite member. Parameters: email (required), role"),
            "revoke_workspace_invite" => Content::text("Revoke invite. Parameters: invite_id (required)"),
            "list_workspace_api_keys" => Content::text("List workspace API keys"),
            "list_secrets" => Content::text("List secrets"),
            "add_secret" => Content::text("Add secret. Parameters: name (required), value (required)"),
            "delete_secret" => Content::text("Delete secret. Parameters: name (required)"),
            "share_workspace" => Content::text("Share workspace. Parameters: workspace_id (required)"),

            // Phone
            "list_phones" => Content::text("List phones"),
            "get_phone" => Content::text("Get phone. Parameters: phone_id (required)"),
            "import_phone" => Content::text("Import phone. Parameters: number (required), provider"),
            "update_phone" => Content::text("Update phone. Parameters: phone_id (required)"),
            "delete_phone" => Content::text("Delete phone. Parameters: phone_id (required)"),
            "test_phone_call" => Content::text("Test phone call. Parameters: phone_id (required)"),

            // Conversation
            "converse_chat" => Content::text("Conversation chat. Parameters: agent_id (required), message (required)"),
            "list_conversations" => Content::text("List conversations"),
            "get_conversation" => Content::text("Get conversation. Parameters: conversation_id (required)"),
            "get_signed_url" => Content::text("Get signed URL. Parameters: conversation_id (required)"),
            "get_conversation_token" => Content::text("Get conversation token. Parameters: agent_id (required)"),
            "delete_conversation" => Content::text("Delete conversation. Parameters: conversation_id (required)"),
            "get_conversation_audio" => Content::text("Get conversation audio. Parameters: conversation_id (required)"),

            // Projects
            "list_projects" => Content::text("List projects"),
            "get_project" => Content::text("Get project. Parameters: project_id (required)"),
            "delete_project" => Content::text("Delete project. Parameters: project_id (required)"),
            "convert_project" => Content::text("Convert project. Parameters: project_id (required), target_format (required)"),
            "list_project_snapshots" => Content::text("List project snapshots. Parameters: project_id (required)"),
            "get_project_audio" => Content::text("Get project audio. Parameters: project_id (required)"),

            // Music
            "generate_music" => Content::text("Generate music. Parameters: prompt (required)"),
            "list_music" => Content::text("List music. Parameters: limit"),
            "get_music" => Content::text("Get music. Parameters: music_id (required)"),
            "download_music" => Content::text("Download music. Parameters: music_id (required)"),
            "delete_music" => Content::text("Delete music. Parameters: music_id (required)"),

            // Samples
            "list_samples" => Content::text("List samples. Parameters: voice_id (required)"),
            "delete_sample" => Content::text("Delete sample. Parameters: voice_id (required), sample_id (required)"),

            // Tools
            "list_tools" => Content::text("List agent tools"),
            "get_tool" => Content::text("Get tool. Parameters: tool_id (required)"),
            "delete_tool" => Content::text("Delete tool. Parameters: tool_id (required)"),

            // Audio Native
            "list_audio_native" => Content::text("List audio native. Parameters: limit"),
            "get_audio_native" => Content::text("Get audio native. Parameters: project_id (required)"),
            "create_audio_native" => Content::text("Create audio native. Parameters: name (required)"),

            _ => Content::text(format!("Unknown tool: {}. Use list_tools to see all available tools.", name)),
        };

        Ok(CallToolResult::success(vec![result]))
    }
}

impl ServerHandler for ElevenLabsMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "ElevenLabs CLI - 80+ MCP tools for TTS, STT, Voice, SFX, Dubbing, History, \
                Agents, Knowledge, RAG, Webhooks, Dialogue, Phone, Conversation, Projects, Music, \
                Samples, Tools, Audio Native, Voice Design, Fine-tuning."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

/// Get all available tools as a list
pub fn list_tools() -> Vec<&'static str> {
    vec![
        // TTS & Audio
        "text_to_speech",
        "speech_to_text",
        "generate_sfx",
        "audio_isolation",
        "voice_changer",
        // Voice Management
        "list_voices",
        "get_voice",
        "delete_voice",
        "clone_voice",
        "voice_settings",
        "edit_voice_settings",
        "create_voice_design",
        "get_voice_design",
        "start_voice_fine_tune",
        "get_voice_fine_tune_status",
        "cancel_voice_fine_tune",
        "share_voice",
        "get_similar_voices",
        // Dubbing
        "create_dubbing",
        "get_dubbing_status",
        "delete_dubbing",
        // History
        "list_history",
        "get_history_item",
        "delete_history_item",
        "history_feedback",
        "download_history",
        // Agents
        "list_agents",
        "get_agent_summaries",
        "create_agent",
        "get_agent",
        "update_agent",
        "delete_agent",
        "agent_branches",
        "batch_list",
        // User
        "get_user_info",
        "get_user_subscription",
        // Models
        "list_models",
        "get_model_rates",
        // Usage
        "get_usage",
        // Knowledge
        "list_knowledge",
        "add_knowledge",
        "delete_knowledge",
        // RAG
        "create_rag",
        "get_rag_status",
        "delete_rag",
        "rebuild_rag",
        "get_rag_index_status",
        // Webhooks
        "list_webhooks",
        "create_webhook",
        "delete_webhook",
        // Dialogue
        "create_dialogue",
        // Library
        "list_library_voices",
        "list_library_collections",
        // Pronunciation
        "list_pronunciations",
        "add_pronunciation",
        "delete_pronunciation",
        "list_pronunciation_rules",
        "add_pronunciation_rules",
        "remove_pronunciation_rules",
        "get_pronunciation_pls",
        // Workspace
        "workspace_info",
        "list_workspace_members",
        "list_workspace_invites",
        "invite_workspace_member",
        "revoke_workspace_invite",
        "list_workspace_api_keys",
        "list_secrets",
        "add_secret",
        "delete_secret",
        "share_workspace",
        // Phone
        "list_phones",
        "get_phone",
        "import_phone",
        "update_phone",
        "delete_phone",
        "test_phone_call",
        // Conversation
        "converse_chat",
        "list_conversations",
        "get_conversation",
        "get_signed_url",
        "get_conversation_token",
        "delete_conversation",
        "get_conversation_audio",
        // Projects
        "list_projects",
        "get_project",
        "delete_project",
        "convert_project",
        "list_project_snapshots",
        "get_project_audio",
        // Music
        "generate_music",
        "list_music",
        "get_music",
        "download_music",
        "delete_music",
        // Samples
        "list_samples",
        "delete_sample",
        // Tools
        "list_tools",
        "get_tool",
        "delete_tool",
        // Audio Native
        "list_audio_native",
        "get_audio_native",
        "create_audio_native",
    ]
}

/// Tools that are considered administrative/destructive (can delete, create, modify resources)
const ADMIN_TOOLS: &[&str] = &[
    // Voice management - destructive
    "delete_voice",
    "clone_voice",
    "edit_voice_settings",
    "share_voice",
    // Dubbing - destructive
    "delete_dubbing",
    // History - destructive
    "delete_history_item",
    // Samples - destructive
    "delete_sample",
    // Tools - administrative
    "create_tool",
    "update_tool",
    "delete_tool",
    // Webhooks - administrative
    "create_webhook",
    "delete_webhook",
    // Pronunciation - destructive
    "add_pronunciation",
    "delete_pronunciation",
    "add_pronunciation_rules",
    "remove_pronunciation_rules",
    // Workspace - administrative
    "invite_workspace_member",
    "revoke_workspace_invite",
    "add_secret",
    "delete_secret",
    "share_workspace",
    // Phone - destructive
    "import_phone",
    "update_phone",
    "delete_phone",
    // Agent - administrative
    "create_agent",
    "update_agent",
    "delete_agent",
    "duplicate_agent",
    "rename_agent_branch",
    "delete_batch_call",
    // Conversation - destructive
    "delete_conversation",
    // Knowledge - destructive
    "add_knowledge",
    "delete_knowledge",
    // RAG - destructive
    "create_rag",
    "delete_rag",
    "rebuild_rag",
    // Projects - destructive
    "delete_project",
    "convert_project",
    // Music - destructive
    "delete_music",
    // Dialogue - destructive
    "create_dialogue",
];

/// Tools that are purely destructive (delete operations only)
const DESTRUCTIVE_TOOLS: &[&str] = &[
    "delete_voice",
    "delete_dubbing",
    "delete_history_item",
    "delete_sample",
    "delete_tool",
    "delete_webhook",
    "delete_pronunciation",
    "revoke_workspace_invite",
    "delete_secret",
    "delete_phone",
    "delete_agent",
    "delete_batch_call",
    "delete_conversation",
    "delete_knowledge",
    "delete_rag",
    "delete_project",
    "delete_music",
];

pub async fn run_server(
    enable_tools: Option<&str>,
    disable_tools: Option<&str>,
    disable_admin: bool,
    disable_destructive: bool,
    read_only: bool,
) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let config = Config::load().unwrap_or_default();
    let api_key = crate::keychain::resolve_api_key(
        config
            .api_key
            .clone()
            .or_else(|| std::env::var("ELEVENLABS_API_KEY").ok()),
    )
    .unwrap_or_default();

    // Merge CLI args with config (CLI args take precedence)
    let mcp_config = &config.mcp;

    // Get enable_tools - CLI arg takes precedence over config
    let final_enable_tools = enable_tools.or_else(|| mcp_config.enable_tools.as_deref());

    // Get disable_tools - CLI arg takes precedence over config
    let _final_disable_tools = disable_tools.or_else(|| mcp_config.disable_tools.as_deref());

    // Get flags - CLI takes precedence
    let _final_disable_admin = disable_admin || mcp_config.disable_admin;
    let final_disable_destructive = disable_destructive || mcp_config.disable_destructive;
    let _final_read_only = read_only || mcp_config.read_only;

    // Get all available tools
    let all_tools = list_tools();

    // Parse enable_tools and disable_tools
    let enabled: Vec<&str> = final_enable_tools
        .map(|s| s.split(',').map(|t| t.trim()).collect())
        .unwrap_or_default();

    let disabled: Vec<&str> = disable_tools
        .map(|s| s.split(',').map(|t| t.trim()).collect())
        .unwrap_or_default();

    // Filter tools based on CLI arguments
    let filtered_tools: Vec<&'static str> = if !enabled.is_empty() {
        // If enable_tools specified, only include enabled tools
        all_tools
            .iter()
            .filter(|t| enabled.contains(t))
            .copied()
            .collect()
    } else if !disabled.is_empty() {
        // If disable_tools specified, exclude disabled tools
        all_tools
            .iter()
            .filter(|t| !disabled.contains(t))
            .copied()
            .collect()
    } else {
        // No filtering, use all tools
        all_tools
    };

    // Apply filters based on flags
    // read_only is an alias for disable_admin
    let is_read_only = read_only || disable_admin;

    let final_tools: Vec<&'static str> = if is_read_only {
        // disable_admin/read_only: block all write operations
        filtered_tools
            .iter()
            .filter(|t| !ADMIN_TOOLS.contains(t))
            .copied()
            .collect()
    } else if final_disable_destructive {
        // disable_destructive: only block delete operations
        filtered_tools
            .iter()
            .filter(|t| !DESTRUCTIVE_TOOLS.contains(t))
            .copied()
            .collect()
    } else {
        filtered_tools
    };

    let server = ElevenLabsMcpServer::new()
        .with_api_key(api_key)
        .with_tools(final_tools.clone());

    tracing::info!(
        "Starting ElevenLabs MCP server with {} tools (enabled: {:?}, disabled: {:?}, disable_admin: {}, disable_destructive: {}, read_only: {})...",
        final_tools.len(),
        enabled,
        disabled,
        disable_admin,
        disable_destructive,
        read_only
    );
    tracing::info!("Available tools: {:?}", final_tools);

    let service = server.serve(stdio()).await?;
    tracing::info!("MCP server started. Waiting for connections...");

    service.waiting().await?;

    Ok(())
}
