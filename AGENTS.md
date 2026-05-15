# AGENTS.md

## Project Overview

ChatGPT Helper is a Windows-focused Tauri 2 desktop app. The Rust backend manages app state, local JSON storage, OAuth login, ChatGPT model/usage calls, screen capture, native screen-area selection, clipboard integration, logging, and Tauri commands. The TypeScript frontend is a small namespace-based UI that renders state, handles keyboard shortcuts, OCR with Tesseract, manual input, scan actions, and Tauri command calls.

## Repository Layout

- `src/main.rs`: Tauri startup, global shortcuts, command registration.
- `src/app`: application state, frontend view models, Tauri command handlers, and UI events.
- `src/domain`: serializable domain models, settings, auth/catalog storage models, defaults, and history types.
- `src/infra`: persistence, paths, logging, shell helpers, clipboard, screen capture/selection, and ChatGPT HTTP helpers.
- `frontend/src`: browser-side TypeScript namespaces compiled by `tsc`.
- `frontend/index.html` and `frontend/styles.css`: desktop UI shell and styling.
- `frontend/scripts/prepare-dist.ps1`: copies frontend assets and Tesseract browser bundle into `frontend/dist`.
- `capabilities/default.json`: packaged capability metadata.
- `vendor/typeid`: local crate patch; do not edit unless the dependency patch itself is the task.

## Build Commands

- Frontend build: `cd frontend && npm.cmd run build`
- Rust/Tauri build check: `cargo build`
- Run in development: `cargo run`

The Rust build script also tries to ensure `frontend/dist` exists. If TypeScript or frontend assets changed, run the frontend build before `cargo build`.

## Coding Conventions

- Keep every source file topped with a short file-purpose comment.
- Keep a short behavior comment directly above each Rust `fn` and TypeScript `function`.
- Prefer existing module boundaries:
  - user-facing state changes belong in `src/app/state.rs`;
  - Tauri command wrappers belong under `src/app/commands`;
  - serializable models and defaults belong in `src/domain`;
  - OS, storage, network, and ChatGPT integration details belong in `src/infra`.
- Rust errors should use `anyhow::Result` internally and convert to `String` only at Tauri command boundaries.
- Frontend code uses global namespaces and triple-slash references, not ES module imports.
- Keep frontend state in `AppContext.model`; render changes through `Renderer`.
- Preserve Windows behavior for screen selection, clipboard, and global shortcuts; non-Windows fallbacks should return clear errors.

## Verification

After code changes, run the narrowest useful checks:

- `cd frontend && npm.cmd run build` for TypeScript/frontend changes.
- `cargo build` for Rust/backend or Tauri command changes.

There is no dedicated automated test suite in this repository at the time of writing.
