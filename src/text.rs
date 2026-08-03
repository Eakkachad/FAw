//! PNG text rasterizer (`katSVG Text`).
//!
//! Rasterizes text into an RGB pixel buffer using embedded fonts via `ab_glyph`
//! (P5). Resolves the G4 gap: the PNG raster now carries readable text (title,
//! metrics, labels) instead of geometry-only placeholders.
//!
//! **Dual-font fallback:** Latin text renders with Inter; glyphs missing from
//! Inter (Thai, etc.) fall back to IBM Plex Sans Thai. Both are SIL OFL-1.1 and
//! embedded at compile time → offline and deterministic.

use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};

const INTER_TTF: &[u8] = include_bytes!("../assets/inter-regular.ttf");
const THAI_TTF: &[u8] = include_bytes!("../assets/ibm-plex-sans-thai-regular.ttf");

/// Rasterizer over an RGB pixel buffer (`w` width, 3 bytes per pixel).
pub struct TextRenderer {
    latin: FontArc,
    thai: FontArc,
}

impl TextRenderer {
    pub fn new() -> Self {
        Self {
            latin: FontArc::try_from_slice(INTER_TTF).expect("embedded Inter TTF must parse"),
            thai: FontArc::try_from_slice(THAI_TTF).expect("embedded Plex Thai TTF must parse"),
        }
    }

    /// Pick the font that has a glyph for `c` (Thai fallback for CJK/Thai).
    fn font_for<'a>(&'a self, c: char) -> &'a FontArc {
        if self.latin.glyph_id(c).0 != 0 {
            &self.latin
        } else {
            &self.thai
        }
    }

    /// Draw `text` starting at `(x, y)` (baseline) with `px` height and `color`,
    /// blending anti-aliased coverage into the RGB buffer.
    pub fn draw_text(
        &self,
        buf: &mut [u8],
        w: usize,
        h: usize,
        x: f32,
        y: f32,
        px: f32,
        color: (u8, u8, u8),
        text: &str,
    ) {
        let (r, g, b) = color;
        let mut pen_x = x;
        let mut last_is_thai = false;
        let scale = PxScale::from(px);

        for c in text.chars() {
            let is_thai = self.latin.glyph_id(c).0 == 0;
            if is_thai != last_is_thai {
                last_is_thai = is_thai;
            }
            let font = if is_thai { &self.thai } else { &self.latin };
            let scaled = font.as_scaled(scale);

            let gid = font.glyph_id(c);
            if gid.0 == 0 && c != ' ' {
                let space = font.glyph_id(' ');
                pen_x += scaled.h_advance(space);
                continue;
            }
            let glyph = gid.with_scale_and_position(scale, point(pen_x, y));
            if let Some(outline) = font.outline_glyph(glyph) {
                let bounds = outline.px_bounds();
                let min_x = bounds.min.x.floor().max(0.0) as usize;
                let min_y = bounds.min.y.floor().max(0.0) as usize;
                let max_x = bounds.max.x.ceil().min(w as f32) as usize;
                let max_y = bounds.max.y.ceil().min(h as f32) as usize;

                outline.draw(|gx, gy, coverage| {
                    let px = min_x + gx as usize;
                    let py = min_y + gy as usize;
                    if px < max_x && py < max_y && px < w && py < h {
                        let i = (py * w + px) * 3;
                        buf[i] = blend(buf[i], r, coverage);
                        buf[i + 1] = blend(buf[i + 1], g, coverage);
                        buf[i + 2] = blend(buf[i + 2], b, coverage);
                    }
                });
            }
            pen_x += scaled.h_advance(gid);
        }
    }

    /// Measure the advance width of `text` at `px` height (for centering).
    pub fn text_width(&self, px: f32, text: &str) -> f32 {
        let scale = PxScale::from(px);
        text.chars()
            .map(|c| {
                let font = self.font_for(c);
                let scaled = font.as_scaled(scale);
                let gid = font.glyph_id(c);
                if gid.0 == 0 && c != ' ' {
                    scaled.h_advance(font.glyph_id(' '))
                } else {
                    scaled.h_advance(gid)
                }
            })
            .sum()
    }

    /// Truncate `text` with an ellipsis so it fits within `budget_px` at `px`
    /// height (F6). Binary search on the longest fitting prefix.
    pub fn truncate_to_fit(&self, px: f32, budget_px: f32, text: &str) -> String {
        if self.text_width(px, text) <= budget_px {
            return text.to_string();
        }
        let mut lo = 0usize;
        let mut hi = text.chars().count();
        let ell = self.text_width(px, "…");
        while lo < hi {
            let mid = (lo + hi + 1) / 2;
            let prefix: String = text.chars().take(mid).collect();
            if self.text_width(px, &prefix) + ell <= budget_px {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        let cut: String = text.chars().take(lo).collect();
        format!("{cut}…")
    }
}

fn blend(bg: u8, fg: u8, a: f32) -> u8 {
    (bg as f32 * (1.0 - a) + fg as f32 * a).round() as u8
}
