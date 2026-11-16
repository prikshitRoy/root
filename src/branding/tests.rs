//! Unit tests for `branding` icon generation and `.ico` binary encoding.

use crate::branding::{
    build_app_ico_bytes, build_logo_rgba, build_window_icon, encode_rgba_to_ico, APP_DISPLAY_TITLE,
    APP_ICON_SVG, DEFAULT_ICON_SIZE,
};

#[test]
fn builds_window_icon_with_expected_dimensions_and_rgba_buffer_length() {
    // Arrange
    let expected_bytes = (DEFAULT_ICON_SIZE * DEFAULT_ICON_SIZE * 4) as usize;

    // Act
    let icon = build_window_icon();

    // Assert
    assert_eq!(icon.width, DEFAULT_ICON_SIZE);
    assert_eq!(icon.height, DEFAULT_ICON_SIZE);
    assert_eq!(icon.rgba.len(), expected_bytes);
}

#[test]
fn renders_transparent_corners_and_three_leaf_glyph_pixels() {
    // Arrange
    let size = 64;

    // Act
    let rgba = build_logo_rgba(size);
    let top_left_alpha = rgba[3];
    // Pixel (35, 22) sits at the center of the main diagonal leaf (0.56, 0.35)
    let leaf_index = ((22 * size + 35) * 4) as usize;
    let leaf_pixel = [
        rgba[leaf_index],
        rgba[leaf_index + 1],
        rgba[leaf_index + 2],
        rgba[leaf_index + 3],
    ];

    // Assert: outside the rounded corner is transparent; main leaf pixel is #9A9A9A
    assert_eq!(top_left_alpha, 0);
    assert_eq!(leaf_pixel, [0x9A, 0x9A, 0x9A, 0xFF]);
}

#[test]
fn encodes_valid_windows_ico_header_and_bgra_payload() {
    // Arrange: 2x2 RGBA image with a known red pixel at (0, 0)
    let rgba = vec![
        255, 0, 0, 255, // (0, 0) red
        0, 255, 0, 255, // (1, 0) green
        0, 0, 255, 255, // (0, 1) blue
        255, 255, 255, 255, // (1, 1) white
    ];

    // Act
    let ico = encode_rgba_to_ico(2, 2, &rgba).unwrap();

    // Assert: ICONDIR magic bytes [0, 0, 1, 0, 1, 0] and 2x2 dimensions at bytes 6..8
    assert_eq!(&ico[0..6], &[0, 0, 1, 0, 1, 0]);
    assert_eq!(ico[6], 2);
    assert_eq!(ico[7], 2);
}

#[test]
fn returns_error_when_ico_dimensions_or_buffer_length_are_invalid() {
    // Arrange
    let valid_pixel = vec![0u8; 4];

    // Act
    let zero_dim_result = encode_rgba_to_ico(0, 16, &valid_pixel);
    let oversize_result = encode_rgba_to_ico(512, 16, &valid_pixel);
    let bad_len_result = encode_rgba_to_ico(4, 4, &valid_pixel);

    // Assert
    assert!(zero_dim_result.is_err());
    assert!(oversize_result.is_err());
    assert!(bad_len_result.is_err());
}

#[test]
fn includes_sprout_branding_in_titles_and_default_ico_output() {
    // Arrange & Act
    let ico_bytes = build_app_ico_bytes();

    // Assert
    assert!(APP_DISPLAY_TITLE.starts_with("🌿"));
    assert!(APP_ICON_SVG.contains("<svg"));
    assert!(ico_bytes.len() > 62);
}
