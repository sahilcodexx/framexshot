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

FrameXShot is packaged for all major Linux distributions and architectures.

#### 📦 Flatpak (Universal — Recommended)

The Flatpak package is self-contained (bundles its own WebKitGTK 4.1 runtime) and works reliably across all Linux distributions without dependency conflicts:

```bash
# Install Flatpak bundle
flatpak install ./framexshot_1.0.0_amd64.flatpak

# Run FrameXShot
flatpak run com.framexshot.app
```

#### 🌀 Debian / Ubuntu / Pop!_OS / Linux Mint

Download the `.deb` package from [Releases](../../releases):

```bash
sudo apt update
sudo apt install ./framexshot_1.0.0_amd64.deb
```

*APT automatically installs required dependencies (`libwebkit2gtk-4.1-0`, `libgtk-3-0`, `libayatana-appindicator3-1`, `tesseract-ocr`).*

#### 🎩 Fedora / RHEL / CentOS Stream / Rocky Linux / AlmaLinux

Download the `.rpm` package from [Releases](../../releases):

```bash
sudo dnf install ./framexshot_1.0.0_x86_64.rpm
```

#### 🦎 openSUSE (Leap / Tumbleweed)

Download the `.rpm` or `.AppImage` from [Releases](../../releases):

```bash
# Via zypper (.rpm)
sudo zypper install ./framexshot_1.0.0_x86_64.rpm
```

#### 🏔️ Arch Linux / Manjaro / EndeavourOS

Use the `.AppImage` or build from source:

```bash
# Make AppImage executable and run
chmod +x framexshot_1.0.0_amd64.AppImage
./framexshot_1.0.0_amd64.AppImage
```

#### 🚀 AppImage (Universal Binary)

Works on any Linux distribution with FUSE installed:

```bash
chmod +x framexshot_1.0.0_amd64.AppImage
./framexshot_1.0.0_amd64.AppImage

# If FUSE is not installed on your system (e.g. Ubuntu 22.04+ default minimal setup):
./framexshot_1.0.0_amd64.AppImage --appimage-extract-and-run
```

---

### Desktop Environment & Wayland Capture Compatibility

FrameXShot automatically detects your desktop environment and Wayland/X11 session, using multi-tiered capture fallbacks:

| Desktop Environment | Display Server | Primary Tool | Fallback Chain |
|---------------------|----------------|--------------|----------------|
| **GNOME 42+** | Wayland / X11 | `org.gnome.Shell` D-Bus (built-in, no install needed) | `xdg-desktop-portal` → `gnome-screenshot` |
| **KDE Plasma 5/6** | Wayland / X11 | `spectacle` | `xdg-desktop-portal` → `grim` + `slurp` |
| **COSMIC** | Wayland | `cosmic-screenshot` | `xdg-desktop-portal` → `grim` |
| **Sway / Hyprland** | Wayland (wlroots) | `grim` + `slurp` | `xdg-desktop-portal` |
| **XFCE / MATE / Cinnamon / LXQt** | X11 | `maim` / `scrot` / `spectacle` | `xdg-desktop-portal` → `xcap` |

#### Install capture backends

Most desktops already ship a working tool (GNOME 42+ works out of the box — no install needed). For the others, install the package for your distro:

**Debian / Ubuntu / Pop!_OS / Mint:**
```bash
sudo apt install spectacle grim slurp scrot maim wl-clipboard tesseract-ocr xdg-desktop-portal
```

**Fedora / RHEL:**
```bash
sudo dnf install spectacle grim slurp scrot maim wl-clipboard tesseract xdg-desktop-portal
```

**Arch Linux / Manjaro:**
```bash
sudo pacman -S spectacle grim slurp scrot maim wl-clipboard tesseract xdg-desktop-portal
```

**openSUSE (Leap / Tumbleweed):**
```bash
sudo zypper install spectacle grim slurp scrot maim wl-clipboard tesseract-ocr xdg-desktop-portal
```

*Tip: For optical character recognition (OCR), install `tesseract-ocr` (Debian/Ubuntu) or `tesseract` (Fedora/Arch/openSUSE). The Flatpak bundle ships its own copy — nothing to install.*

---

### Build from Source

#### Requirements

| Tool | Minimum Version |
|------|----------------|
| Node.js | 20+ |
| pnpm | 10+ |
| Rust | 1.80+ (stable) |
| Tauri CLI | v2 |

#### System Build Dependencies by Distro

**Debian / Ubuntu / Pop!_OS:**
```bash
sudo apt update
sudo apt install -y build-essential curl wget pkg-config \
  libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
  librsvg2-dev patchelf libxdo-dev clang libclang-dev \
  libpipewire-0.3-dev libgbm-dev libdrm-dev
```

**Fedora / RHEL / CentOS:**
```bash
sudo dnf install -y gcc gcc-c++ make curl wget pkg-config \
  gtk3-devel webkit2gtk4.1-devel libayatana-appindicator-gtk3-devel \
  librsvg2-devel patchelf libxdo-devel clang-devel \
  pipewire-devel mesa-libgbm-devel libdrm-devel
```

**Arch Linux / Manjaro:**
```bash
sudo pacman -S --needed base-devel curl wget pkgconf \
  gtk3 webkit2gtk-4.1 libayatana-appindicator \
  librsvg patchelf xdotool clang \
  pipewire mesa libdrm
```

**openSUSE (Leap / Tumbleweed):**
```bash
sudo zypper install -t pattern devel_basis
sudo zypper install -y gtk3-devel libwebkit2gtk-4_1-devel \
  libayatana-appindicator3-devel librsvg-devel patchelf \
  libxdo-devel clang-devel pipewire-devel
```

#### Clone & Build

```bash
# 1. Clone repository
git clone https://github.com/kartiklhb/framexshot.git
cd framexshot

# 2. Install frontend dependencies
pnpm install --frozen-lockfile

# 3. Build production bundle (generates AppImage, .deb, .rpm)
pnpm tauri build
```

The compiled packages will be created in `src-tauri/target/release/bundle/`.

#### Build the Flatpak bundle (optional)

Requires `flatpak` + `flatpak-builder` on the host:

```bash
# 1. Frontend build (embedded into the Rust binary)
pnpm install --frozen-lockfile
pnpm run build

# 2. Rust release binary (webview assets are baked in at compile time)
cargo build --release --manifest-path src-tauri/Cargo.toml

# 3. Install the GNOME 49 runtime (48 is EOL and must not be used)
flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install --user --noninteractive -y flathub org.gnome.Platform//49 org.gnome.Sdk//49

# 4. Build the bundle
flatpak-builder --user --force-clean --repo=flatpak-repo \
  flatpak-build-dir flatpak/com.framexshot.app.yml

# 5. Export a single-file bundle and install it
flatpak build-bundle --runtime-repo=https://flathub.org/repo/flathub.flatpakrepo \
  flatpak-repo framexshot.flatpak com.framexshot.app
flatpak install --user ./framexshot.flatpak
```

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
