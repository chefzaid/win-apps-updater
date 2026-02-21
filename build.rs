use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let icon_path = Path::new(&out_dir).join("icon.ico");
    generate_icon(&icon_path);

    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon(icon_path.to_str().unwrap());
        res.compile().unwrap();
    }
}

// ── Icon generation ──────────────────────────────────────────────────

/// Generates a multi-resolution `.ico` file with a circular-arrow update icon.
fn generate_icon(path: &Path) {
    let sizes: &[u32] = &[16, 32, 48];
    let images: Vec<Vec<u8>> = sizes.iter().map(|&s| build_bmp_icon(s)).collect();

    // ICO header: reserved(2) + type(2) + count(2)
    let num = sizes.len() as u16;
    let mut ico = Vec::new();
    ico.extend_from_slice(&0u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes()); // type = ICO
    ico.extend_from_slice(&num.to_le_bytes());

    // Directory entries then image data
    let dir_end = 6 + sizes.len() * 16;
    let mut offset = dir_end;

    for (i, &size) in sizes.iter().enumerate() {
        let w = if size >= 256 { 0u8 } else { size as u8 };
        ico.push(w); // width
        ico.push(w); // height
        ico.push(0); // colour count
        ico.push(0); // reserved
        ico.extend_from_slice(&1u16.to_le_bytes()); // colour planes
        ico.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
        ico.extend_from_slice(&(images[i].len() as u32).to_le_bytes());
        ico.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += images[i].len();
    }

    for img in &images {
        ico.extend_from_slice(img);
    }

    std::fs::write(path, ico).unwrap();
}

/// Builds a BMP DIB (BITMAPINFOHEADER + pixels + AND mask) for one icon size.
fn build_bmp_icon(size: u32) -> Vec<u8> {
    let center = size as f32 / 2.0;
    let scale = size as f32 / 32.0;
    let inner_r = 7.5 * scale;
    let outer_r = 12.0 * scale;
    let arrow_ext = 4.0 * scale;

    let row_bytes = (size * 4) as usize;
    let mut pixels = vec![0u8; row_bytes * size as usize];

    for y in 0..size {
        let buf_row = (size - 1 - y) as usize; // BMP is bottom-up
        for x in 0..size {
            let idx = buf_row * row_bytes + (x as usize) * 4;
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);

            let in_ring = dist > inner_r && dist < outer_r;
            let gap = angle > -0.6 && angle < 0.9;
            let arrow =
                dist > outer_r && dist < outer_r + arrow_ext && angle > -0.2 && angle < 0.9 && dx > 0.0;

            if (in_ring && !gap) || arrow {
                pixels[idx] = 237; // B
                pixels[idx + 1] = 143; // G
                pixels[idx + 2] = 77; // R
                pixels[idx + 3] = 255; // A
            }
        }
    }

    // AND mask (all zeros — alpha channel is authoritative for 32-bit icons)
    let mask_row = (size.div_ceil(32) * 4) as usize;
    let and_mask = vec![0u8; mask_row * size as usize];

    // BITMAPINFOHEADER (40 bytes)
    let mut bmp = Vec::with_capacity(40 + pixels.len() + and_mask.len());
    bmp.extend_from_slice(&40u32.to_le_bytes()); // biSize
    bmp.extend_from_slice(&(size as i32).to_le_bytes()); // biWidth
    bmp.extend_from_slice(&((size * 2) as i32).to_le_bytes()); // biHeight (×2 for ICO)
    bmp.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    bmp.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    bmp.extend_from_slice(&0u32.to_le_bytes()); // biCompression
    let img_size = (pixels.len() + and_mask.len()) as u32;
    bmp.extend_from_slice(&img_size.to_le_bytes()); // biSizeImage
    bmp.extend_from_slice(&0i32.to_le_bytes()); // biXPelsPerMeter
    bmp.extend_from_slice(&0i32.to_le_bytes()); // biYPelsPerMeter
    bmp.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
    bmp.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant

    bmp.extend_from_slice(&pixels);
    bmp.extend_from_slice(&and_mask);
    bmp
}

