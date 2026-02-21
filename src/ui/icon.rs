/// Creates a window icon with a circular-arrow update symbol.
pub fn create_icon() -> Option<iced::window::Icon> {
    const SIZE: u32 = 32;
    const CENTER: f32 = 16.0;
    const INNER_R: f32 = 7.5;
    const OUTER_R: f32 = 12.0;

    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = ((y * SIZE + x) * 4) as usize;
            let dx = x as f32 - CENTER;
            let dy = y as f32 - CENTER;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);

            // Draw a 270-degree arc (ring) leaving a gap in the upper-right.
            let in_ring = dist > INNER_R && dist < OUTER_R;
            let gap = angle > -0.6 && angle < 0.9; // ~34° to ~52° gap

            // Arrow-head triangle at the gap boundary (upper-right).
            let arrow = dist > OUTER_R
                && dist < OUTER_R + 4.0
                && angle > -0.2
                && angle < 0.9
                && (x as f32) > CENTER;

            if (in_ring && !gap) || arrow {
                // Accent blue: rgb(77, 143, 237) ≈ Color(0.30, 0.56, 0.93)
                set_pixel(&mut rgba, idx, 77, 143, 237, 255);
            }
        }
    }

    iced::window::icon::from_rgba(rgba, SIZE, SIZE).ok()
}

/// Helper function to set RGBA pixel values.
#[inline]
fn set_pixel(rgba: &mut [u8], idx: usize, r: u8, g: u8, b: u8, a: u8) {
    rgba[idx] = r;
    rgba[idx + 1] = g;
    rgba[idx + 2] = b;
    rgba[idx + 3] = a;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_icon_returns_some() {
        let icon = create_icon();
        assert!(icon.is_some());
    }

    #[test]
    fn test_set_pixel() {
        let mut rgba = vec![0u8; 16]; // 4 pixels
        set_pixel(&mut rgba, 0, 255, 128, 64, 32);

        assert_eq!(rgba[0], 255);
        assert_eq!(rgba[1], 128);
        assert_eq!(rgba[2], 64);
        assert_eq!(rgba[3], 32);
    }

    #[test]
    fn test_icon_has_non_transparent_pixels() {
        const SIZE: u32 = 32;
        let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

        // Replicate the drawing logic to verify it produces visible pixels
        let center = 16.0_f32;
        let inner_r = 7.5_f32;
        let outer_r = 12.0_f32;
        let mut colored_pixels = 0u32;

        for y in 0..SIZE {
            for x in 0..SIZE {
                let idx = ((y * SIZE + x) * 4) as usize;
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let dist = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx);

                let in_ring = dist > inner_r && dist < outer_r;
                let gap = angle > -0.6 && angle < 0.9;
                let arrow = dist > outer_r
                    && dist < outer_r + 4.0
                    && angle > -0.2
                    && angle < 0.9
                    && (x as f32) > center;

                if (in_ring && !gap) || arrow {
                    set_pixel(&mut rgba, idx, 77, 143, 237, 255);
                    colored_pixels += 1;
                }
            }
        }

        assert!(
            colored_pixels > 50,
            "Icon should have a significant number of coloured pixels, got {colored_pixels}"
        );
    }
}

