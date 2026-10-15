pub mod palette;
pub mod player;

pub use palette::{extract_palette_from_bytes, extract_palette_from_uri};
pub use player::MprisWatcher;
