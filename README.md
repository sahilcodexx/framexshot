# FrameXShot

> A fast, open-source Linux screenshot tool built with Tauri v2 + React. Capture, edit, and enhance your screenshots with professional quality — entirely offline, entirely local.

[![License: BSD 3-Clause](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24C8D8?logo=tauri)](https://tauri.app)
[![React](https://img.shields.io/badge/React-19-61DAFB?logo=react)](https://react.dev)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.8-3178C6?logo=typescript)](https://www.typescriptlang.org)
[![Platform](https://img.shields.io/badge/Platform-Linux-FCC624?logo=linux&logoColor=black)](https://www.linux.org)

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
  - [Capture Modes](#capture-modes)
  - [Image Editor](#image-editor)
  - [Annotation Tools](#annotation-tools)
  - [Workflow](#workflow)
- [Install](#install)
  - [Download a Release](#download-a-release)
  - [Build from Source](#build-from-source)
- [Usage](#usage)
  - [Quick Start](#quick-start)
  - [Auto-Apply Workflow](#auto-apply-workflow)
  - [Keyboard Shortcuts](#keyboard-shortcuts)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

FrameXShot is a lightweight, native Linux desktop application designed to replace complex screenshot workflows. It lives in the **system tray**, captures with a keypress, and lets you apply backgrounds, effects, and annotations — all in a slick dark-mode UI powered by Rust + React.

No cloud. No telemetry. Everything happens locally.

**Stack:** Tauri v2 · React 19 · TypeScript 5.8 · Vite 7 · Zustand · Tailwind CSS v4 · xcap (X11/Wayland)

---

## Features

### Capture Modes

| Mode | Description | Default Shortcut |
|------|-------------|-----------------|
| **Region** | Drag-select any area of the screen | `Ctrl+Shift+2` |
| **Fullscreen** | Capture the full display | `Ctrl+Shift+F` |
| **Window** | Capture a specific application window | `Ctrl+Shift+D` |
| **OCR Region** | Extract text from a selected region (copies to clipboard) | `Ctrl+Shift+O` |

All shortcuts are customisable in Preferences.

### Image Editor

- **Background library** — Curated wallpapers, mac-style assets, mesh gradients, and solid colours
- **Custom backgrounds** — Pick any hex colour or use a transparent checkerboard
- **Effects** — Blur + noise sliders with 200 ms idle-commit for silky preview performance
- **Shadow** — Configurable X/Y offset, blur, and opacity
- **Border radius** — Pixel-perfect corner rounding
- **Padding** — Independent top / bottom / left / right control
- **Export** — High-quality JPEG to disk, or direct clipboard copy

### Annotation Tools

- **Shapes** — Circle, rectangle, line, arrow
- **Text** — Add labels with adjustable size
- **Numbered badges** — Auto-incrementing callout labels for step-by-step guides
- **Interaction** — Select, move, resize, and delete annotations
- **Styling** — Colour, opacity, border, and alignment controls

### Workflow

- **Global shortcuts** — Capture from anywhere, even when the window is hidden in the tray
- **Auto-apply** — Apply your default background and save without ever opening the editor
- **Quick Overlay** — A floating preview window that fades out automatically after 5 seconds
- **System tray** — FrameXShot lives in the tray; close to hide, never quits until you say so
- **Persistent preferences** — Save directory, shortcut bindings, and defaults survive restarts
- **Keep-mounted editor** — The editor stays in the DOM between captures, so state is never lost

---

## Install

### Download a Release

1. Go to [Releases](../../releases)
2. Download the appropriate package for your distro:
   - `.AppImage` — Universal (x86\_64)
   - `.deb` — Debian / Ubuntu
   - `.rpm` — Fedora / openSUSE
3. Make it executable (AppImage) and launch:
   ```bash
   chmod +x framexshot_*.AppImage
   ./framexshot_*.AppImage
   ```
4. On first launch, grant **Screen Recording** permission if prompted by your compositor.

### Build from Source

#### Requirements

| Tool | Minimum Version |
|------|----------------|
| Node.js | 18+ |
| pnpm | 10+ |
| Rust | latest stable |
| Tauri CLI | v2 |

#### Linux system dependencies

```bash
# Debian / Ubuntu
sudo apt install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev \
  libxcb1-dev libxrandr-dev libdbus-1-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel libayatana-appindicator-gtk3-devel librsvg2-devel \
  libxcb-devel libXrandr-devel dbus-devel
```

#### Clone and build

```bash
git clone https://github.com/sahilcodexx/framexshot.git
cd framexshot

pnpm install
pnpm tauri build

# or you can use bun for better JS runtime performance
```

The distributable is written to `src-tauri/target/release/bundle/`.

---

## Usage

### Quick Start

1. Launch FrameXShot — it appears in the system tray
2. Press a capture shortcut (default: `Ctrl+Shift+2` for region)
3. Select your area
4. The editor opens — pick a background, adjust effects, annotate
5. Press `Ctrl+S` to save or `Shift+Ctrl+C` to copy to clipboard

### Auto-Apply Workflow

For lightning-fast captures without touching the editor:

1. Toggle **"Auto-apply background"** on the main screen
2. Set your preferred default background in Preferences
3. Capture — FrameXShot automatically applies the background and saves instantly
4. A **Quick Overlay** preview fades in for 5 seconds, then disappears
5. Done — no editor required

### Keyboard Shortcuts

#### Capture Shortcuts

| Action | Default |
|--------|---------|
| Capture Region | `Ctrl+Shift+2` |
| Capture Fullscreen | `Ctrl+Shift+F` *(disabled by default)* |
| Capture Window | `Ctrl+Shift+D` *(disabled by default)* |
| OCR Region | `Ctrl+Shift+O` *(disabled by default)* |
| Cancel Selection | `Esc` |

#### Editor Shortcuts

| Action | Shortcut |
|--------|----------|
| Save Image | `Ctrl+S` |
| Copy to Clipboard | `Shift+Ctrl+C` |
| Undo | `Ctrl+Z` |
| Redo | `Shift+Ctrl+Z` |
| Delete Annotation | `Delete` / `Backspace` |
| Close Editor | `Esc` |

---

## Development

This repo contains:

- **Desktop app** (Tauri + Vite) — repo root
- **Landing site** (Next.js) — `framexshot-landing/` *(legacy, rename in progress)*

### Desktop app

```bash
pnpm tauri dev       # Full dev mode (Rust + HMR frontend)
pnpm run build       # Frontend-only build (fast, ~5 s)
pnpm lint:ci         # TypeScript type-check only
pnpm test            # vitest unit tests
pnpm test:rust       # cargo test (Rust unit tests)
```

> **Note:** Changes to Rust code (`.rs` files) require `pnpm tauri build` or `pnpm tauri dev` — `pnpm run build` only compiles the frontend.

### Architecture highlights

| Concern | Solution |
|---------|----------|
| Capture | `xcap` crate (X11 + Wayland via xdg-desktop-portal) |
| State | Zustand v5 with **granular selectors** per field |
| Preview | Synchronous `canvas.toDataURL("image/jpeg", 0.85)` with 200 ms idle-commit |
| Annotations | Ref-based drag — zero React re-renders during drawing |
| Editor mount | Keep-mounted (`display: none` when inactive) — no re-mount cost |
| Tray lifecycle | Close → hide to tray; Quit from tray → `app.exit(0)` |

---

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

- **Bug reports** — Open a GitHub issue with reproduction steps
- **Feature requests** — Open an issue tagged `enhancement`
- **Pull requests** — Fork → branch → PR against `main`

---

## License

BSD 3-Clause License — see [LICENSE](LICENSE) for details.
