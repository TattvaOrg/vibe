use vibe_mpris::player::{extract_clean_filename, extract_string_recursive};
use zbus::zvariant::Value;

#[test]
fn test_recursive_value_extraction() {
    // Nested variant wrapping string
    let inner_str = Value::Str("Test Track".into());
    let boxed_variant = Value::Value(Box::new(inner_str));
    assert_eq!(extract_string_recursive(&boxed_variant), Some("Test Track".to_string()));

    // Nested variant wrapping array of strings (xesam:artist)
    let a1 = Value::Str("Artist 1".into());
    let a2 = Value::Str("Artist 2".into());
    let arr = Value::Array(vec![a1, a2].into());
    let boxed_arr = Value::Value(Box::new(arr));
    assert_eq!(extract_string_recursive(&boxed_arr), Some("Artist 1, Artist 2".to_string()));
}

#[test]
fn test_downloaded_file_name_extraction() {
    let url1 = "file:///home/user/Music/Alan%20Walker%20-%20Faded.mp3";
    assert_eq!(extract_clean_filename(url1), Some("Alan Walker - Faded".to_string()));

    let url2 = "file:///home/user/Downloads/synthwave_track_01.flac";
    assert_eq!(extract_clean_filename(url2), Some("synthwave_track_01".to_string()));

    let url3 = "/tmp/ambient_lofi_beat.opus";
    assert_eq!(extract_clean_filename(url3), Some("ambient_lofi_beat".to_string()));
}
