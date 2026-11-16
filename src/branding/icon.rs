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

/// Dark charcoal squircle background color (`#181C24`).
const BG_COLOR: [u8; 4] = [0x18, 0x1C, 0x24, 0xFF];

/// Primary bright mint leaf and stem color (`#4ADE80`).
const LEAF_BRIGHT_COLOR: [u8; 4] = [0x4A, 0xDE, 0x80, 0xFF];

/// Secondary emerald leaf and root node accent color (`#22C55E`).
const LEAF_ACCENT_COLOR: [u8; 4] = [0x22, 0xC5, 0x5E, 0xFF];

/// Transparent pixel outside the rounded squircle badge.
const TRANSPARENT_PIXEL: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

/// Scalable vector (`root.svg`) representation of the `🌿 root` logo.
pub const APP_ICON_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128">
  <rect width="128" height="128" rx="28" fill="#181c24"/>
  <path d="M64 102V46" stroke="#4ade80" stroke-width="10" stroke-linecap="round"/>
  <path d="M64 90L44 106M64 90L84 106" stroke="#22c55e" stroke-width="8" stroke-linecap="round"/>
  <path d="M64 62C64 42 86 30 98 34C98 52 82 64 64 62Z" fill="#4ade80"/>
  <path d="M64 72C64 54 44 44 30 48C30 64 46 74 64 72Z" fill="#22c55e"/>
</svg>
"##;

/// Builds the native `egui::IconData` for the `root` and `root-installer` window titlebars and taskbar.
pub fn build_window_icon() -> egui::IconData {
    egui::IconData {
        rgba: build_logo_rgba(DEFAULT_ICON_SIZE),
        width: DEFAULT_ICON_SIZE,
        height: DEFAULT_ICON_SIZE,
    }
}

/// Builds a binary Windows `.ico` file containing the default `64x64` `🌿 root` icon.
pub fn build_app_ico_bytes() -> Vec<u8> {
    let rgba = build_logo_rgba(DEFAULT_ICON_SIZE);
    encode_rgba_to_ico(DEFAULT_ICON_SIZE, DEFAULT_ICON_SIZE, &rgba).unwrap_or_default()
}

/// Rasterizes the `🌿 root` squircle badge and sprout emblem into a `size x size` RGBA buffer.
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

    if let Some(emblem_color) = sample_sprout_emblem(nx, ny) {
        return emblem_color;
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

/// Samples the `🌿` central stem, upper-right leaf, middle-left leaf, and lower root branches.
fn sample_sprout_emblem(nx: f32, ny: f32) -> Option<[u8; 4]> {
    // Upper-right tilted leaf ellipse
    let rx = (nx - 0.63) * 0.8 + (ny - 0.37) * -0.6;
    let ry = (nx - 0.63) * 0.6 + (ny - 0.37) * 0.8;
    if (rx / 0.16).powi(2) + (ry / 0.09).powi(2) <= 1.0 {
        return Some(LEAF_BRIGHT_COLOR);
    }

    // Middle-left tilted leaf ellipse
    let lx = (nx - 0.37) * 0.8 + (ny - 0.47) * 0.6;
    let ly = (nx - 0.37) * -0.6 + (ny - 0.47) * 0.8;
    if (lx / 0.16).powi(2) + (ly / 0.09).powi(2) <= 1.0 {
        return Some(LEAF_ACCENT_COLOR);
    }

    // Central vertical stem
    if (nx - 0.50).abs() <= 0.042 && (0.34..=0.78).contains(&ny) {
        return Some(LEAF_BRIGHT_COLOR);
    }

    // Bottom forked root branches
    if (0.68..=0.84).contains(&ny) {
        let spread = (ny - 0.68) * 0.85;
        if ((nx - 0.50).abs() - spread).abs() <= 0.045 {
            return Some(LEAF_ACCENT_COLOR);
        }
    }

    None
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
