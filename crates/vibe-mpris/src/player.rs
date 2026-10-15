use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::time::sleep;
use vibe_core::state::AppState;
use vibe_core::types::MediaInfo;
use zbus::connection::Connection;
use zbus::zvariant::{OwnedValue, Value};

use crate::palette::extract_palette_from_uri;

#[zbus::proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_path = "/org/mpris/MediaPlayer2"
)]
trait MediaPlayer2Player {
    #[zbus(property)]
    fn playback_status(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn metadata(&self) -> zbus::Result<HashMap<String, OwnedValue>>;

    fn play_pause(&self) -> zbus::Result<()>;
    fn next(&self) -> zbus::Result<()>;
    fn previous(&self) -> zbus::Result<()>;
}

/// Recursively unwraps variants, arrays, and boxed values into a trimmed string.
pub fn extract_string_recursive(v: &Value) -> Option<String> {
    match v {
        Value::Str(s) => {
            let st = s.as_str().trim();
            if !st.is_empty() {
                Some(st.to_string())
            } else {
                None
            }
        }
        Value::Value(inner) => extract_string_recursive(inner),
        Value::Array(arr) => {
            let mut parts = Vec::new();
            for item in arr.iter() {
                if let Some(s) = extract_string_recursive(item) {
                    parts.push(s);
                }
            }
            if parts.is_empty() {
                None
            } else {
                Some(parts.join(", "))
            }
        }
        _ => None,
    }
}

/// Extract clean filename from URL (e.g. file:///home/user/Music/My%20Song.mp3 -> My Song)
pub fn extract_clean_filename(url_str: &str) -> Option<String> {
    let clean = url_str.strip_prefix("file://").unwrap_or(url_str);
    let filename = clean.split('/').last()?;
    if filename.is_empty() {
        return None;
    }

    // Decode %20 and other percent-encoded chars
    let mut decoded = String::new();
    let mut chars = filename.chars();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    decoded.push(byte as char);
                    continue;
                }
            }
            decoded.push('%');
            decoded.push_str(&hex);
        } else {
            decoded.push(ch);
        }
    }

    // Strip common audio extensions
    let stem = if let Some(idx) = decoded.rfind('.') {
        let ext = &decoded[idx + 1..];
        match ext.to_lowercase().as_str() {
            "mp3" | "flac" | "wav" | "ogg" | "opus" | "m4a" | "aac" | "webm" | "mp4" => {
                &decoded[..idx]
            }
            _ => &decoded,
        }
    } else {
        &decoded
    };

    if stem.trim().is_empty() {
        None
    } else {
        Some(stem.trim().to_string())
    }
}

#[derive(Debug, Clone)]
struct PlayerCandidate {
    bus_name: String,
    title: String,
    artist: String,
    album: String,
    art_url: Option<String>,
    is_playing: bool,
    is_paused: bool,
}

/// Background watcher for MPRIS2 media players over D-Bus.
pub struct MprisWatcher {
    state: AppState,
}

impl MprisWatcher {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    /// Run the polling/listening loop for media metadata changes.
    pub async fn run(self) -> Result<()> {
        let connection = Connection::session().await.context("Connecting to D-Bus session bus")?;

        log::info!("MPRIS watcher initialized on D-Bus session bus");

        while self.state.is_running() {
            if let Err(e) = self.update_active_player(&connection).await {
                log::debug!("Error polling active MPRIS player: {:#}", e);
            }
            sleep(Duration::from_millis(500)).await;
        }

        Ok(())
    }

    async fn update_active_player(&self, connection: &Connection) -> Result<()> {
        let reply = connection
            .call_method(
                Some("org.freedesktop.DBus"),
                "/org/freedesktop/DBus",
                Some("org.freedesktop.DBus"),
                "ListNames",
                &(),
            )
            .await?;

        let names: Vec<String> = reply.body().deserialize()?;
        let mpris_players: Vec<String> = names
            .into_iter()
            .filter(|n| n.starts_with("org.mpris.MediaPlayer2."))
            .collect();

        let mut candidates: Vec<PlayerCandidate> = Vec::new();

        for player_bus_name in &mpris_players {
            let proxy_res = MediaPlayer2PlayerProxy::builder(connection)
                .destination(player_bus_name.as_str())
                .and_then(|b| b.path("/org/mpris/MediaPlayer2"))
                .map(|b| b.build());

            let proxy = match proxy_res {
                Ok(fut) => match fut.await {
                    Ok(p) => p,
                    Err(_) => continue,
                },
                Err(_) => continue,
            };

            let status = proxy.playback_status().await.unwrap_or_default();
            let is_playing = status.eq_ignore_ascii_case("Playing");
            let is_paused = status.eq_ignore_ascii_case("Paused");

            let metadata = proxy.metadata().await.unwrap_or_default();

            let mut title = metadata
                .get("xesam:title")
                .and_then(|v| extract_string_recursive(v))
                .unwrap_or_default();

            // If title is missing (common with local files without ID3 tags), extract from URL
            if title.is_empty() {
                if let Some(url_str) = metadata.get("xesam:url").and_then(|v| extract_string_recursive(v)) {
                    if let Some(filename) = extract_clean_filename(&url_str) {
                        title = filename;
                    }
                }
            }

            let artist = metadata
                .get("xesam:artist")
                .and_then(|v| extract_string_recursive(v))
                .unwrap_or_default();

            let album = metadata
                .get("xesam:album")
                .and_then(|v| extract_string_recursive(v))
                .unwrap_or_default();

            let art_url = metadata
                .get("mpris:artUrl")
                .and_then(|v| extract_string_recursive(v));

            if !title.is_empty() || is_playing || is_paused {
                candidates.push(PlayerCandidate {
                    bus_name: player_bus_name.clone(),
                    title,
                    artist,
                    album,
                    art_url,
                    is_playing,
                    is_paused,
                });
            }
        }

        // Secondary fallback: playerctl if installed and no candidate found
        if candidates.is_empty() {
            if let Some(cand) = Self::query_playerctl_fallback() {
                candidates.push(cand);
            }
        }

        // Pick the best candidate
        // Priority:
        // 1. is_playing && !title.is_empty()
        // 2. is_playing
        // 3. is_paused && !title.is_empty()
        // 4. !title.is_empty()
        candidates.sort_by_key(|c| {
            if c.is_playing && !c.title.is_empty() {
                0
            } else if c.is_playing {
                1
            } else if c.is_paused && !c.title.is_empty() {
                2
            } else if !c.title.is_empty() {
                3
            } else {
                4
            }
        });

        if let Some(best) = candidates.first() {
            let old_media = self.state.media();
            let art_changed = old_media.art_url != best.art_url;

            if art_changed {
                if let Some(ref url) = best.art_url {
                    if let Ok(palette) = extract_palette_from_uri(url) {
                        log::info!("Extracted dynamic palette from album art: {:?}", palette);
                        self.state.set_palette(palette);
                    }
                }
            }

            let effective_artist = if best.artist.is_empty() {
                if best.bus_name.contains("brave") {
                    "Brave Browser".to_string()
                } else if best.bus_name.contains("firefox") {
                    "Firefox".to_string()
                } else if best.bus_name.contains("chromium") || best.bus_name.contains("chrome") {
                    "Chrome".to_string()
                } else if best.bus_name.contains("spotify") {
                    "Spotify".to_string()
                } else if best.bus_name.contains("vlc") {
                    "VLC Media Player".to_string()
                } else if best.bus_name.contains("mpv") {
                    "MPV".to_string()
                } else {
                    "Media Player".to_string()
                }
            } else {
                best.artist.clone()
            };

            self.state.set_media(MediaInfo {
                title: best.title.clone(),
                artist: effective_artist,
                album: best.album.clone(),
                art_url: best.art_url.clone(),
                playing: best.is_playing,
                position: 0.0,
                duration: 0.0,
            });
        } else {
            // No player metadata found. Check if audio is currently playing through PipeWire/ALSA loopback!
            let audio = self.state.audio();
            if audio.energy > 0.02 {
                self.state.set_media(MediaInfo {
                    title: "Active Audio Stream".to_string(),
                    artist: "PipeWire Capture".to_string(),
                    album: String::new(),
                    art_url: None,
                    playing: true,
                    position: 0.0,
                    duration: 0.0,
                });
            } else {
                let current = self.state.media();
                if current.playing || !current.title.is_empty() {
                    self.state.set_media(MediaInfo::default());
                }
            }
        }

        Ok(())
    }

    /// Fallback query via playerctl CLI if D-Bus discovery missed a player
    fn query_playerctl_fallback() -> Option<PlayerCandidate> {
        let status_output = std::process::Command::new("playerctl")
            .arg("status")
            .output()
            .ok()?;

        if !status_output.status.success() {
            return None;
        }

        let status_str = String::from_utf8_lossy(&status_output.stdout).trim().to_string();
        let is_playing = status_str.eq_ignore_ascii_case("Playing");
        let is_paused = status_str.eq_ignore_ascii_case("Paused");

        let meta_output = std::process::Command::new("playerctl")
            .args(["metadata", "--format", "{{title}}||{{artist}}||{{album}}||{{mpris:artUrl}}"])
            .output()
            .ok()?;

        if !meta_output.status.success() {
            return None;
        }

        let line = String::from_utf8_lossy(&meta_output.stdout).trim().to_string();
        let parts: Vec<&str> = line.split("||").collect();
        let title = parts.get(0).copied().unwrap_or("").trim().to_string();
        let artist = parts.get(1).copied().unwrap_or("").trim().to_string();
        let album = parts.get(2).copied().unwrap_or("").trim().to_string();
        let art_url = parts.get(3).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());

        if title.is_empty() && !is_playing {
            return None;
        }

        Some(PlayerCandidate {
            bus_name: "playerctl".to_string(),
            title,
            artist,
            album,
            art_url,
            is_playing,
            is_paused,
        })
    }

    /// Trigger play/pause on the first active MPRIS player
    pub async fn play_pause() -> Result<()> {
        Self::call_player_action("PlayPause").await
    }

    /// Trigger next on the first active MPRIS player
    pub async fn next() -> Result<()> {
        Self::call_player_action("Next").await
    }

    /// Trigger previous on the first active MPRIS player
    pub async fn previous() -> Result<()> {
        Self::call_player_action("Previous").await
    }

    async fn call_player_action(method: &str) -> Result<()> {
        let connection = Connection::session().await?;
        let reply = connection
            .call_method(
                Some("org.freedesktop.DBus"),
                "/org/freedesktop/DBus",
                Some("org.freedesktop.DBus"),
                "ListNames",
                &(),
            )
            .await?;

        let names: Vec<String> = reply.body().deserialize()?;
        if let Some(player) = names.into_iter().find(|n| n.starts_with("org.mpris.MediaPlayer2.")) {
            let _ = connection
                .call_method(
                    Some(player.as_str()),
                    "/org/mpris/MediaPlayer2",
                    Some("org.mpris.MediaPlayer2.Player"),
                    method,
                    &(),
                )
                .await;
        }

        Ok(())
    }
}
