/// Creates a window icon with an update/refresh symbol
pub fn create_icon() -> Option<iced::window::Icon> {
    const SIZE: u32 = 32;
    const CENTER: f32 = 16.0;
    const INNER_RADIUS: f32 = 8.0;
    const OUTER_RADIUS: f32 = 12.0;
    const ARROW_RADIUS: f32 = 14.0;

    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = ((y * SIZE + x) * 4) as usize;
            let dx = x as f32 - CENTER;
            let dy = y as f32 - CENTER;
            let dist = (dx * dx + dy * dy).sqrt();

            // Draw circular ring or arrow head
            if (dist > INNER_RADIUS && dist < OUTER_RADIUS)
                || (dist > OUTER_RADIUS && dist < ARROW_RADIUS && x > 16 && y < 16)
            {
                set_pixel(&mut rgba, idx, 100, 150, 255, 255);
            }
        }
    }

    iced::window::icon::from_rgba(rgba, SIZE, SIZE).ok()
}

/// Helper function to set RGBA pixel values
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
}

