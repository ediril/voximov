# TODO

Open work for Voximov. This project was detached from arconw/simpleVoiceover on 2026-10-30 and no longer syncs with it.

## Next: AI-made demo videos

Goal: an AI agent records a feature demo, cuts it, and narrates it end to end through a Voximov CLI, with no human recording. Each step is useful on its own; do them in order.

Every feature works from the app first: recording, marks, cutting idle time and anchoring are buttons and menus in the UI. The CLI drives the same engine commands for AI agents, so the app and the CLI never get separate implementations.

1. **Kokoro inside the engine, no Docker.**
   - Run the Kokoro-82M ONNX model in the Rust engine with the `ort` crate (ONNX Runtime, CoreML on macOS).
   - Phonemize with `misaki-rs` with default features off. Do not use espeak-ng: it is GPL-3 and cannot ship in this app.
   - Download the model and voices on first use, with progress, into the app data folder. Do not bundle them in the dmg.
   - Unknown words: spell them out first. Per-project pronunciation overrides can come later.
   - Generate sentence by sentence, for real progress and a Stop that takes effect after the current sentence.
   - Remove `scripts/kokoro.sh`, `scripts/kokoro.ps1`, the server address setting, the "server not ready" toast and the Docker steps in the README.
   - This replaces most of the HTTP-related engine fixes below (stalled reads, timeouts, TLS, the voice-list pre-check). Recheck that list afterwards.
2. **Headless CLI.**
   - A `voximov` command over the existing `studio_command` layer, with JSON output for agents.
   - Commands: create and open projects, import media, add script clips, generate, and export audio and video.
   - Even before recording exists, an agent can narrate a video recorded by hand.
3. **Window recording with marks.**
   - Record one window (for example the browser the agent drives) or a display with ScreenCaptureKit (`screencapturekit` crate), with the cursor and optional system audio.
   - `voximov record start --window <name>`, `record mark <label>`, `record stop`. Marks are timestamped markers on the timeline. In the app: a Record screen button with a window picker, and a shortcut to add a mark while recording.
   - The CLI asks the running app to record, so the Screen Recording permission belongs to Voximov, not to the terminal or agent.
   - Not chosen: `screencapture -v` (whole screen only, stops on a key press) and ffmpeg's avfoundation input (needs a rebuilt ffmpeg, whole displays, drops frames).
4. **Cut idle time and anchor narration to marks.**
   - Agents pause 5 to 30 seconds between actions. Remove idle stretches using the marks or unchanged frames, keeping some padding around each action. In the app: a Remove idle time action with the padding setting.
   - Anchor script clips to marks (`script add --at-mark <label>`, or by dropping a script clip onto a mark in the app), so narration stays in sync after cuts.

After step 4: write a tutorial for terminal app demos with asciinema. Record with `asciinema rec` (its `--idle-time-limit` already shortens pauses), turn the `.cast` into a video (`agg` makes a GIF, then ffmpeg converts it to MP4, or replay it in a terminal window and record that window with step 3), then import it into Voximov and add narration. Consider importing `.cast` files directly, using their timed events as marks.

To consider later: cursor polish (smoothing, click highlights, zoom on clicks), an MCP server over the same commands, and Playwright's own video recording for browser-only demos (no screen permission, but no visible cursor).

## Later

### AI voiceover engine fixes

A three-model review of the voiceover engine (`backend/src/generator.rs`, `speech.rs`, `desktop.rs`, `model.rs`) found these. The UI blocks Play and Record while clips generate, which hides the worst of item 1 for now.

1. **Generation bypasses the engine's edit rules.**
   - A take that finishes during playback or recording changes the project without stopping playback. The mixer then fails with `error.sourceCacheMissing` when it reaches that clip.
   - Generation ignores track locks.
   - Saving the Kokoro settings (`speech_preferences_patch`) stops playback, because the command is missing from the controller's settings allow-list.

   Fix: refuse Play and Record in the engine while anything generates, and refuse new generation while playing or recording. Commit takes through the same path as other edits, and respect locks.

2. **The queue and Stop are fragile.**
   - There are no read timeouts. A Kokoro server that accepts a request and then stalls freezes the single worker forever, and Stop cannot interrupt it.
   - `Queued`, `Generating` and `Cancelled` events can arrive out of order.
   - Jobs are keyed by track and clip. Moving a clip to another track loses its take.
   - Jobs are not tied to a project, so after Discard or opening another project a late take can attach to the wrong project.
   - The voice-list pre-check in `script_generate` adds a blocking request and a Stop race.

   Fix: key jobs by clip and project id, add response and body timeouts, and make Stop interrupt a stalled read. Publish events in order, clear the queue when the project changes, and drop the pre-check.

3. **Attaching a take is not all-or-nothing.**
   - A failed `store.replace` leaves media files behind and the project marked dirty.
   - The final store lock covers the whole decode.
   - The whole response is buffered twice in memory, up to a 2 GB limit.

   Fix: stream to a temporary file, decode outside the lock, and lock only to install the result. Use one transactional import helper for import and generation.

4. **Validation gaps.**
   - Script clip length has no upper limit, so a huge value can hang export.
   - Older app versions can open projects with generated script clips and drop the script text on save. Bump the project version.
   - `project.voice` is never set, so the voice follows the Settings default.
   - `https://` addresses are accepted, but the HTTP client has no TLS.
   - Error details are not localized ("empty", "voices", raw status codes).
   - Voice and text limits differ between entry points.
5. **Missing tests.**
   - Editing while a clip generates.
   - Moving or deleting the clip mid-generation.
   - Playing while a take lands.
   - The rollback paths.

Smaller items: catch panics in the worker thread, and start the worker only in the desktop build. This also removes the "field `generator` is never read" warning.

### ElevenLabs as a second voice provider

Do the engine fixes first. A remote, paid service makes the timeout and cancellation gaps worse.

- Add a provider choice to `speech.rs`, Kokoro or ElevenLabs, with the same "list voices" and "stream speech" operations. Store voices with their provider, such as `kokoro:af_bella` or `elevenlabs:<voice id>`. Record the provider in each take.
- Enable TLS in `ureq` (`rustls`).
- Settings: a provider choice, an API key field with a connection test, and the voice menu showing ElevenLabs voices.
- Handle an invalid key, exhausted quota, rate limits and network drops.
- Docs: script text leaves the computer and is billed per character. Say so in the README and in Settings.
- Add a fake ElevenLabs server for tests.

Decisions needed:

- Where to keep the API key: the Keychain (recommended, via the `keyring` crate) or `settings.json`.
- One provider for the whole app, or per project.
- Verify the API details against the ElevenLabs docs before building: `POST /v1/text-to-speech/{voice_id}/stream`, the `xi-api-key` header, PCM output formats, and `GET /v1/voices`.

### Smaller items

- Use the native macOS traffic-light window buttons (overlay title bar) and hide the custom Windows-style window controls on macOS.
- Release workflow: build and attach the macOS `.dmg` (on a macOS runner, as `bin/build-macos` does), since macOS is now the primary platform. It currently builds only the Windows executable.
- Untested: `scripts/kokoro.ps1` on Windows.
- Script progress is estimated from elapsed time. Kokoro-FastAPI sends each clip's audio in one burst at the end.
- A webview reload during generation re-enables Play and Record while the engine may still be working.
- Partially selecting a script clip still reaches the engine and shows its error instead of being prevented in the UI.
- Now possible without sync conflicts: shorten the README inherited from upstream, and move the `bin/` scripts into `scripts/`.
