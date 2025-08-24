-# Copilot Instructions for Nakama

## Project Overview
- Nakama is a Tauri desktop app with a React/TypeScript frontend and a Rust backend.
- The frontend is in `src/` and uses Vite, Tailwind CSS, and Tauri's JS API for backend communication.
- The backend is in `src-tauri/`, with Rust modules for commands and features (e.g., `command.rs`, `recorder.rs`).

## Architecture & Data Flow
- Frontend communicates with backend using Tauri's `invoke` API (see `MessageInput.tsx`).
- Backend exposes commands via `#[tauri::command]` (see `command.rs`, `recorder.rs`).
- Screen recording is triggered from the frontend and runs in Rust, saving video segments to `/recordings`.
- Hotkey (Ctrl+Alt+Shift+C) and frontend button both stop recording; status is reflected in the UI.

## Key Patterns & Conventions
- All Tauri commands must be registered in `lib.rs` via `tauri::generate_handler!`.
- Frontend state (e.g., recording status, timer) is managed with React hooks.
- Recording indicator is always visible in the top-left (`MessageInput.tsx`).
- Rust backend uses `screenshots`, `rdev`, and `ffmpeg` for screen capture and encoding.
- Output files are segmented and stored in `/recordings`.
- Use `invoke('command_name', { ... })` for frontend-backend calls.

## Developer Workflows
- **Build/Run:**
  - Frontend: `pnpm dev` or `npm run dev` (Vite)
  - Tauri app: `pnpm tauri dev` or `npm run tauri dev`
- **Rust dependencies:** Managed in `src-tauri/Cargo.toml`.
- **FFmpeg:** Must be installed on the system for recording to work.
- **Debugging:**
  - Rust: Use `println!` for logs (e.g., recording timer)
  - Frontend: Use browser devtools and React DevTools

## Integration Points
- `MessageInput.tsx`: Example of invoking backend commands and showing live status/timer.
- `recorder.rs`: Shows hotkey handling, FFmpeg process management, and timer logic.
- `lib.rs`: Registers all Tauri commands and plugins.

## Examples
- To add a new backend feature, create a Rust function with `#[tauri::command]`, register it in `lib.rs`, and call it from the frontend with `invoke`.
- To update UI state based on backend events, use React hooks and async calls.

## External Dependencies
- Tauri, React, TypeScript, Vite, Tailwind CSS, FFmpeg, screenshots, rdev, crossbeam-channel, nix

---

If any section is unclear or missing important project-specific details, please provide feedback or request further clarification.
