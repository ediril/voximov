# The engine owns media and edits

Date: 2026-10-05

## Decision

Voximov is split into a React and TypeScript interface and a Rust engine.

- The Rust engine in `backend/src` owns decoding, DSP, project storage and
  editing. Every change to a project goes through an engine command.
- React components live in `src/components`. Timeline behavior lives in
  `src/timeline`.
- Hooks coordinate transport and the desktop lifecycle through `StudioClient`
  and Tauri IPC.
- The WebSocket carries binary audio only. Commands and state travel over IPC.

## Rationale

Audio work needs exact sample timing, large buffers and crash-safe storage,
which the engine handles better than the webview. Keeping every edit in one
engine command layer also means the planned CLI drives the same commands as
the app, with no second implementation.

## Boundaries

- The interface displays state and sends commands. It does not edit the
  project on its own.
- New editing features are engine commands first. The app and the CLI both
  call them.
