# Releases and builds

Date: 2026-10-10

## Decision

Voximov uses Semantic Versioning, and `CHANGELOG.md` follows Keep a Changelog.
The version changes only when a release is prepared, not with every change.
Completed work goes under `Unreleased` in `CHANGELOG.md`. The date-based
scheme used in other projects does not apply here.

- The version is the same in `package.json`, `package-lock.json`,
  `backend/Cargo.toml`, `backend/Cargo.lock` and `backend/tauri.conf.json`.
- Every release commit gets an annotated tag `v<version>`. A release tag is
  never moved or reused. A bad release is fixed with a new version.
- The owner commits and creates the tag. For a release, the hand-off includes
  the exact `git tag -a` command next to the commit message.
- Builds go to `build/`, with Cargo intermediates in `build/.cache/`. A
  repeated build of the same version replaces the earlier one, and obsolete
  builds and backup copies are removed.
- Generated binaries stay out of commits.

## Rationale

The release workflow starts from the tag and refuses to build when any
manifest version differs from it, so the five manifests and the tag must
agree. Users download versioned builds, so a published version must always
mean the same files.

## Mechanism

`README.md` (Releasing) lists the steps, and
`.github/workflows/release.yml` checks the versions and drafts the GitHub
Release.
