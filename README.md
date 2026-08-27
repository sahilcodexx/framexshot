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
  - [Arch Linux (AUR)](#arch-linux-aur)
  - [Universal CLI Installer (all other distros)](#universal-cli-installer-all-other-distros)
  - [Windows / macOS](#windows--macos)
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

FrameXShot is a lightweight, native Linux desktop application designed to replace complex screenshot workflows. It lives in the **system tray**, captures with a keypress, and lets you apply backgrounds, effects, and annotations — all in a slick adaptive UI (light/dark via shadcn oklch) powered by Rust + React.

No cloud. No telemetry. Everything happens locally.

**Stack:** Tauri v2 · React 19 · TypeScript 5.8 · Vite 7 · Zustand · Tailwind CSS v4 + shadcn oklch (`@custom-variant dark`, `@theme inline`) · xcap (X11/Wayland)

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
- **Shadow** — Configurable X/Y offset, blur, and opacity (minimal in light, heavier in dark)
- **Border radius** — Pixel-perfect corner rounding
- **Padding** — Independent top / bottom / left / right control (persists via *Set as Default*, no random jumps)
- **Floating sidebar** — Detached `rounded-2xl` card (`bg-card` on `bg-canvas`, `shadow-sm` light / `shadow-xl` dark, fills top gap via `-mt-8`)
- **Cursors** — `pointer` for buttons, `grab/grabbing` for canvas & 2D pad, `ew-resize` for sliders, `crosshair` for drawing
- **Export** — High-quality JPEG to disk, or direct clipboard copy (Cancel is `destructive` tint)

### Annotation Tools

- **Shapes** — Circle, rectangle, line, arrow
- **Text** — Add labels with adjustable size
- **Numbered badges** — Auto-incrementing callout labels for step-by-step guides
- **Interaction** — Select, move, resize, and delete annotations
- **Styling** — Colour, opacity, border, and alignment controls

### Workflow

- **Global shortcuts** — Capture from anywhere, even when the window is hidden in the tray (Capture Screen has `400ms` hide delay so tray menu isn't captured)
- **Auto-apply** — Apply your default background and save without ever opening the editor
- **Quick Overlay** — A floating preview window that fades out automatically after 5 seconds
- **System tray** — FrameXShot lives in the tray; close to hide, never quits until you say so
- **Persistent preferences** — Save directory, shortcut bindings, theme (`light`/`dark` via `html.dark` + `settings.json`) and defaults survive restarts
- **Keep-mounted editor** — The editor stays in the DOM between captures, so state is never lost
- **Minimal TitleBar** — Dots only (`TitleBar.tsx:13`), transparent `h-8`, `data-tauri-drag-region` default cursor

---

## Install

FrameXShot builds against your system libraries, so there are no AppImage/Flatpak/deb/rpm bundles to fight with. On Linux you either install from the AUR (Arch-based) or via the universal CLI installer, which installs every dependency through your native package manager and compiles the app from source.

### 🏔️ Arch Linux (AUR)

For Arch, Manjaro, and EndeavourOS, install from the AUR (any AUR helper works):

```bash
yay -S framexshot
# or: paru -S framexshot
```

The PKGBUILD pulls in the runtime dependencies (`webkit2gtk-4.1`, `gtk3`, `libayatana-appindicator`, `tesseract`, capture tools) automatically.

### 🐧 Universal CLI Installer (all other distros)

One command installs all build + runtime dependencies via your distro's package manager, downloads the source for the latest release, compiles it, and installs it to `/usr/local`:

```bash
curl -fsSL https://raw.githubusercontent.com/sahilcodexx/framexshot/main/packaging/install.sh | sh
```

**Supported:** Debian 12+ · Ubuntu 22.04+ · Fedora · RHEL/Rocky/Alma/CentOS *(fail with a clear message — they only ship WebKitGTK 4.0)* · openSUSE · Arch/Manjaro (if you prefer not to use the AUR).

**Environment overrides:**

| Variable | Purpose |
|----------|---------|
| `FXS_VERSION` | Build a specific tag (default: latest release) |
| `FXS_SKIP_DEPS` | `1` to skip dependency installation |
| `FXS_NO_SUDO` | `1` to fail instead of prompting for sudo |
| `FXS_KEEP_SRC` | `1` to keep the source tree in `/tmp` (debugging) |

*Debian 11 and Ubuntu 20.04 (and older) can't run Tauri v2 apps — `libwebkit2gtk-4.1-0` doesn't exist there. The installer fails fast with a clear message instead of a confusing package-not-found error.*

### 🖥️ Windows / macOS

GitHub Releases ship native installers for Windows (`.exe`/NSIS) and macOS (`.dmg`). Download the installer for your platform from [Releases](../../releases), then run it — no dependencies to install.

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

*Tip: For optical character recognition (OCR), install `tesseract-ocr` (Debian/Ubuntu) or `tesseract` (Fedora/Arch/openSUSE). The CLI installer and AUR package pull it in automatically.*

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

> **Tip:** Most users never need to do this manually — `packaging/install.sh` handles deps + build + install in one command.

```bash
# 1. Clone repository
git clone https://github.com/sahilcodexx/framexshot.git
cd framexshot

# 2. Install frontend dependencies
pnpm install --frozen-lockfile

# 3. Build production bundle (generates AppImage, .deb, .rpm for local dev)
pnpm tauri build
```

The compiled packages will be created in `src-tauri/target/release/bundle/`. Note that Linux **releases** only ship the source-build installer above — the AppImage/deb/rpm targets exist for local testing, not distribution.

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
| Capture | `xcap` crate (X11 + Wayland via xdg-desktop-portal), `fullscreen` 400ms hide for tray menu |
| State | Zustand v5 with **granular selectors** per field, padding respects `defaultPaddingTop` from `settings.json` |
| Preview | Synchronous `canvas.toDataURL("image/jpeg", 0.85)` with 200 ms idle-commit, centered `flex` loading |
| Annotations | Ref-based drag — zero React re-renders during drawing, `grab`/`crosshair` cursors |
| Editor mount | Keep-mounted (`display: none` when inactive) — no re-mount cost, floating sidebar `rounded-2xl` with `-mt-8` top fill |
| Tray lifecycle | Close → hide to tray; Quit from tray → `app.exit(0)` |
| Theme | shadcn oklch `light` (`:root`) / `dark` (`.dark`), `@custom-variant dark`, `useTheme` persisting to `settings.json`, no hardcoded `bg-[#xxx]` |

---

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

- **Bug reports** — Open a GitHub issue with reproduction steps
- **Feature requests** — Open an issue tagged `enhancement`
- **Pull requests** — Fork → branch → PR against `main`

---

## License

BSD 3-Clause License — see [LICENSE](LICENSE) for details.
