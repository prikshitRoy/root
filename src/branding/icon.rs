//! Pure functions for generating the `🌿 root` RGBA window icon, Windows `.ico`, and `.svg`.

use eframe::egui;

/// Default width and height in pixels for the generated window and `.ico` icon.
pub const DEFAULT_ICON_SIZE: u32 = 64;

/// Number of bytes per RGBA pixel.
const BYTES_PER_PIXEL: usize = 4;

/// Maximum pixel dimension supported by a standard single-entry `.ico` directory header.
const MAX_ICO_DIMENSION: u32 = 256;

/// Size in bytes of the Windows `ICONDIR` + single `ICONDIRENTRY` header (`6 + 16`).
const ICO_DIRECTORY_HEADER_BYTES: u32 = 22;

/// Size in bytes of the Windows `BITMAPINFOHEADER` structure.
const DIB_HEADER_BYTES: u32 = 40;

/// Dark background color matching the `egui` header badge (`#1B1B1B`).
const BG_COLOR: [u8; 4] = [0x1B, 0x1B, 0x1B, 0xFF];

/// Soft monochrome leaf and stem fill color matching the `🌿` glyph (`#9A9A9A`).
const LEAF_COLOR: [u8; 4] = [0x9A, 0x9A, 0x9A, 0xFF];

/// Transparent pixel outside the rounded squircle badge.
const TRANSPARENT_PIXEL: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

/// Scalable vector (`root.svg`) logo, shared with the `.deb` package via `assets/root.svg`.
pub const APP_ICON_SVG: &str = include_str!("../../assets/root.svg");

/// Builds the native `egui::IconData` for the `root` and `root-installer` window titlebars and taskbar.
pub fn build_window_icon() -> egui::IconData {
    egui::IconData {
        rgba: build_logo_rgba(DEFAULT_ICON_SIZE),
        width: DEFAULT_ICON_SIZE,
        height: DEFAULT_ICON_SIZE,
    }
}

/// Builds a binary Windows `.ico` file containing the default `64x64` 3-leaf `root` icon.
pub fn build_app_ico_bytes() -> Vec<u8> {
    let rgba = build_logo_rgba(DEFAULT_ICON_SIZE);
    encode_rgba_to_ico(DEFAULT_ICON_SIZE, DEFAULT_ICON_SIZE, &rgba).unwrap_or_default()
}

/// Rasterizes the 3-leaf `🌿` badge into a `size x size` RGBA buffer.
pub fn build_logo_rgba(size: u32) -> Vec<u8> {
    let safe_size = size.max(1);
    let mut rgba = Vec::with_capacity((safe_size * safe_size) as usize * BYTES_PER_PIXEL);

    for y in 0..safe_size {
        for x in 0..safe_size {
            let pixel = classify_logo_pixel(x, y, safe_size);
            rgba.extend_from_slice(&pixel);
        }
    }

    rgba
}

/// Determines the RGBA color of a single normalized pixel `(nx, ny)` in `0.0..=1.0`.
fn classify_logo_pixel(x: u32, y: u32, size: u32) -> [u8; 4] {
    let nx = (x as f32 + 0.5) / size as f32;
    let ny = (y as f32 + 0.5) / size as f32;

    if !is_inside_rounded_box(nx, ny) {
        return TRANSPARENT_PIXEL;
    }

    if is_inside_three_leaf_emblem(nx, ny) {
        return LEAF_COLOR;
    }

    BG_COLOR
}

/// Returns `true` if `(nx, ny)` lies inside a rounded square with margin `0.06` and radius `0.18`.
fn is_inside_rounded_box(nx: f32, ny: f32) -> bool {
    let margin = 0.06;
    let radius = 0.18;

    if nx < margin || nx > 1.0 - margin || ny < margin || ny > 1.0 - margin {
        return false;
    }

    let inner_min = margin + radius;
    let inner_max = 1.0 - margin - radius;
    let dx = if nx < inner_min {
        inner_min - nx
    } else if nx > inner_max {
        nx - inner_max
    } else {
        0.0
    };
    let dy = if ny < inner_min {
        inner_min - ny
    } else if ny > inner_max {
        ny - inner_max
    } else {
        0.0
    };

    (dx * dx + dy * dy) <= (radius * radius)
}

/// Returns `true` if `(nx, ny)` falls inside the diagonal stem or any of the 3 pointed leaves.
fn is_inside_three_leaf_emblem(nx: f32, ny: f32) -> bool {
    // 1. Short diagonal stem at bottom-left
    let in_stem = is_inside_capsule(nx, ny, (0.22, 0.76), (0.41, 0.55), 0.042);

    // 2. Top-left pointed leaf
    let in_left_leaf = is_inside_pointed_leaf(nx, ny, (0.27, 0.37), -1.72, 0.21, 0.082);

    // 3. Main center-right diagonal leaf
    let in_main_leaf = is_inside_pointed_leaf(nx, ny, (0.56, 0.35), -0.785, 0.27, 0.105);

    // 4. Lower-right horizontal leaf
    let in_right_leaf = is_inside_pointed_leaf(nx, ny, (0.58, 0.58), -0.10, 0.20, 0.082);

    in_stem || in_left_leaf || in_main_leaf || in_right_leaf
}

/// Checks whether `(px, py)` lies within a rotated lenticular (pointed-tip) leaf shape.
fn is_inside_pointed_leaf(
    px: f32,
    py: f32,
    center: (f32, f32),
    angle_rad: f32,
    half_length: f32,
    max_half_width: f32,
) -> bool {
    let dx = px - center.0;
    let dy = py - center.1;
    let (sin_a, cos_a) = angle_rad.sin_cos();

    let along_axis = dx * cos_a + dy * sin_a;
    let across_axis = -dx * sin_a + dy * cos_a;

    let normalized_pos = (along_axis / half_length).abs();
    if normalized_pos > 1.0 {
        return false;
    }

    let allowed_half_width = max_half_width * (1.0 - normalized_pos * normalized_pos);
    across_axis.abs() <= allowed_half_width
}

/// Checks whether `(px, py)` is within `radius` distance of line segment `start -> end`.
fn is_inside_capsule(px: f32, py: f32, start: (f32, f32), end: (f32, f32), radius: f32) -> bool {
    let vx = end.0 - start.0;
    let vy = end.1 - start.1;
    let len_sq = vx * vx + vy * vy;
    let t = if len_sq > 0.0 {
        (((px - start.0) * vx + (py - start.1) * vy) / len_sq).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let closest_x = start.0 + t * vx;
    let closest_y = start.1 + t * vy;
    let dist_sq = (px - closest_x).powi(2) + (py - closest_y).powi(2);

    dist_sq <= radius * radius
}

/// Encodes a top-down RGBA pixel slice into a valid 32-bit Windows `.ico` binary buffer.
pub fn encode_rgba_to_ico(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    if width == 0 || height == 0 || width > MAX_ICO_DIMENSION || height > MAX_ICO_DIMENSION {
        return Err(format!(
            "Invalid .ico dimensions {width}x{height} (must be within 1..={MAX_ICO_DIMENSION})"
        ));
    }

    let expected_len = (width * height) as usize * BYTES_PER_PIXEL;
    if rgba.len() != expected_len {
        return Err(format!(
            "RGBA buffer length mismatch: expected {expected_len} bytes, got {}",
            rgba.len()
        ));
    }

    let mask_row_bytes = width.div_ceil(32) * 4;
    let mask_total_bytes = mask_row_bytes * height;
    let pixel_total_bytes = (width * height * 4) as u32;
    let dib_total_bytes = DIB_HEADER_BYTES + pixel_total_bytes + mask_total_bytes;

    let mut out = Vec::with_capacity((ICO_DIRECTORY_HEADER_BYTES + dib_total_bytes) as usize);
    write_ico_headers(&mut out, width, height, dib_total_bytes);

    // Write bottom-up BGRA pixel rows as required by the Windows DIB `.ico` format
    for y in (0..height).rev() {
        for x in 0..width {
            let idx = ((y * width + x) as usize) * BYTES_PER_PIXEL;
            out.extend_from_slice(&[rgba[idx + 2], rgba[idx + 1], rgba[idx], rgba[idx + 3]]);
        }
    }

    // Write 1-bit opacity AND mask padded to 32-bit boundaries
    out.resize(out.len() + mask_total_bytes as usize, 0u8);
    Ok(out)
}

/// Appends the 6-byte `ICONDIR`, 16-byte `ICONDIRENTRY`, and 40-byte `BITMAPINFOHEADER`.
fn write_ico_headers(out: &mut Vec<u8>, width: u32, height: u32, dib_total_bytes: u32) {
    let dir_width = if width == 256 { 0u8 } else { width as u8 };
    let dir_height = if height == 256 { 0u8 } else { height as u8 };

    // ICONDIR (6 bytes): reserved=0, type=1 (icon), count=1
    out.extend_from_slice(&[0, 0, 1, 0, 1, 0]);

    // ICONDIRENTRY (16 bytes)
    out.push(dir_width);
    out.push(dir_height);
    out.extend_from_slice(&[0, 0]); // palette count & reserved
    out.extend_from_slice(&1u16.to_le_bytes()); // color planes
    out.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
    out.extend_from_slice(&dib_total_bytes.to_le_bytes());
    out.extend_from_slice(&ICO_DIRECTORY_HEADER_BYTES.to_le_bytes());

    // BITMAPINFOHEADER (40 bytes): height is doubled in ICO headers to account for the AND mask
    out.extend_from_slice(&DIB_HEADER_BYTES.to_le_bytes());
    out.extend_from_slice(&(width as i32).to_le_bytes());
    out.extend_from_slice(&((height * 2) as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // planes
    out.extend_from_slice(&32u16.to_le_bytes()); // bpp
    out.extend_from_slice(&[0u8; 24]); // compression=0 and remaining DIB fields
}
