# macOS first

Date: 2026-11-09

## Decision

macOS is the primary platform. New features are built for macOS only, and
macOS-only implementations such as AVFoundation and ScreenCaptureKit are fine.

- Build the macOS app and `.dmg` with `bin/build-macos`.
- Keep the existing Windows code and the Windows executable while they stay
  low-maintenance. Do not remove them.
- Linux is not supported. Its packages and build scripts were removed.

## Rationale

The target users are people recording demos and tutorials on a Mac, and the
planned features (window recording, hardware video encoding) depend on macOS
frameworks. Writing cross-platform versions of them would slow the product
down for platforms nobody has asked for yet.

## Boundaries

- A new feature does not need a Windows version. It must not break the
  Windows build.
- If keeping Windows stops being low-maintenance, that is a new decision.
