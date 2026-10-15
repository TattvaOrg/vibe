use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::time::sleep;
use vibe_core::state::AppState;
use vibe_core::types::MediaInfo;
use zbus::connection::Connection;
use zbus::zvariant::{OwnedValue, Value};

use crate::palette::extract_palette_from_uri;

fn extract_string(v: &OwnedValue) -> Option<String> {
    match &**v {
        Value::Str(s) => Some(s.as_str().to_string()),
        _ => None,
    }
}

fn extract_first_string(v: &OwnedValue) -> Option<String> {
    match &**v {
        Value::Str(s) => Some(s.as_str().to_string()),
        Value::Array(arr) => {
            if let Some(item) = arr.first() {
                match item {
                    Value::Str(s) => Some(s.as_str().to_string()),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
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
            sleep(Duration::from_millis(1000)).await;
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

        if mpris_players.is_empty() {
            let current = self.state.media();
            if current.playing {
                self.state.set_media(MediaInfo::default());
            }
            return Ok(());
        }

        for player_bus_name in mpris_players {
            let status_reply = connection
                .call_method(
                    Some(player_bus_name.as_str()),
                    "/org/mpris/MediaPlayer2",
                    Some("org.freedesktop.DBus.Properties"),
                    "Get",
                    &("org.mpris.MediaPlayer2.Player", "PlaybackStatus"),
                )
                .await;

            let is_playing = match status_reply {
                Ok(msg) => {
                    let val_res: Result<OwnedValue, _> = msg.body().deserialize();
                    if let Ok(val) = val_res {
                        extract_string(&val).map(|s| s == "Playing").unwrap_or(false)
                    } else {
                        false
                    }
                }
                Err(_) => false,
            };

            let metadata_reply = connection
                .call_method(
                    Some(player_bus_name.as_str()),
                    "/org/mpris/MediaPlayer2",
                    Some("org.freedesktop.DBus.Properties"),
                    "Get",
                    &("org.mpris.MediaPlayer2.Player", "Metadata"),
                )
                .await;

            if let Ok(msg) = metadata_reply {
                let dict_res: Result<HashMap<String, OwnedValue>, _> = msg.body().deserialize();

                if let Ok(metadata) = dict_res {
                    let title = metadata
                        .get("xesam:title")
                        .and_then(extract_string)
                        .unwrap_or_default();

                    let artist = metadata
                        .get("xesam:artist")
                        .and_then(extract_first_string)
                        .unwrap_or_default();

                    let album = metadata
                        .get("xesam:album")
                        .and_then(extract_string)
                        .unwrap_or_default();

                    let art_url = metadata
                        .get("mpris:artUrl")
                        .and_then(extract_string);

                    let old_media = self.state.media();
                    let art_changed = old_media.art_url != art_url;

                    if art_changed {
                        if let Some(ref url) = art_url {
                            if let Ok(palette) = extract_palette_from_uri(url) {
                                log::info!("Extracted dynamic palette from album art: {:?}", palette);
                                self.state.set_palette(palette);
                            }
                        }
                    }

                    self.state.set_media(MediaInfo {
                        title,
                        artist,
                        album,
                        art_url,
                        playing: is_playing,
                        position: 0.0,
                        duration: 0.0,
                    });

                    if is_playing {
                        break;
                    }
                }
            }
        }

        Ok(())
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
            connection
                .call_method(
                    Some(player.as_str()),
                    "/org/mpris/MediaPlayer2",
                    Some("org.mpris.MediaPlayer2.Player"),
                    method,
                    &(),
                )
                .await?;
        }

        Ok(())
    }
}
