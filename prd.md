# Product Requirements Document (PRD)

## Project: vibe - Ambient Lighting and Liquid-Glass Now-Playing for Linux

**Document Version:** 1.2.0  
**Status:** Approved / Specification Complete  
**License:** GNU General Public License v3.0 (GPL-3.0)  
**Target Platform:** Linux (Wayland-First: Niri, Hyprland, Sway, KDE Plasma 6, COSMIC; X11 via modular backend)  
**Reference Benchmark:** Lumn for macOS (trylumn.xyz)

---

## 1. Executive Summary

vibe is an open-source, hardware-accelerated fluid ambient aura and dynamic now-playing overlay suite crafted exclusively for Linux.

Directly matching the aesthetics and behavior of Lumn for macOS, vibe transforms screen borders into a fluid ambient aura that dynamically reacts to audio rhythm and syncs with media player colors. It features the signature luminous perimeter tracer beam—a crisp white light line that sweeps and orbits around the screen's curved contours in rhythm with the music. It inspects media player metadata (Spotify, Apple Music, Cider, Web browsers, local players) via MPRIS2 D-Bus, analyzes audio streams in real time via PipeWire loopback, and renders an interactive frosted matte "liquid glass" card on both the active desktop and the lockscreen.

vibe is engineered in 100% native Rust, pairing a lightweight declarative Slint settings application (zero Node.js, zero WebKit, <15MB RAM) with universal GLSL shaders (glow + EGL) and PipeWire audio loopback, adhering to a strict matte-black minimalist design system with vector SVGs and zero emojis throughout the application interface, system tray, and documentation.

---

## 2. Design System and Visual Language

### 2.1 Color Palette: Rich Matte Black and Obsidian
The user interface avoids noisy colorful chrome or cheap gradients, adopting a restrained, high-end industrial design inspired by modern dark-mode software (Linear, Raycast, macOS Sonoma dark aesthetic):

| Token | Hex Value | Usage |
| :--- | :--- | :--- |
| Background Base | `#09090b` | Main application background, window canvas |
| Background Surface | `#121214` | Cards, control groupings, floating panels |
| Background Elevated | `#18181b` | Hover states, active dropdown items, tooltips |
| Border Subtle | `#27272a` | Primary 1px container borders and dividers |
| Border Prominent | `#3f3f46` | Focused elements, active slider thumbs, active states |
| Text Primary | `#f4f4f5` | Main titles, active song title, slider values |
| Text Secondary | `#a1a1aa` | Subtitles, artist names, setting labels |
| Text Muted | `#71717a` | Units, secondary hints, disabled options |
| Specular Highlight | `rgba(255, 255, 255, 0.08)` | Frosted glass top-edge light reflection |

### 2.2 Geometry, Shapes, and Restraint
- Window Geometry: Rectilinear layouts with disciplined, intentional corner radiuses (8px for controls, 12px for cards and floating panels).
- No Unwanted or Exaggerated Shapes: Free of irregular blobs, exaggerated pill shapes, or asymmetrical rounded geometry. Every panel aligns strictly to an 8px grid.
- Border Treatment: Clean, crisp 1px hairline strokes (`#27272a`) separating distinct operational areas.

### 2.3 Iconography and Logo: Pure Vector SVG
- Zero Emoji Policy: Absolutely no unicode emojis anywhere in the UI, system tray menus, desktop notifications, or documentation.
- Vector SVG Icons: All icons (play, pause, next, previous, settings, sliders, power, lock, audio wave) are crafted as clean, geometric 1.5px/2px vector SVGs based on Lucide/Feather geometric standards.
- Brand Identity and Logo:
  - File: `assets/icons/logo.svg`
  - Concept: Minimalist monochrome squircle with rounded outer corners representing a display bezel, framed by a continuous curved ambient light beam and subtle center audio transient wave.
  - Tray Glyph: `assets/icons/tray-icon.svg`, a high-contrast 24x24 monochrome vector outline ensuring razor-sharp clarity on HiDPI displays.

---

## 3. Core Functional Modules (Lumn Parity)

### 3.1 Architectural Shift: Why Rigid "Rim" Was Replaced with Parametric Squircle Contours
A critical design and kinetic insight:
- The Flaw of a Rigid Rim: Treating edge lighting as a static rectangular "rim" clamped to monitor bezels forces harsh 90-degree corner collisions. When an animated white light line travels along a hard box rim, it hits sharp corners and snaps awkwardly, looking like an outdated desktop window border or a rigid plastic phone case.
- The Solution: Continuous Parametric Squircle Contour:
  - The white line travels along an organic, continuous superellipse/squircle path with a gentle curvature radius (configurable from 16px to 64px) and subtle screen inset (2px to 8px).
  - This allows the white line to preserve tangential momentum, sweeping effortlessly through corners like a photon in a particle accelerator.
  - The ambient glow is modeled not as a boxed "rim", but as a volumetric ambient aura with smooth exponential falloff that breathes inward naturally.

### 3.2 Fluid Ambient Aura Engine
Directly matching Lumn's visual controls without rigid boxing:
- Dial the Aura (Diffusion and Blur):
  - Whisper: A subtle hairline glow contour tracing the curved display edge with zero content obstruction.
  - Room Wash: Broad exponential Gaussian diffusion extending deep into the display borders for immersive night listening.
- Set the Contour Band:
  - Adjustable continuous range from 1px (hairline laser) to 120px+ (broad ambient wash).
- Primary vs Secondary Color Weight:
  - Dual-color gradient mixing (e.g., Primary 60% / Secondary 40%) extracted from media art or custom themes.
- Click-Through Transparency:
  - Layer-shell overlay surface marks input regions as empty (`wl_surface.set_input_region(empty)`), guaranteeing zero interference with clicks, mouse gestures, or window resizing underneath.

### 3.3 Perimeter White Tracer Beam Effect (Curved Orbit Line)
Directly implementing the signature dynamic effect featured in the reference video:
- Visual Anatomy:
  - A razor-sharp, luminous pure-white (or accent-matched) beam of light with an intense head and a tapered tail that orbits continuously along the curved squircle contour (Top -> Right -> Bottom -> Left).
  - High-Intensity Core: A 1.5px to 3.5px crisp, antialiased core filament rendered with high dynamic range brightness.
  - Outer Bloom / Halation: A soft 6px to 14px subtle outer glow hugging the tracer path so it feels integrated into the screen glass rather than overlaid artificially.
  - Continuous Corner Curvature: Smoothly negotiates corners via parametric squircle/bezier paths with zero sharp corner snaps.
- Motion and Kinetic Mechanics:
  - Continuous Orbit Mode: The white beam circles the perimeter at a steady baseline speed, dynamically accelerating during high-tempo (BPM) segments and energetic beat drops.
  - Beat-Triggered Surge: When a bass transient, drum kick, or snare hit occurs, the tracer beam surges in velocity and emits an outward energy bloom.
  - Track-Change Circuit Sweep: When a new song starts or playback resumes, dual white beams shoot out simultaneously from the top-center, racing in opposite directions around the display edges and fusing at the bottom-center to initiate playback.
  - Configurable Tracer Tail: User can adjust the tail length from a concise shooting-star comet head (15% perimeter length) to a continuous looping energy ribbon (100% perimeter length).
- Layering with Ambient Aura:
  - The ambient diffused aura sits as the foundation layer providing room-filling color atmosphere, while the crisp white tracer line glides on the foreground layer along the curved squircle path.

### 3.4 Audio Reactivity and Beat Detection Engine
- Audio Loopback: Direct native PipeWire stream monitoring the active default sink without requiring microphone permissions or recording hardware inputs.
- Real-Time Frequency Analysis:
  - Discrete FFT running in a dedicated audio thread at 60–120Hz update rates.
  - Sub-bass and bass transient onset detection (20Hz–200Hz) calculating instantaneous energy differentials to trigger rhythm kicks.
- Animation Modes:
  1. Orbit Tracer (Reference Video Style): Crisp white perimeter line orbiting around curved screen contours with beat-accelerated velocity, superimposed over ambient aura glow.
  2. Beat Pulse: Contour width and brightness pulse outward strictly in sync with bass transients and song rhythm.
  3. Fluid Wave: Continuous ambient color wave flowing along the display perimeter.
  4. Ambient Breathe: Slow, organic breathing luminescence matching track tempo (BPM).
  5. Steady Static Glow: Motionless, steady ambient wash holding the current palette without movement.
- Idle State Management:
  - Smooth fade to static idle glow or subtle sleep state after 3 seconds of audio silence.

### 3.5 Dynamic Color Extraction Engine
- MPRIS2 Integration:
  - Listens to active media players via D-Bus (`org.mpris.MediaPlayer2.*`).
  - Monitors `PlaybackStatus` and `Metadata` signals (track title, artist, album, `mpris:artUrl`).
- Palette Extraction:
  - Fetches album art buffer from local file URI or HTTPS URL.
  - Executes color quantization (k-means clustering / octree) to extract dominant primary and accent secondary colors.
  - Cosine interpolation shader transition (300ms to 1200ms) prevents abrupt color cuts between track changes.
- Fallback and Manual Modes:
  - Screen edge sampling via `wlr-screencopy` protocol for ambient video/game backlighting.
  - Handcrafted matte monochrome and studio gradient presets (Obsidian Mono, Slate, Amber Glow, Cyberpunk, Nordic Dusk).

### 3.6 Floating "Liquid Glass" Now-Playing Card
- Frosted Matte Glassmorphism:
  - GPU-accelerated backdrop blur shader with subtle inner specular highlight and 1px border.
  - High-resolution album cover thumbnail with 8px corner radius.
  - Track metadata: Crisp white track title and muted artist name.
  - Interactive scrubbing progress bar and playback buttons (Previous, Play/Pause, Next) using vector SVGs.
- Window Management:
  - Draggable across screens with edge-snapping.
  - Configurable auto-dismiss (shows for 5 seconds on track change, then fades out) or permanently pinned.

### 3.7 Lockscreen Integration
- Wayland `ext-session-lock-v1` integration:
  - When the screen locks (via `hyprlock`, `swaylock`, `gtklock`, or compositor lock events), vibe renders a perimeter ambient glow and a centered or bottom-anchored frosted now-playing badge on the lock surface.
  - Automatically suspends interactive mouse event processing on lock surface.

### 3.8 Settings Interface and System Tray
- Settings GUI (Native Slint Declarative Framework):
  - 100% compiled native Rust binary with zero Node.js, zero WebKit, and zero Electron runtime dependencies.
  - Ultra-lean footprint: Under 15MB RAM, instant launch time, smooth 60-144 FPS interactivity.
  - Visual Theme: Matte-black luxury aesthetic (#09090b canvas, #121214 cards, #27272a hairline borders, #f4f4f5 text).
  - Interactive Live Preview Canvas: Built-in display preview showcasing real-time feedback as the user adjusts the squircle ambient aura and orbiting white tracer beam.
  - Intuitive Sliders matching Lumn controls:
    - Glow Thickness (1px - 150px)
    - Glow Diffusion / Blur Radius (0% - 100%)
    - Intensity / Opacity (0% - 100%)
    - White Tracer Beam: Toggle on/off, Orbit Velocity / Speed, Tail Length (15% - 100%), and Beat Burst Sensitivity
    - Beat Sensitivity (Low, Balanced, Aggressive)
    - Mode Selector (Orbit Tracer, Beat Pulse, Fluid Wave, Breathe, Steady)
    - Color Source (Album Art, Screen Edge, Preset Gradients)
- System Tray Applet:
  - Native StatusNotifierItem (`ksni`) menu using vector SVG icon (`assets/icons/tray-icon.svg`).
  - Quick toggles: Enable/Disable Ambient Glow, Toggle White Tracer Beam, Cycle Visualizer Mode, Toggle Now-Playing Card, Settings, Quit.
- Configuration Backend:
  - Human-readable TOML configuration at `~/.config/vibe/config.toml`.
  - Supports live file-watch reload on save (`notify`).

---

## 4. Technical Architecture

```
                                +-----------------------------+
                                |  Media Players (MPRIS2)     |
                                +--------------+--------------+
                                               | D-Bus
                                               v
+-----------------------+              +-----------------------+              +-----------------------+
|   PipeWire Stream     |              |   vibe-mpris Engine   |              |  wlr-screencopy       |
| (Sink Loopback Node)  |              | (Album Art & Palette) |              | (Screen Edge Sampler) |
+-----------+-----------+              +-----------+-----------+              +-----------+-----------+
            | PCM Frames                           | RGB Palette                          | Edge Pixels
            v                                      |                                      |
+-----------------------+                          |                                      |
|   vibe-audio Engine   |                          |                                      |
| (FFT & Beat Detector) |                          |                                      |
+-----------+-----------+                          |                                      |
            | Energy & Beat Signals                |                                      |
            +-------------------+                  |                                      |
                                |                  |                                      |
                                v                  v                                      |
                        +----------------------------------+                              |
                        |      vibe-render GPU Pipeline    |<-----------------------------+
                        |      (OpenGL / EGL / GLSL)       |
                        +-----------------+----------------+
                                          |
                        +-----------------+-----------------+
                        |                                   |
                        v                                   v
             +---------------------+             +---------------------+
             |  Ambient Edge Layer |             |  Liquid Glass Card  |
             |  (wlr-layer-shell)  |             | (ext-session-lock)  |
             +---------------------+             +---------------------+
```

---

## 5. Technology Stack and Dependencies

| Subsystem | Component | Purpose |
| :--- | :--- | :--- |
| Core Language | Rust (Edition 2024) | High performance, memory safety, minimal resource usage |
| Settings GUI & Widgets | `slint` (Native Rust declarative toolkit) | Ultra-lightweight native settings GUI, zero Node.js / WebKit dependencies, <15MB RAM |
| Wayland Protocol | `wayland-client`, `smithay-client-toolkit` | Native `wlr-layer-shell` and `ext-session-lock-v1` bindings |
| Rendering Engine | `glow` + `wayland-egl` (OpenGL 3.3 Core / GLES 3.0) | Universal hardware acceleration using native GLSL shaders; zero Vulkan driver dependency |
| Audio Capture | `pipewire-rs` / `libpipewire-0.3` | Non-intrusive output sink loopback monitoring |
| Audio Math | `rustfft` | Real-time frequency binning and transient onset calculation |
| Media D-Bus | `zbus` (Async D-Bus) | MPRIS2 track changes, player control, and metadata parsing |
| Color Quantization | `palette`, `image`, `color-thief-rs` | Fast k-means color extraction from album art |
| System Tray | `ksni` (StatusNotifierItem) | Modern Wayland/XDG desktop tray integration |
| Configuration | `serde`, `toml`, `notify` | Schema serialization and hot-reloading |

---

## 6. Performance and System Resource Budget

- Universal Hardware Compatibility: Seamless execution on legacy Intel HD Graphics (Sandy Bridge, Ivy Bridge, Haswell), budget AMD APUs, Raspberry Pi 4/5, virtual machines, and modern discrete GPUs without requiring Vulkan drivers.
- Idle State (No Audio Playing): Less than 0.2% CPU, 0% GPU, <15 MB RAM.
- Active Beat-Reactive State (60–144 FPS): Less than 1.0% CPU, <2% GPU, <30 MB RAM.
- Frame Time: Under 6ms render pass on standard Intel/AMD/Nvidia hardware.
- Audio-to-Glow Response Latency: Less than 15 milliseconds.

---

## 7. Implementation Roadmap

### Phase 1: Workspace and Foundation (Current)
- Initialize Rust Cargo workspace with modular crates (`vibe-core`, `vibe-audio`, `vibe-mpris`, `vibe-render`, `vibe-ui`).
- Implement `assets/icons/logo.svg` and `assets/icons/tray-icon.svg`.
- Implement MPRIS2 client listening to player metadata and extracting color palettes.

### Phase 2: Audio Loopback and Beat Engine
- Implement PipeWire loopback stream capturing default sink audio.
- Build discrete FFT and transient onset energy detector.
- Verify beat trigger signals against live music playback.

### Phase 3: Hardware-Accelerated Wayland Layer-Shell Renderer
- Create Wayland `wlr-layer-shell` surface with input pass-through.
- Write universal GLSL shaders (`glow` + EGL) supporting squircle contour, diffusion, and multi-color interpolation.
- Implement animation modes: Orbit Tracer, Beat Pulse, Fluid Wave, Ambient Breathe, Steady Glow.

### Phase 4: Frosted Liquid-Glass Card and Lockscreen Integration
- Develop floating draggable Now-Playing card with frosted glass shader and vector SVG controls.
- Implement `ext-session-lock-v1` companion lockscreen ambient surface.

### Phase 5: Settings Interface and System Tray
- Construct matte-black Slint Settings GUI with real-time sliders and preset selector.
- Implement `ksni` system tray menu with vector SVG glyph.
- Implement configuration file manager (`~/.config/vibe/config.toml`).

### Phase 6: Packaging and Documentation
- Comprehensive documentation without emojis.
- Arch Linux / CachyOS PKGBUILD for AUR.
- Automated GitHub Actions build pipeline.
