//! Real-time TTS using WebSocket API for ultra-low latency streaming
//!
//! This implements ElevenLabs' WebSocket TTS API for streaming audio with minimal latency.
//! The WebSocket API allows sending text incrementally and receiving audio chunks in real-time.

use crate::cli::RealtimeTtsArgs;
use crate::output::print_info;

#[cfg(feature = "audio")]
use crate::audio::audio_io;

use anyhow::Result;
use colored::*;

pub async fn execute(args: RealtimeTtsArgs, _api_key: &str, _assume_yes: bool) -> Result<()> {
    if args.text.is_empty() {
        return Err(anyhow::anyhow!("Text cannot be empty"));
    }

    print_info(&format!(
        "Real-time TTS using voice '{}'...",
        args.voice.cyan()
    ));

    #[cfg(not(feature = "ws"))]
    {
        return Err(anyhow::anyhow!(
            "WebSocket feature not enabled. Build with --features ws to enable realtime-tts."
        ));
    }

    #[cfg(feature = "ws")]
    {
        execute_ws_tts(args, _api_key, _assume_yes).await
    }
}

#[cfg(feature = "ws")]
async fn execute_ws_tts(args: RealtimeTtsArgs, api_key: &str, assume_yes: bool) -> Result<()> {
    use crate::output::print_success;
    use crate::utils::{confirm_overwrite, generate_output_filename, write_bytes_to_file};
    use anyhow::Context;
    use futures_util::{SinkExt, StreamExt};
    use std::time::Duration;
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

    // Build WebSocket URL with API key
    let base_url = "wss://api.elevenlabs.io/v1/text-to-speech";
    let ws_url = format!(
        "{}/{}/stream-input?model_id={}&xi-api-key={}",
        base_url, args.voice, args.model, api_key
    );

    print_info("Connecting to ElevenLabs WebSocket API...");
    print_info(&format!("Voice: {}, Model: {}", args.voice, args.model));

    // Connect to WebSocket
    let (ws_stream, _response) =
        tokio::time::timeout(Duration::from_secs(30), connect_async(&ws_url))
            .await
            .context("Connection timeout")?
            .context("Failed to connect to ElevenLabs WebSocket")?;

    print_info("Connected! Streaming audio...");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Send BOS (Beginning of Stream) message
    let bos = serde_json::json!({
        "text_type": "text",
        "EOS": false
    });

    ws_sender
        .send(Message::Text(bos.to_string().into()))
        .await
        .context("Failed to send BOS message")?;

    // Send the text
    let text_msg = serde_json::json!({
        "text": args.text,
        "EOS": true
    });

    ws_sender
        .send(Message::Text(text_msg.to_string().into()))
        .await
        .context("Failed to send text message")?;

    // Collect audio chunks in real-time
    let mut audio_chunks = Vec::new();
    let mut chunk_count = 0;

    // Play audio in real-time if requested
    #[cfg(feature = "audio")]
    let player = if args.play {
        match audio_io::StreamingPlayer::new() {
            Ok(p) => Some(p),
            Err(e) => {
                print_info(&format!("Warning: could not create audio player: {}", e));
                None
            }
        }
    } else {
        None
    };

    while let Some(msg_result) = ws_receiver.next().await {
        match msg_result {
            Ok(Message::Binary(data)) => {
                chunk_count += 1;
                audio_chunks.extend_from_slice(&data);

                // Stream to speaker in real-time if requested
                #[cfg(feature = "audio")]
                if let Some(ref p) = player {
                    if let Err(e) = p.send_chunk(&data) {
                        print_info(&format!("Warning: could not send chunk to player: {}", e));
                    }
                }

                // Print progress for first few chunks
                if chunk_count <= 3 {
                    print_info(&format!(
                        "Received chunk {} ({} bytes)",
                        chunk_count,
                        data.len()
                    ));
                }
            }
            Ok(Message::Text(text)) => {
                // Check for server messages (like errors or completion)
                if let Ok(response) = serde_json::from_str::<serde_json::Value>(&text) {
                    // Check for errors
                    if let Some(error) = response.get("error") {
                        return Err(anyhow::anyhow!("API error: {}", error));
                    }

                    // Check for completion
                    if let Some(is_final) = response.get("is_final").and_then(|v| v.as_bool()) {
                        if is_final {
                            print_info("Stream complete");
                            break;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                print_info("Connection closed by server");
                break;
            }
            Err(e) => {
                print_info(&format!("WebSocket error: {}", e));
                break;
            }
            _ => {}
        }
    }

    // Finish streaming playback
    #[cfg(feature = "audio")]
    if let Some(p) = player {
        if let Err(e) = p.finish() {
            print_info(&format!("Warning: error finishing playback: {}", e));
        }
    }

    if audio_chunks.is_empty() {
        return Err(anyhow::anyhow!("No audio data received"));
    }

    print_success(&format!(
        "Received {} chunks ({} bytes total)",
        chunk_count,
        audio_chunks.len()
    ));

    // Determine output path
    let output_path = if let Some(output) = args.output {
        output
    } else {
        let extension = match args.output_format.as_str() {
            f if f.starts_with("mp3_") => "mp3",
            f if f.starts_with("pcm_") => "wav",
            f if f.starts_with("ulaw_") => "ulaw",
            f if f.starts_with("opus_") => "opus",
            _ => "mp3",
        };
        generate_output_filename("realtime_tts", extension)
    };

    // Check for overwrite
    let path = std::path::Path::new(&output_path);
    if !confirm_overwrite(path, assume_yes)? {
        print_info("Cancelled");
        return Ok(());
    }

    // Write audio file
    write_bytes_to_file(&audio_chunks, path)?;
    print_success(&format!("Saved -> {}", output_path.green()));

    Ok(())
}
