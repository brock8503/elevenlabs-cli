<h1 align="center">ElevenLabs CLI</h1>

<p align="center">
  The community-built command line for ElevenLabs AI audio workflows.
</p>

<p align="center">
  <a href="https://github.com/hongkongkiwi/elevenlabs-cli/releases">
    <img src="https://img.shields.io/github/v/release/hongkongkiwi/elevenlabs-cli?style=for-the-badge&logo=github&label=release" alt="GitHub Release" />
  </a>
  <a href="https://lib.rs/crates/elevenlabs-cli">
    <img src="https://img.shields.io/crates/v/elevenlabs-cli?style=for-the-badge&logo=rust" alt="Crates.io Version" />
  </a>
  <a href="https://lib.rs/crates/elevenlabs-cli">
    <img src="https://img.shields.io/crates/d/elevenlabs-cli?style=for-the-badge&logo=rust" alt="Crates.io Downloads" />
  </a>
  <a href="https://github.com/hongkongkiwi/elevenlabs-cli/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/hongkongkiwi/elevenlabs-cli/ci.yml?branch=main&style=for-the-badge&logo=github&label=ci" alt="CI" />
  </a>
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/license-MIT-3DA639?style=for-the-badge" alt="MIT License" />
  </a>
</p>

<p align="center">
  <a href="https://github.com/hongkongkiwi/action-elevenlabs-cli/blob/main/README.md">
    <img src="https://img.shields.io/badge/github-action-2088FF?style=flat-square&logo=githubactions&logoColor=white" alt="GitHub Action" />
  </a>
  <a href="https://github.com/hongkongkiwi/homebrew-elevenlabs-cli/blob/main/README.md">
    <img src="https://img.shields.io/badge/homebrew-tap-FBB040?style=flat-square&logo=homebrew&logoColor=black" alt="Homebrew Tap" />
  </a>
  <a href="https://github.com/hongkongkiwi/scoop-elevenlabs-cli/blob/main/README.md">
    <img src="https://img.shields.io/badge/scoop-bucket-4A89DC?style=flat-square" alt="Scoop Bucket" />
  </a>
  <a href="https://github.com/hongkongkiwi/elevenlabs-cli/pkgs/container/elevenlabs-cli">
    <img src="https://img.shields.io/badge/docker-ghcr-2496ED?style=flat-square&logo=docker&logoColor=white" alt="Docker GHCR" />
  </a>
  <a href="https://github.com/hongkongkiwi/elevenlabs-cli-skill/blob/main/README.md">
    <img src="https://img.shields.io/badge/ai%20skill-clawhub-7A5AF8?style=flat-square" alt="ClawHub Skill" />
  </a>
</p>

<p align="center">
  <a href="#installation">Install</a>
  ·
  <a href="#quick-start">Quick Start</a>
  ·
  <a href="#common-workflows">Workflows</a>
  ·
  <a href="#mcp-server-mode">MCP</a>
  ·
  <a href="#ecosystem">Ecosystem</a>
  ·
  <a href="#support">Support</a>
</p>

> [!WARNING]
> This is an independent, community-built CLI and is not an official ElevenLabs release. For official platform docs, visit [elevenlabs.io/docs](https://elevenlabs.io/docs).

## Why ElevenLabs CLI

- Unified interface for TTS, STT, voice cloning, dubbing, and audio tooling
- Script-friendly output (`--json`) for automation and CI pipelines
- MCP server mode for AI assistants with tool filtering and safety controls
- Multi-channel distribution (Homebrew, Scoop, Cargo, Docker, source)

## Installation

| Channel | Command |
| --- | --- |
| Homebrew (macOS/Linux) | `brew tap hongkongkiwi/elevenlabs-cli && brew install elevenlabs-cli` |
| Scoop (Windows) | `scoop bucket add elevenlabs-cli https://github.com/hongkongkiwi/scoop-elevenlabs-cli && scoop install elevenlabs-cli` |
| Cargo (all platforms) | `cargo install elevenlabs-cli` |
| Cargo with MCP feature | `cargo install elevenlabs-cli --features mcp` |
| Docker | `docker run --rm -e ELEVENLABS_API_KEY=your-key ghcr.io/hongkongkiwi/elevenlabs-cli tts "Hello"` |
| From source | `git clone https://github.com/hongkongkiwi/elevenlabs-cli.git && cd elevenlabs-cli && cargo install --path .` |

## Quick Start

```bash
# 1) Set API key (or use: elevenlabs-cli config set api_key "..." )
export ELEVENLABS_API_KEY="your-api-key"

# 2) Generate speech
elevenlabs-cli tts "Hello from ElevenLabs CLI" --output hello.mp3

# 3) List voices
elevenlabs-cli voice list

# 4) Transcribe audio
elevenlabs-cli stt audio.mp3
```

> [!TIP]
> Examples use `elevenlabs-cli` (the default binary name). If you want a shorter command, add an alias such as `alias elevenlabs='elevenlabs-cli'`.

## Authentication

The CLI reads your key from `ELEVENLABS_API_KEY` (or `--api-key`, or `api_key` in the config file). Create a key at [elevenlabs.io/app/settings/api-keys](https://elevenlabs.io/app/settings/api-keys).

### Store the key in macOS Keychain (recommended)

Keep the key out of shell history and plaintext files:

```bash
# Store or replace the key (-U updates an existing entry). It prompts for the key.
security add-generic-password -U -a "$USER" -s elevenlabs-api-key -w

# Load it in your shell profile (e.g. ~/.bash_profile or ~/.zshrc)
ELEVENLABS_API_KEY="$(security find-generic-password -a "$USER" -s elevenlabs-api-key -w 2>/dev/null)"
export ELEVENLABS_API_KEY
```

Open a new shell (or re-source your profile) after changing the key, because running shells keep the old value.

If you use `config set api_key` instead, the config file is written with owner-only permissions (`0600`).

### Troubleshooting authentication errors

When the API rejects a key, the CLI prints the raw error and then a fix hint. The common cases:

| API `status` | What it means | Fix |
| --- | --- | --- |
| `api_key_id_used_as_api_key` | You pasted the key **ID** from the dashboard key list, not the secret. | Use the secret key, which starts with `sk_`. It is shown only once, when the key is created. If you lost it, rotate the key or create a new one, then update Keychain with the `-U` command above. |
| `missing_permissions` | The key is a restricted key and lacks the permission named in the message (e.g. `voices_read`). Other commands may still work. | Edit the key in the dashboard and enable that permission, or use a key with broader access. |
| `invalid_api_key` | The key is unknown, revoked, or mistyped. | Check it in the dashboard (it should start with `sk_`) and update Keychain. |

Restricted keys are a good default: grant only what you use. `tts` needs **Text to Speech**. `voice list` needs **Voices: Read**.

## Feature Surface

| Area | Commands |
| --- | --- |
| Speech | `tts`, `tts-stream`, `tts-timestamps`, `realtime-tts`, `stt` |
| Voice | `voice`, `voice-changer`, `voice-design`, `library`, `samples`, `pronunciation` |
| Content | `sfx`, `dialogue`, `music`, `dub`, `isolate`, `audio-native` |
| Agent Platform | `agent`, `converse`, `tools`, `projects`, `knowledge`, `rag`, `workspace` |
| Platform Ops | `history`, `usage`, `models`, `user`, `config`, `webhook`, `update`, `interactive` |
| Developer UX | `completions`, global `--json`, optional `mcp` mode |

## Common Workflows

### Text to Speech

```bash
elevenlabs-cli tts "Ship high-quality audio from the terminal" --voice Brian --output narration.mp3
```

### Speech to Text with Speaker Diarization

```bash
elevenlabs-cli stt meeting.mp3 --diarize --num-speakers 3
```

### Voice Cloning

```bash
elevenlabs-cli voice clone --name "My Voice" --samples voice1.mp3,voice2.mp3
```

### Sound Effects

```bash
elevenlabs-cli sfx "A soft UI notification" --duration 3 --output notification.mp3
```

### JSON for Automation

```bash
elevenlabs-cli --json voice list | jq '.[0].voice_id'
```

## MCP Server Mode

Start the server:

```bash
elevenlabs-cli mcp
```

> [!NOTE]
> `mcp` is available in builds that include the `mcp` feature (for example, `cargo install elevenlabs-cli --features mcp`).

Minimal client config:

```json
{
  "mcpServers": {
    "elevenlabs": {
      "command": "elevenlabs-cli",
      "args": ["mcp"],
      "env": {
        "ELEVENLABS_API_KEY": "your-api-key"
      }
    }
  }
}
```

Security options:

```bash
elevenlabs-cli mcp --enable-tools tts,stt,voice
elevenlabs-cli mcp --disable-admin
elevenlabs-cli mcp --disable-destructive
elevenlabs-cli mcp --read-only
```

## Configuration

Default config path: `~/.config/elevenlabs-cli/config.toml`

```toml
api_key = "your-api-key"
default_voice = "Brian"
default_model = "eleven_multilingual_v2"
default_output_format = "mp3_44100_128"

[mcp]
enable_tools = "tts,stt"
disable_tools = "agents"
disable_admin = false
read_only = false
```

## Available Models

| Model | Best For |
| --- | --- |
| `eleven_multilingual_v2` | High quality synthesis across many languages |
| `eleven_flash_v2_5` | Lowest latency |
| `eleven_turbo_v2_5` | Balanced speed and quality |
| `eleven_v3` | Expressive and emotional speech |
| `scribe_v1` | Higher-accuracy speech-to-text |
| `scribe_v1_base` | Faster, lower-cost speech-to-text |

## Output Formats

| Format | Use Case |
| --- | --- |
| `mp3_44100_128` | Default, broadly compatible |
| `mp3_44100_192` | Higher quality MP3 |
| `wav_44100` | Editing and mastering |
| `pcm_16000` | Telephony and real-time systems |
| `opus_48000_128` | Streaming and WebRTC |
| `ulaw_8000` | Legacy telephony |

## Command Reference

Use `elevenlabs-cli --help` and `elevenlabs-cli <command> --help` for details.

| Command | Description |
| --- | --- |
| `tts` | Text-to-speech synthesis |
| `stt` | Speech-to-text transcription |
| `voice` | Voice management (list, clone, edit, delete) |
| `library` | Shared/community voice library |
| `isolate` | Background-noise removal |
| `voice-changer` | Speech-to-speech voice transformation |
| `dub` | Dubbing and translation workflows |
| `dialogue` | Multi-speaker dialogue generation |
| `sfx` | Sound effect generation |
| `music` | Music generation |
| `agent`, `converse`, `tools`, `projects` | Conversational/agent features |
| `knowledge`, `rag`, `workspace` | Knowledge and workspace management |
| `history`, `usage`, `models`, `user` | Account/model/runtime information |
| `config`, `webhook`, `update`, `interactive`, `completions` | Tooling and operations |
| `mcp` | Model Context Protocol server mode |

## Ecosystem

Companion repositories and their README files:

| Repository | Purpose | README |
| --- | --- | --- |
| [hongkongkiwi/action-elevenlabs-cli](https://github.com/hongkongkiwi/action-elevenlabs-cli) | GitHub Action wrapper for CI workflows | [Open](https://github.com/hongkongkiwi/action-elevenlabs-cli/blob/main/README.md) |
| [hongkongkiwi/homebrew-elevenlabs-cli](https://github.com/hongkongkiwi/homebrew-elevenlabs-cli) | Homebrew tap for releases | [Open](https://github.com/hongkongkiwi/homebrew-elevenlabs-cli/blob/main/README.md) |
| [hongkongkiwi/scoop-elevenlabs-cli](https://github.com/hongkongkiwi/scoop-elevenlabs-cli) | Scoop bucket for Windows installs | [Open](https://github.com/hongkongkiwi/scoop-elevenlabs-cli/blob/main/README.md) |
| [hongkongkiwi/elevenlabs-cli-skill](https://github.com/hongkongkiwi/elevenlabs-cli-skill) | AI-agent skill integration | [Open](https://github.com/hongkongkiwi/elevenlabs-cli-skill/blob/main/README.md) |

## Resources

- [ElevenLabs API Reference](https://elevenlabs.io/docs/api-reference)
- [ElevenLabs Documentation](https://elevenlabs.io/docs)
- [Voice Library](https://elevenlabs.io/app/voice-library)
- [API Keys](https://elevenlabs.io/app/settings/api-keys)
- [Status Page](https://status.elevenlabs.io)

## Support

- [Report a bug](https://github.com/hongkongkiwi/elevenlabs-cli/issues)
- [Request a feature](https://github.com/hongkongkiwi/elevenlabs-cli/issues)
- [Releases](https://github.com/hongkongkiwi/elevenlabs-cli/releases)

## Contributing

Contributions are welcome.

1. Fork the repository.
2. Create a feature branch.
3. Run tests and checks.
4. Open a pull request with context and examples.

## License

MIT License. See [LICENSE](LICENSE).
