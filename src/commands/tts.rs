use crate::cli::TextToSpeechArgs;
use crate::output::{print_info, print_success};
use crate::utils::{
    confirm_overwrite, format_to_extension, generate_output_filename, get_input_text,
    parse_output_format, validate_text_length, write_bytes_to_file,
};
use crate::validation::validate_voice_settings;
use anyhow::Result;
use colored::*;
use elevenlabs_rs::{
    endpoints::genai::tts::{TextToSpeech, TextToSpeechBody, TextToSpeechQuery},
    ElevenLabsClient, Model, VoiceSettings,
};
use std::path::Path;

/// Premade "Brian" voice. The API takes voice IDs, not names.
pub const DEFAULT_VOICE_ID: &str = "nPczCjzI2devNBz1zQrb";

/// The API accepts speeds from 0.7 to 1.2.
pub fn validate_speed(speed: f32) -> Result<()> {
    if !(0.7..=1.2).contains(&speed) {
        anyhow::bail!("speed must be between 0.7 and 1.2 (got {})", speed);
    }
    Ok(())
}

pub async fn execute(
    args: TextToSpeechArgs,
    api_key: &str,
    output_format: &str,
    assume_yes: bool,
) -> Result<()> {
    let voice = args
        .voice
        .as_deref()
        .unwrap_or(DEFAULT_VOICE_ID)
        .to_string();
    let voice = voice.as_str();

    // Get input text
    let text = get_input_text(args.text, args.file)?;

    // Validate text length
    validate_text_length(&text)?;

    // Create client
    let client = ElevenLabsClient::new(api_key);

    // Parse model
    let model = parse_model(&args.model);

    // Parse output format
    let format = parse_output_format(output_format)?;

    // Validate voice settings using validation module
    validate_voice_settings(args.stability, args.similarity_boost, args.style)?;
    if let Some(sp) = args.speed {
        validate_speed(sp)?;
    }

    // Build voice settings if any provided
    let voice_settings = if args.stability.is_some()
        || args.similarity_boost.is_some()
        || args.style.is_some()
        || args.speaker_boost
        || args.speed.is_some()
    {
        let mut settings = VoiceSettings::default();
        if let Some(s) = args.stability {
            settings = settings.with_stability(s);
        }
        if let Some(sb) = args.similarity_boost {
            settings = settings.with_similarity_boost(sb);
        }
        if let Some(st) = args.style {
            settings = settings.with_style(st);
        }
        if let Some(sp) = args.speed {
            // 1.2f32 serializes as 1.2000000476, which the API rejects as > 1.2
            settings = settings.with_speed(sp.min(1.199_999_9));
        }
        settings = settings.use_speaker_boost(args.speaker_boost);
        Some(settings)
    } else {
        None
    };

    // Build request body
    let mut body = TextToSpeechBody::new(&text).with_model_id(model);

    if let Some(settings) = voice_settings {
        body = body.with_voice_settings(settings);
    }

    if let Some(lang) = &args.language {
        body = body.with_language_code(lang);
    }

    if let Some(seed) = args.seed {
        body = body.with_seed(seed as u64);
    }

    print_info(&format!(
        "Generating speech with voice '{}'...",
        voice.cyan()
    ));
    print_info(&format!("Model: {}", args.model.yellow()));
    print_info(&format!("Characters: {}", text.len().to_string().yellow()));

    // Build query with output format
    let query = TextToSpeechQuery::default().with_output_format(format);

    // Create endpoint
    let endpoint = TextToSpeech::new(voice, body).with_query(query);

    // Generate speech
    let start_time = std::time::Instant::now();
    let audio = client.hit(endpoint).await.map_err(|e| anyhow::anyhow!(e))?;
    let duration = start_time.elapsed();

    // Determine output path
    let output_path = args
        .output
        .unwrap_or_else(|| generate_output_filename("speech", format_to_extension(output_format)));

    // Check for overwrite
    let path = Path::new(&output_path);
    if !confirm_overwrite(path, assume_yes)? {
        print_info("Cancelled");
        return Ok(());
    }

    // Write audio file
    write_bytes_to_file(&audio, path)?;

    print_success(&format!(
        "Generated speech in {:.2}s -> {}",
        duration.as_secs_f64(),
        output_path.green()
    ));

    // Play audio if requested
    if args.play {
        print_info("Playing audio...");
        elevenlabs_rs::utils::play(audio).map_err(|e| anyhow::anyhow!(e))?;
    }

    Ok(())
}

fn parse_model(model: &str) -> Model {
    match model {
        "eleven_multilingual_v2" => Model::ElevenMultilingualV2,
        "eleven_flash_v2_5" => Model::ElevenFlashV2_5,
        "eleven_turbo_v2" => Model::ElevenTurboV2,
        "eleven_turbo_v2_5" => Model::ElevenTurboV2_5,
        "eleven_english_sts_v2" => Model::ElevenEnglishV2,
        "eleven_multilingual_sts_v2" => Model::ElevenMultilingualV2STS,
        "eleven_flash_v2" => Model::ElevenFlashV2,
        "eleven_v3" => Model::ElevenMultilingualV2,
        _ => Model::ElevenMultilingualV2,
    }
}
