---
name: tts
description: Generate speech audio from text with the ElevenLabs CLI (`elevenlabs-cli`, the hardened fork at ~/Development/elevenlabs-cli). TRIGGER when the user says "text to speech", "tts", "read this aloud", "voiceover", "narrate", "generate audio/voice", or mentions ElevenLabs. DO NOT TRIGGER for transcription (speech to text) unless they ask for ElevenLabs STT.
---

# Text to speech with ElevenLabs

Run `elevenlabs-cli` (installed to `~/.cargo/bin`). It reads `ELEVENLABS_API_KEY`, which `~/.custom` exports from macOS Keychain (service `elevenlabs-api-key`). Never pass the key with `-a`, never echo it, never write it to a file or `config set`.

If the CLI reports a missing key, ask the user to run:
`! security add-generic-password -a "$USER" -s elevenlabs-api-key -w`
then open a new shell.

## Common commands

```bash
elevenlabs-cli tts "Hello there" -o ~/Development/scratch/tts/hello.mp3
elevenlabs-cli tts -i script.md -o ~/Development/scratch/tts/script.mp3 --voice <VOICE_ID>
elevenlabs-cli tts "Hello" -o out.mp3 --play  # plays on your ssh/mosh client if remote
elevenlabs-cli play out.mp3 [--local]
elevenlabs-cli voice list -j                 # find voice IDs
```

- Default output folder: `~/Development/scratch/tts/` (create it if missing). Name files after the content, e.g. `2026-11-03-tdr-intro.mp3`.
- Speed: `--speed 0.7-1.2`, else `default_speed` from config (user default 1.15).
- Voice: `--voice <ID>`, else `default_voice` from config, else Brian (`nPczCjzI2devNBz1zQrb`). Always pass voice IDs, never names (names return `voice_not_found`). Set the default with `elevenlabs-cli config set default_voice <ID>`. Ask before switching to a paid or cloned voice.
- Picking a voice: `elevenlabs-cli voice list -j` needs the key's **Voices: Read** permission. Without it, generate short samples per premade voice ID into `~/Development/scratch/tts/voice-samples/` and let the user listen.
- Long scripts cost credits (billed per character). For text over ~5,000 characters, tell the user the character count and confirm before generating.
- Strip markdown syntax (headings, bullets, links) from text before sending, or it gets read aloud.
- Play with `--play` on `tts` (run it in the background), or `elevenlabs-cli play <file>` for an existing file. Never use bare `afplay`. Playback goes where the user is: if their most recently active tmux client came in over ssh or mosh, the CLI copies the file there with scp and plays it (afplay, mpv, ffplay or paplay). Otherwise it plays locally. `--local` forces local playback; `--remote <host>` picks a host.
- Remote playback needs key-based ssh from this Mac to the client machine. The ssh user comes from `config set remote_user <name>`, `--remote-user`, or ssh config. On failure the CLI prints why and plays locally; relay the reason. A phone client (iOS mosh) has no ssh server, so say audio played on the host and give the file path.

## Auth errors

The CLI prints a fix hint after the raw error. `api_key_id_used_as_api_key`: the user stored the key ID; they need the `sk_` secret. `missing_permissions`: the restricted key lacks the named permission; enable it in the dashboard. Either way the key is updated with `! security add-generic-password -U -a "$USER" -s elevenlabs-api-key -w` and then a new shell. See the README's Authentication section.

## Safety notes (hardened fork)

- `elevenlabs-cli update` is disabled. Update with `git -C ~/Development/elevenlabs-cli pull && cargo install --path ~/Development/elevenlabs-cli --locked`.
- Do not build with `--features mcp`: its file-reading tools can exfiltrate local files.
- Avoid `realtime-tts`; it puts the key in the WebSocket URL.
