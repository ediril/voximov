# Voximov

Voximov is a desktop app for recording and editing a voiceover over a video. Video audio,
your voice and music each sit on their own track, so you can cut, move and process them without
touching the originals. A Rust engine decodes, processes, mixes and exports all audio itself. On
macOS, a bundled FFmpeg turns the edited video and the mix into an MP4. Everything runs locally on
your computer.

## Who is it for?

Voximov is for people who narrate their own videos: tutorials, game footage, product
demos, or anything else that needs a voice on top of the picture.

If your current process is "record in one app, line it up in another, and hope the levels match,"
Voximov keeps recording, editing and export in one window.

It works best for a single narrator and a finished video. It is not a full video editor: export
produces the audio mix, or on macOS an MP4 of the cut video with that mix, and the original video
stays untouched.

## Features

- Record from the microphone at the playhead while the other tracks play.
- Split with 48 kHz sample accuracy, move clips between tracks, copy and paste sections, and undo
  any edit.
- Voice effects with presets: high-pass, equalizer, compressor, expander and more.
- Loudness normalization to −16 LUFS and a −1.5 dBTP true-peak ceiling by default.
- Mixer with volume, pan, mute and solo per track.
- Video preview for H.264 MP4 and Matroska, with one track per audio stream.
- Export a single track or the whole mix as WAV or MP3.
- On macOS, export the edited video with the full mix as an MP4, encoded on the Mac's hardware
  H.264 encoder. Gaps between video clips become black frames.
- AI voiceover from a written script with Kokoro, running locally.
- One project file (`.voximov`) that holds sources, takes and edits, with crash-safe saves.
  Projects saved as `.justspeak` files by earlier versions still open.
- 13 interface languages.

## Installation

There are no prebuilt downloads. Build the app from source.

### macOS

macOS is the primary platform. You need Node.js, a Rust toolchain, and the Xcode Command Line
Tools:

```console
xcode-select --install
bin/build-macos
```

The script builds a slim LGPL FFmpeg for video export with `scripts/build-ffmpeg.sh` (the first run
downloads the pinned FFmpeg source and compiles it into `build/.cache/ffmpeg/`), installs the npm
dependencies, builds the app for Apple Silicon, and copies a versioned `.dmg` to `build/`. The `.app` itself is in `build/.cache/release/bundle/macos/`.

The app is signed ad hoc and not notarized. A build made on your own Mac opens normally. A copy
downloaded from the internet or from another Mac is blocked on first launch; allow it under
**System Settings → Privacy & Security → Open Anyway**. Recording asks for microphone access on
first use.

### Windows

Windows is kept but secondary. `bin/build-windows` cross-builds a single executable from WSL; it
needs the `x86_64-pc-windows-msvc` Rust target, `cargo-xwin` and LLVM's `llvm-rc`. The
[Windows build](.github/workflows/windows-build.yml) workflow builds the same executable on
GitHub-hosted runners. The app needs the WebView2 Runtime.

## Quick start

1. Open the menu at the top left and import a video, or drop it on the window.
2. Put on headphones, move the playhead, and press <kbd>R</kbd> to record on the **My voice**
   track while the other tracks play. Press <kbd>R</kbd> again to finish.
3. Edit: split with <kbd>X</kbd>, select with <kbd>V</kbd>, and drag clips or their edges.
4. Shape your voice in the **Effects** tab and balance the tracks in the **Mixer** tab.
5. Export the mix, or a single track from the mixer, as WAV or MP3, or export the video as MP4.
   Save the project with <kbd>Cmd</kbd>+<kbd>S</kbd>.

## AI voiceover

Voximov can turn a written script into speech with
[Kokoro](https://huggingface.co/hexgrad/Kokoro-82M), an open-source voice model. The app talks to
[Kokoro-FastAPI](https://github.com/remsky/Kokoro-FastAPI), a server that runs in Docker on your
own computer, so your script stays on your machine.

Install and start Docker (for example Docker Desktop or Rancher Desktop), then manage Kokoro with
the script in this repository:

```console
scripts/kokoro.sh start
```

The first `start` downloads about 1.5 GB. Kokoro then starts with Docker and listens only on this
computer at `http://localhost:8880`. The script also has `status`, `stop`, `reset` and `remove`
commands. `status` checks Docker, the container and the server. `reset` recreates the container without a new download.
`remove` deletes the container; `remove --all` also deletes the image. Run `scripts/kokoro.sh help`
for settings such as `KOKORO_PORT`, or `KOKORO_IMAGE` for the NVIDIA GPU image. On Windows, use
`powershell -ExecutionPolicy Bypass -File scripts\kokoro.ps1` with the same commands.

To write a voiceover:

1. Select the **Script** tool in the timeline toolbar.
2. Drag across the **My voice** track to create a script clip. Its width is the time the speech
   may take.
3. Double-click the clip, type the text, and press <kbd>Enter</kbd>. The clip generates in the
   background.

If the speech is longer than the clip, the rest appears as a faded overflow; drag the clip's right
edge or double-click the overflow to fit it. Choose the voice in the **Script** tab and the Kokoro
address in **Settings**. If the app reports that Kokoro isn't ready, run `scripts/kokoro.sh status`.

## Keyboard shortcuts

On macOS, <kbd>Cmd</kbd> works in place of <kbd>Ctrl</kbd> except for wheel zoom.

| Keys                  | Action                           |
| --------------------- | -------------------------------- |
| Space                 | Play / pause                     |
| R                     | Start / finish recording         |
| V / X                 | Select / split tool              |
| Delete                | Remove selected clip             |
| Home                  | Go to start                      |
| Ctrl+S / Ctrl+Shift+S | Save / save as                   |
| Ctrl+Z / Ctrl+Shift+Z | Undo / redo                      |
| Ctrl+C / Ctrl+V       | Copy section / paste at playhead |
| Ctrl+ + / Ctrl+ −     | Horizontal timeline zoom         |
| Ctrl+wheel            | Zoom at the pointer              |

## License

Voximov is source-available under the [PolyForm Strict License 1.0.0](LICENSE). Use is
free of charge for the purposes permitted by that license, including:

- Any noncommercial purpose.
- The personal uses listed in the license, such as personal study and hobby projects, without any
  anticipated commercial application.
- Use by charitable organizations, educational institutions, public research organizations, public
  safety or health organizations, environmental protection organizations, and government
  institutions, regardless of funding source or obligations resulting from funding.

This license grants no rights to modify or redistribute Voximov or create works based on
it. The full license governs; this summary does not grant additional permissions.

Commercial use outside those permissions requires a separate paid license. Contact
[Emrah Diril](mailto:emrah@diril.org) for pricing and a commercial agreement before beginning such
use.

Voximov is based on simpleVoiceover by Arcon, used under the MIT License; that notice is in
[`NOTICE`](NOTICE). The macOS app bundles FFmpeg under the GNU LGPL 2.1 or later, also described in
[`NOTICE`](NOTICE).

## Development

Install the dependencies, then run the tests and checks:

```console
npm ci
npm test
npm run build
npm run lint
npm run format:check
cargo fmt --manifest-path backend/Cargo.toml --check
```

`npm test` runs the Vitest unit tests and the Rust engine tests. `npm run lint` runs ESLint and
Clippy. On macOS, the video export tests run FFmpeg from `VOXIMOV_FFMPEG` and `VOXIMOV_FFPROBE`, or from
`PATH`, and skip with a message when neither is found. To test the bundled build, run
`scripts/build-ffmpeg.sh` and point both variables at `backend/binaries/ffmpeg-aarch64-apple-darwin`
and `backend/binaries/ffprobe-aarch64-apple-darwin`. Without those binaries, desktop builds still
compile but cannot export video. Builds go to `build/`, and Cargo keeps intermediate files in `build/.cache/`. See
[`CHANGELOG.md`](CHANGELOG.md) for completed changes and [`TODO.md`](TODO.md) for open work.

### Releasing

1. In [`CHANGELOG.md`](CHANGELOG.md), rename `Unreleased` to the new version and date, and add a
   new empty `Unreleased` section above it.
2. Set the same version in `package.json`, `package-lock.json`, `backend/Cargo.toml`,
   `backend/Cargo.lock` and `backend/tauri.conf.json`.
3. Run the checks above.
4. Commit, create the annotated tag `v<version>` on that commit, and push both.
5. The [Release](.github/workflows/release.yml) workflow starts from the tag. It stops if any
   manifest version differs from the tag, builds the Windows executable, writes
   `checksums.sha256`, and creates a draft GitHub Release with the notes from `CHANGELOG.md`.
   Review the draft and publish it.

The workflow does not build macOS. Build the `.dmg` locally with `bin/build-macos`. Never move or
reuse a release tag; fix a bad release with a new version.
