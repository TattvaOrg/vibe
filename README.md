# vibe

Ambient aura and liquid-glass now-playing suite for Linux.
Directly inspired by Lumn for macOS (trylumn.xyz).

---

## Direct Binary Installation

Install the pre-built Linux binary directly with a single command (no compiler or package manager required):

```bash
curl -fsSL https://raw.githubusercontent.com/TattvaOrg/vibe/main/install.sh | sh
```

This installs:
- Executable binary to `~/.local/bin/vibe`
- Desktop entry to `~/.local/share/applications/vibe.desktop`
- Application icon to `~/.local/share/icons/hicolor/scalable/apps/vibe.svg`

---

## Quick Start

```bash
# Start background ambient lighting overlay
vibe daemon

# Open the rich matte-black settings window
vibe settings

# Toggle ambient glow and tracer line on or off via keybinding
vibe toggle

# Cycle through animation modes (Orbit, Beat Pulse, Fluid Wave, Breathe, Steady)
vibe next-mode
```

---

## Overview

vibe transforms Linux displays with a fluid ambient aura that dynamically synchronizes with active music playback.

- Perimeter Tracer Beam: A luminous white light line racing and orbiting around the screen's curved contours, accelerating dynamically with music tempo and beat bursts.
- Audio Reactive: Ambient aura pulses and breathes to the rhythm and bass transients of music via low-latency PipeWire loopback audio.
- Dynamic Color Palette: Extracts dominant and secondary accent palettes from album artwork across Spotify, Apple Music, Cider, Web browsers, and MPRIS-compatible players.
- Frosted Liquid-Glass Card: Interactive floating player with playback controls, dynamic progress, and clean specular highlights.
- Lockscreen Integration: Ambient aura and media banner integrated into Wayland lockscreens via the ext-session-lock-v1 protocol.
- Wayland-Native: Native wlr-layer-shell overlay support with click-through input transparency for Niri, Hyprland, Sway, KDE Plasma 6, and COSMIC.
- Ultra-Low Resource Usage: Written in Rust with hardware-accelerated OpenGL/GLSL shaders, targeting under 1 percent CPU and under 15 MB RAM at 60 to 144+ FPS.
- Matte Black Minimalist Design: Zero emojis, disciplined rectangular layouts, clean vector SVGs, and rich obsidian dark themes (#09090b, #121214, #27272a).

---

## Logo and Identity

- Vector Logo: `assets/icons/logo.svg` (Minimalist monochrome squircle with curved ambient edge glow).
- System Tray Glyph: `assets/icons/tray-icon.svg` (High-contrast 24x24 vector outline).

---

## Specifications and Architecture

For complete technical specifications, architectural diagrams, performance budgets, and the implementation roadmap, consult [prd.md](prd.md).

---

## Tech Stack

- Language: Rust (Edition 2024)
- Settings & Widget UI: Slint native declarative framework (zero Node.js / WebKit, <15MB RAM)
- Display Protocols: Wayland wlr-layer-shell, ext-session-lock-v1
- Shaders: Universal GLSL shaders via glow (OpenGL 3.3 Core / OpenGL ES 3.0)
- Audio Capture: PipeWire native sink-monitor loopback
- Audio Analysis: Real-time FFT and transient onset energy detection
- Media Metadata: MPRIS2 D-Bus client via zbus
- Color Quantization: Fast k-means color extraction via palette and color-thief
- System Tray: StatusNotifierItem via ksni

---

## License

GPL-3.0 License. See [LICENSE](LICENSE) for details.