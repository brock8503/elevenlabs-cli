//! Basic CLI tests for ElevenLabs CLI
//!
//! These tests verify the CLI binary runs correctly.

use std::path::Path;
use std::process::Command;

/// Get the cargo binary path
fn cargo_bin() -> String {
    let debug_path = "target/debug/elevenlabs-cli";
    if Path::new(debug_path).exists() {
        return debug_path.to_string();
    }
    let release_path = "target/release/elevenlabs-cli";
    if Path::new(release_path).exists() {
        return release_path.to_string();
    }
    "elevenlabs-cli".to_string()
}

/// Run CLI with args and check exit code
fn run_cli(args: &[&str], expected_code: i32) -> bool {
    let output = Command::new(cargo_bin())
        .args(args)
        .env("ELEVENLABS_API_KEY", "test-api-key")
        .output()
        .expect("Failed to run CLI");

    output.status.code() == Some(expected_code)
}

#[test]
fn test_help_global() {
    assert!(run_cli(&["--help"], 0));
}

#[test]
fn test_help_tts() {
    assert!(run_cli(&["tts", "--help"], 0));
}

#[test]
fn test_help_stt() {
    assert!(run_cli(&["stt", "--help"], 0));
}

#[test]
fn test_help_voice() {
    assert!(run_cli(&["voice", "--help"], 0));
}

#[test]
fn test_help_models() {
    assert!(run_cli(&["models", "--help"], 0));
}

#[test]
fn test_help_agent() {
    assert!(run_cli(&["agent", "--help"], 0));
}

#[test]
fn test_help_converse() {
    assert!(run_cli(&["converse", "--help"], 0));
}

#[test]
fn test_help_knowledge() {
    assert!(run_cli(&["knowledge", "--help"], 0));
}

#[test]
fn test_help_rag() {
    assert!(run_cli(&["rag", "--help"], 0));
}

#[test]
fn test_help_workspace() {
    assert!(run_cli(&["workspace", "--help"], 0));
}

#[test]
fn test_help_phone() {
    assert!(run_cli(&["phone", "--help"], 0));
}

#[test]
fn test_help_history() {
    assert!(run_cli(&["history", "--help"], 0));
}

#[test]
fn test_help_library() {
    assert!(run_cli(&["library", "--help"], 0));
}

#[test]
fn test_help_pronunciation() {
    assert!(run_cli(&["pronunciation", "--help"], 0));
}

#[test]
fn test_help_music() {
    assert!(run_cli(&["music", "--help"], 0));
}

#[test]
fn test_help_sfx() {
    assert!(run_cli(&["sfx", "--help"], 0));
}

#[test]
fn test_help_dub() {
    assert!(run_cli(&["dub", "--help"], 0));
}

#[test]
fn test_help_webhook() {
    assert!(run_cli(&["webhook", "--help"], 0));
}

#[test]
fn test_help_dialogue() {
    assert!(run_cli(&["dialogue", "--help"], 0));
}

#[test]
fn test_help_tools() {
    assert!(run_cli(&["tools", "--help"], 0));
}

#[test]
fn test_help_projects() {
    assert!(run_cli(&["projects", "--help"], 0));
}

#[test]
fn test_help_samples() {
    assert!(run_cli(&["samples", "--help"], 0));
}

#[test]
fn test_help_isolate() {
    assert!(run_cli(&["isolate", "--help"], 0));
}

#[test]
fn test_help_voice_changer() {
    assert!(run_cli(&["voice-changer", "--help"], 0));
}

#[test]
fn test_help_voice_design() {
    assert!(run_cli(&["voice-design", "--help"], 0));
}

#[test]
fn test_help_audio_native() {
    assert!(run_cli(&["audio-native", "--help"], 0));
}

#[test]
fn test_help_usage() {
    assert!(run_cli(&["usage", "--help"], 0));
}

#[test]
fn test_help_user() {
    assert!(run_cli(&["user", "--help"], 0));
}

#[test]
fn test_help_config() {
    assert!(run_cli(&["config", "--help"], 0));
}

#[test]
fn test_help_completions() {
    assert!(run_cli(&["completions", "--help"], 0));
}

#[test]
fn test_help_interactive() {
    assert!(run_cli(&["interactive", "--help"], 0));
}

#[test]
fn test_help_update() {
    assert!(run_cli(&["update", "--help"], 0));
}

#[test]
fn test_update_is_disabled() {
    // Self-update is disabled in this fork; it must fail without touching the network
    let output = Command::new(cargo_bin())
        .args(["update", "--check"])
        .output()
        .expect("Failed to run CLI");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("self-update is disabled"));
}

#[test]
fn test_voice_fine_tune_help() {
    assert!(run_cli(&["voice", "fine-tune", "--help"], 0));
}

#[test]
fn test_voice_edit_help() {
    assert!(run_cli(&["voice", "edit", "--help"], 0));
}

#[test]
fn test_voice_share_help() {
    assert!(run_cli(&["voice", "share", "--help"], 0));
}

#[test]
fn test_voice_similar_help() {
    assert!(run_cli(&["voice", "similar", "--help"], 0));
}

#[test]
fn test_history_feedback_help() {
    assert!(run_cli(&["history", "feedback", "--help"], 0));
}

#[test]
fn test_models_rates_help() {
    assert!(run_cli(&["models", "rates", "--help"], 0));
}

#[test]
fn test_pronunciation_rules_help() {
    assert!(run_cli(&["pronunciation", "rules", "--help"], 0));
}

#[test]
fn test_pronunciation_add_rules_help() {
    assert!(run_cli(&["pronunciation", "add-rules", "--help"], 0));
}

#[test]
fn test_rag_rebuild_help() {
    assert!(run_cli(&["rag", "rebuild", "--help"], 0));
}

#[test]
fn test_workspace_secrets_help() {
    assert!(run_cli(&["workspace", "secrets", "--help"], 0));
}

#[test]
fn test_phone_test_help() {
    assert!(run_cli(&["phone", "test", "--help"], 0));
}

#[test]
fn test_converse_delete_help() {
    assert!(run_cli(&["converse", "delete", "--help"], 0));
}

#[test]
fn test_converse_audio_help() {
    assert!(run_cli(&["converse", "audio", "--help"], 0));
}

#[test]
fn test_audio_native_list_help() {
    assert!(run_cli(&["audio-native", "list", "--help"], 0));
}

#[test]
fn test_audio_native_get_help() {
    assert!(run_cli(&["audio-native", "get", "--help"], 0));
}

#[test]
fn test_invalid_command_fails() {
    // Invalid command should fail
    assert!(!run_cli(&["nonexistent-command"], 0));
}

#[test]
fn test_tts_requires_text() {
    // TTS without text should fail
    assert!(!run_cli(&["tts"], 0));
}

#[test]
fn test_stt_requires_file() {
    // STT without file should fail
    assert!(!run_cli(&["stt"], 0));
}
