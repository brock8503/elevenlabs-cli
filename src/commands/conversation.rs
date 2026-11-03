use crate::cli::{ConversationArgs, ConversationCommands, ConverseArgs};
use crate::client::create_http_client;
use crate::output::{print_error, print_info, print_success, print_warning};
use crate::utils::{confirm_overwrite, write_bytes_to_file};
use anyhow::{Context, Result};
use colored::*;
use comfy_table::Table;
use dialoguer::Confirm;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/// WebSocket connection timeout in seconds
const WS_CONNECT_TIMEOUT_SECS: u64 = 30;

/// Conversation command dispatcher
pub async fn execute(args: ConversationArgs, api_key: &str, assume_yes: bool) -> Result<()> {
    match args.command {
        ConversationCommands::Converse(converse_args) => {
            start_websocket_conversation(converse_args).await
        }
        ConversationCommands::List {
            agent_id,
            branch_id,
            limit,
        } => list_conversations(&api_key, agent_id.as_deref(), branch_id.as_deref(), limit).await,
        ConversationCommands::Get { conversation_id } => {
            get_conversation(&api_key, &conversation_id).await
        }
        ConversationCommands::SignedUrl {
            agent_id,
            branch_id,
        } => get_signed_url(&api_key, &agent_id, branch_id.as_deref()).await,
        ConversationCommands::Token {
            agent_id,
            branch_id,
        } => get_conversation_token(&api_key, &agent_id, branch_id.as_deref()).await,
        ConversationCommands::Delete { conversation_id } => {
            delete_conversation(&api_key, &conversation_id, assume_yes).await
        }
        ConversationCommands::Audio {
            conversation_id,
            output,
        } => {
            get_conversation_audio(&api_key, &conversation_id, output.as_deref(), assume_yes).await
        }
        ConversationCommands::Feedback {
            conversation_id,
            thumbs_up,
            feedback,
        } => {
            send_conversation_feedback(&api_key, &conversation_id, thumbs_up, feedback.as_deref())
                .await
        }
        ConversationCommands::Outbound {
            agent_id,
            caller_id,
            to,
            message,
        } => outbound_call(&api_key, &agent_id, &caller_id, &to, message.as_deref()).await,
    }
}

/// List conversations
async fn list_conversations(
    api_key: &str,
    agent_id: Option<&str>,
    branch_id: Option<&str>,
    limit: Option<u32>,
) -> Result<()> {
    let client = create_http_client();
    print_info("Fetching conversations...");

    let mut url = "https://api.elevenlabs.io/v1/convai/conversations".to_string();
    let mut params = Vec::new();

    if let Some(aid) = agent_id {
        params.push(format!("agent_id={}", aid));
    }
    if let Some(bid) = branch_id {
        params.push(format!("branch_id={}", bid));
    }
    if let Some(lim) = limit {
        params.push(format!("page_size={}", lim));
    }

    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to fetch conversations")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct ConversationListItem {
        conversation_id: String,
        #[serde(default)]
        agent_id: Option<String>,
        #[serde(default)]
        version_id: Option<String>,
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        created_at: Option<String>,
    }

    #[derive(Deserialize)]
    struct ConversationsResponse {
        conversations: Vec<ConversationListItem>,
    }

    let result: ConversationsResponse =
        response.json().await.context("Failed to parse response")?;

    if result.conversations.is_empty() {
        print_info("No conversations found");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec![
        "Conversation ID",
        "Agent",
        "Version",
        "Status",
        "Created",
    ]);

    for conv in &result.conversations {
        let agent = conv
            .agent_id
            .as_deref()
            .map(|s| {
                if s.len() > 12 {
                    format!("{}...", &s[..12])
                } else {
                    s.to_string()
                }
            })
            .unwrap_or_default();
        let version = conv.version_id.as_deref().unwrap_or("-");

        table.add_row(vec![
            conv.conversation_id.yellow(),
            agent.cyan(),
            version.into(),
            conv.status.as_deref().unwrap_or("-").into(),
            conv.created_at.as_deref().unwrap_or("-").into(),
        ]);
    }

    println!("{}", table);
    print_success(&format!(
        "Found {} conversation(s)",
        result.conversations.len()
    ));
    Ok(())
}

/// Get conversation details
async fn get_conversation(api_key: &str, conversation_id: &str) -> Result<()> {
    let client = create_http_client();
    print_info(&format!(
        "Fetching conversation '{}'...",
        conversation_id.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/conversations/{}",
        conversation_id
    );
    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct ConversationDetail {
        conversation_id: String,
        #[serde(default)]
        agent_id: Option<String>,
        #[serde(default)]
        version_id: Option<String>,
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        created_at: Option<String>,
        #[serde(default)]
        transcript: Option<Vec<TranscriptMessage>>,
    }

    #[derive(Deserialize)]
    struct TranscriptMessage {
        role: String,
        content: String,
        #[serde(default)]
        timestamp: Option<String>,
    }

    let conv: ConversationDetail = response.json().await?;

    let mut table = Table::new();
    table.set_header(vec!["Property", "Value"]);
    table.add_row(vec!["Conversation ID", &conv.conversation_id.yellow()]);

    if let Some(agent) = &conv.agent_id {
        table.add_row(vec!["Agent ID", agent]);
    }
    if let Some(version) = &conv.version_id {
        table.add_row(vec!["Version ID", version]);
    }
    if let Some(status) = &conv.status {
        table.add_row(vec!["Status", status]);
    }
    if let Some(created) = &conv.created_at {
        table.add_row(vec!["Created", created]);
    }

    println!("{}", table);

    if let Some(transcript) = &conv.transcript {
        if !transcript.is_empty() {
            println!("\n{}", "Transcript:".bold().underline());
            for msg in transcript {
                let role_color = match msg.role.as_str() {
                    "user" => "User".blue(),
                    "agent" | "assistant" => "Agent".green(),
                    _ => msg.role.normal(),
                };
                let timestamp = msg.timestamp.as_deref().unwrap_or("");
                println!("[{}] {}: {}", timestamp, role_color, msg.content);
            }
        }
    }

    Ok(())
}

/// Get signed URL for conversation
async fn get_signed_url(api_key: &str, agent_id: &str, branch_id: Option<&str>) -> Result<()> {
    let client = create_http_client();
    print_info(&format!(
        "Getting signed URL for agent '{}'...",
        agent_id.cyan()
    ));

    let mut url = format!(
        "https://api.elevenlabs.io/v1/convai/conversation/get-signed-url?agent_id={}",
        agent_id
    );

    if let Some(bid) = branch_id {
        url.push_str(&format!("&branch_id={}", bid));
    }

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct SignedUrlResponse {
        signed_url: String,
    }

    let result: SignedUrlResponse = response.json().await?;
    print_success("Signed URL generated:");
    println!("{}", result.signed_url.green());

    Ok(())
}

/// Get conversation token
async fn get_conversation_token(
    api_key: &str,
    agent_id: &str,
    branch_id: Option<&str>,
) -> Result<()> {
    let client = create_http_client();
    print_info(&format!(
        "Getting conversation token for agent '{}'...",
        agent_id.cyan()
    ));

    let mut url = format!(
        "https://api.elevenlabs.io/v1/convai/conversation/token?agent_id={}",
        agent_id
    );

    if let Some(bid) = branch_id {
        url.push_str(&format!("&branch_id={}", bid));
    }

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    #[derive(Deserialize)]
    struct TokenResponse {
        token: String,
        #[serde(default)]
        expires_at: Option<String>,
    }

    let result: TokenResponse = response.json().await?;
    print_success("Conversation token generated:");
    println!("Token: {}", result.token.yellow());
    if let Some(expires) = &result.expires_at {
        println!("Expires: {}", expires);
    }

    Ok(())
}

/// WebSocket conversation with an ElevenLabs agent
async fn start_websocket_conversation(args: ConverseArgs) -> Result<()> {
    print_info(&format!(
        "Starting conversation with agent '{}'...",
        args.agent_id.cyan()
    ));
    print_info("Type your message and press Enter to send. Press Ctrl+C to exit.\n");

    // Build WebSocket URL
    let ws_url = format!(
        "wss://api.elevenlabs.io/v1/convai/conversation?agent_id={}",
        args.agent_id
    );

    // Connect to WebSocket with timeout
    let connect_future = connect_async(&ws_url);
    let (ws_stream, _) =
        tokio::time::timeout(Duration::from_secs(WS_CONNECT_TIMEOUT_SECS), connect_future)
            .await
            .context("Connection timeout")?
            .context("Failed to connect to ElevenLabs WebSocket")?;

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Send initialization message
    let init_message = json!({
        "type": "conversation_initiation_client_data",
        "conversation_config_override": {
            "agent": {
                "prompt": {}
            }
        }
    });

    ws_sender
        .send(Message::Text(init_message.to_string().into()))
        .await
        .context("Failed to send initialization message")?;

    // Create channel for input
    let (input_tx, mut input_rx) = mpsc::channel::<String>(32);

    // Spawn WebSocket sender task
    let ws_sender_handle = tokio::spawn(async move {
        while let Some(text) = input_rx.recv().await {
            let message = json!({
                "type": "user_input",
                "text": text
            });

            if ws_sender
                .send(Message::Text(message.to_string().into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Spawn WebSocket receiver task
    let ws_receiver_handle = tokio::spawn(async move {
        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    if let Ok(response) = serde_json::from_str::<ConversationResponse>(&text) {
                        match response.event_type.as_str() {
                            "conversation_initiation_metadata" => {
                                print_success("Connected to agent!");
                                if let Some(meta) = response.conversation_initiation_metadata_event
                                {
                                    print_info(&format!(
                                        "Conversation ID: {}",
                                        meta.conversation_id
                                    ));
                                }
                                println!();
                            }
                            "audio" => {
                                if let Some(audio_event) = response.audio_event {
                                    print_info(&format!(
                                        "[Agent]: {} (audio {}ms)",
                                        audio_event.event_id.cyan(),
                                        audio_event.audio_duration_ms.unwrap_or(0)
                                    ));
                                }
                            }
                            "agent_response" => {
                                if let Some(agent_event) = response.agent_response_event {
                                    if let Some(text) = agent_event.agent_response {
                                        println!("{} {}", "[Agent]:".green().bold(), text);
                                    }
                                }
                            }
                            "user_transcript" => {
                                if let Some(user_event) = response.user_transcription_event {
                                    if let Some(text) = user_event.user_transcript {
                                        println!("{} {}", "[You]:".blue().bold(), text);
                                    }
                                }
                            }
                            "ping" => {}
                            "interruption" => {
                                print_warning("Agent was interrupted");
                            }
                            "error" => {
                                if let Some(error) = response.error_event {
                                    print_error(&format!("Error: {}", error.error));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    print_warning("Connection closed by server");
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    print_error(&format!("WebSocket error: {}", e));
                    break;
                }
            }
        }
    });

    // Input loop in main thread
    println!(
        "{} Type your message and press Enter (Ctrl+C to exit)",
        "?".yellow()
    );

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let text = input.trim();
        if text.is_empty() {
            continue;
        }

        if input_tx.send(text.to_string()).await.is_err() {
            break;
        }
    }

    // Wait for tasks
    let _ = tokio::try_join!(ws_sender_handle, ws_receiver_handle);

    Ok(())
}

#[derive(Debug, Deserialize)]
struct ConversationResponse {
    #[serde(rename = "type", default)]
    event_type: String,
    #[serde(default)]
    conversation_initiation_metadata_event: Option<ConversationInitiationMetadata>,
    #[serde(default)]
    audio_event: Option<AudioEvent>,
    #[serde(default)]
    agent_response_event: Option<AgentResponseEvent>,
    #[serde(default)]
    user_transcription_event: Option<UserTranscriptEvent>,
    #[serde(default)]
    error_event: Option<ErrorEvent>,
}

#[derive(Debug, Deserialize)]
struct ConversationInitiationMetadata {
    #[serde(default)]
    conversation_id: String,
    #[allow(dead_code)]
    #[serde(default)]
    agent_output: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AudioEvent {
    #[serde(default)]
    event_id: String,
    #[allow(dead_code)]
    #[serde(default)]
    audio_base_64: Option<String>,
    #[serde(default)]
    audio_duration_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct AgentResponseEvent {
    #[serde(default)]
    agent_response: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserTranscriptEvent {
    #[serde(default)]
    user_transcript: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ErrorEvent {
    #[serde(default)]
    error: String,
}

/// Delete a conversation
async fn delete_conversation(api_key: &str, conversation_id: &str, assume_yes: bool) -> Result<()> {
    print_warning(&format!(
        "You are about to delete conversation '{}'",
        conversation_id
    ));

    if !assume_yes {
        let confirm = Confirm::new()
            .with_prompt("Are you sure?")
            .default(false)
            .interact()?;
        if !confirm {
            print_info("Cancelled");
            return Ok(());
        }
    }

    let client = create_http_client();
    print_info(&format!(
        "Deleting conversation '{}'...",
        conversation_id.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/conversations/{}",
        conversation_id
    );

    let response = client
        .delete(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to delete conversation")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    print_success(&format!(
        "Conversation '{}' deleted successfully",
        conversation_id
    ));
    Ok(())
}

/// Download conversation audio
async fn get_conversation_audio(
    api_key: &str,
    conversation_id: &str,
    output: Option<&str>,
    assume_yes: bool,
) -> Result<()> {
    let client = create_http_client();
    print_info(&format!(
        "Downloading audio for conversation '{}'...",
        conversation_id.cyan()
    ));

    let url = format!(
        "https://api.elevenlabs.io/v1/convai/conversations/{}/audio",
        conversation_id
    );

    let response = client
        .get(&url)
        .header("xi-api-key", api_key)
        .send()
        .await
        .context("Failed to download conversation audio")?;

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let audio = response
        .bytes()
        .await
        .context("Failed to read audio data")?;

    let default_filename = format!("{}.mp3", conversation_id);
    let output_path = output.unwrap_or(&default_filename);

    let path = Path::new(output_path);
    if !confirm_overwrite(path, assume_yes)? {
        print_info("Cancelled");
        return Ok(());
    }

    write_bytes_to_file(&audio, path)?;

    print_success(&format!(
        "Conversation audio downloaded -> {}",
        output_path.green()
    ));
    Ok(())
}

/// Send feedback on a conversation
async fn send_conversation_feedback(
    api_key: &str,
    conversation_id: &str,
    thumbs_up: bool,
    feedback: Option<&str>,
) -> Result<()> {
    let client = create_http_client();

    let body = if let Some(feedback_text) = feedback {
        json!({
            "score": if thumbs_up { "like" } else { "dislike" },
            "feedback": feedback_text
        })
    } else {
        json!({
            "score": if thumbs_up { "like" } else { "dislike" }
        })
    };

    let url = format!(
        "https://api.elevenlabs.io/v1/conversations/{}/feedback",
        conversation_id
    );

    let response = client
        .post(&url)
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to send conversation feedback")?;

    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let feedback_type = if thumbs_up { "liked" } else { "disliked" };
    print_success(&format!(
        "Conversation feedback submitted: {} for '{}'",
        feedback_type, conversation_id
    ));

    Ok(())
}

/// Make an outbound call via Twilio
async fn outbound_call(
    api_key: &str,
    agent_id: &str,
    caller_id: &str,
    to: &str,
    message: Option<&str>,
) -> Result<()> {
    let client = create_http_client();

    let mut body = json!({
        "agent_id": agent_id,
        "caller_id": caller_id,
        "phone_number": to,
    });

    if let Some(msg) = message {
        body["greeting_message"] = json!(msg);
    }

    let url = "https://api.elevenlabs.io/v1/conversations/outbound";

    print_info("Initiating outbound call...");

    let response = client
        .post(url)
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await
        .context("Failed to initiate outbound call")?;

    if !response.status().is_success() {
        let error = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("API error: {}", error));
    }

    let result: serde_json::Value = response.json().await.context("Failed to parse response")?;

    println!("\n{}", "Outbound Call:".bold().underline());
    println!(
        "  Conversation ID: {}",
        result["conversation_id"].as_str().unwrap_or("-").cyan()
    );
    println!(
        "  Status: {}",
        result["status"].as_str().unwrap_or("-").yellow()
    );

    print_success("Outbound call initiated successfully");

    Ok(())
}
