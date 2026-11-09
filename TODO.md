# TODO

Open work for Voximov. This project was detached from arconw/simpleVoiceover on 2026-10-30 and no longer syncs with it.

## AI voiceover engine fixes

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

## ElevenLabs as a second voice provider

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

## Smaller items

- Use the native macOS traffic-light window buttons (overlay title bar) and hide the custom Windows-style window controls on macOS.
- Release workflow: build and attach the macOS `.dmg` (on a macOS runner, as `bin/build-macos` does), since macOS is now the primary platform. It currently builds only the Windows executable.
- Untested: `scripts/kokoro.ps1` on Windows.
- Script progress is estimated from elapsed time. Kokoro-FastAPI sends each clip's audio in one burst at the end.
- A webview reload during generation re-enables Play and Record while the engine may still be working.
- Partially selecting a script clip still reaches the engine and shows its error instead of being prevented in the UI.
- Now possible without sync conflicts: shorten the README inherited from upstream, and move the `bin/` scripts into `scripts/`.
