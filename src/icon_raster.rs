//! Icon rendering for raster/vector formats beyond SVG (`katSVG Icon Raster`).
//!
//! F3: draws icon glyphs (from the icon corpus) as raster strokes into an RGB
//! pixel buffer, and as PDF stroke ops. Uses the shared `icon_paths` tokenizer.

use crate::icon_paths::parse_path;
use crate::IconRenderer;

/// Draw an icon named `name` into an RGB buffer centered on `(cx, cy)` with
/// stroke width `stroke` (the icon is a 24×24 box).
pub fn draw_icon_raster(buf: &mut [u8], w: usize, h: usize, cx: usize, cy: usize, stroke: u32, color: (u8, u8, u8), name: &str) {
    let origin_x = cx as i32 - 12;
    let origin_y = cy as i32 - 12;
    for path in IconRenderer::paths(name) {
        let cmds = parse_path(&path);
        let mut cur = (0.0f32, 0.0f32);
        let mut start = (0.0f32, 0.0f32);
        for pc in &cmds {
            let c = pc.cmd;
            let a = &pc.args;
            match c {
                'M' => {
                    if let (Some(&x), Some(&y)) = (a.first(), a.get(1)) {
                        cur = (x, y);
                        start = cur;
                    }
                }
                'm' => {
                    if let (Some(&dx), Some(&dy)) = (a.first(), a.get(1)) {
                        cur = (cur.0 + dx, cur.1 + dy);
                        start = cur;
                    }
                }
                'L' => {
                    if let (Some(&x), Some(&y)) = (a.first(), a.get(1)) {
                        let (sx, sy) = to_px(cur, origin_x, origin_y);
                        let (ex, ey) = to_px((x, y), origin_x, origin_y);
                        draw_line(buf, w, h, sx, sy, ex, ey, stroke, color);
                        cur = (x, y);
                    }
                }
                'l' => {
                    if let (Some(&dx), Some(&dy)) = (a.first(), a.get(1)) {
                        let end = (cur.0 + dx, cur.1 + dy);
                        let (sx, sy) = to_px(cur, origin_x, origin_y);
                        let (ex, ey) = to_px(end, origin_x, origin_y);
                        draw_line(buf, w, h, sx, sy, ex, ey, stroke, color);
                        cur = end;
                    }
                }
                'H' => {
                    if let Some(&x) = a.first() {
                        let (sx, sy) = to_px(cur, origin_x, origin_y);
                        let (ex, _) = to_px((x, cur.1), origin_x, origin_y);
                        draw_line(buf, w, h, sx, sy, ex, sy, stroke, color);
                        cur = (x, cur.1);
                    }
                }
                'h' => {
                    if let Some(&dx) = a.first() {
                        let end = (cur.0 + dx, cur.1);
                        let (sx, sy) = to_px(cur, origin_x, origin_y);
                        let (ex, _) = to_px(end, origin_x, origin_y);
                        draw_line(buf, w, h, sx, sy, ex, sy, stroke, color);
                        cur = end;
                    }
                }
                'V' => {
                    if let Some(&y) = a.first() {
                        let (sx, sy) = to_px(cur, origin_x, origin_y);
                        let (_, ey) = to_px((cur.0, y), origin_x, origin_y);
                        draw_line(buf, w, h, sx, sy, sx, ey, stroke, color);
                        cur = (cur.0, y);
                    }
                }
                'v' => {
                    if let Some(&dy) = a.first() {
                        let end = (cur.0, cur.1 + dy);
                        let (sx, sy) = to_px(cur, origin_x, origin_y);
                        let (_, ey) = to_px(end, origin_x, origin_y);
                        draw_line(buf, w, h, sx, sy, sx, ey, stroke, color);
                        cur = end;
                    }
                }
                'A' | 'a' | 'C' | 'c' => {
                    // approximate arcs/curves as a chord to the endpoint
                    let off = if c == 'C' || c == 'c' { 4 } else { 5 };
                    if let (Some(&x), Some(&y)) = (a.get(off), a.get(off + 1)) {
                        let end = if c == 'a' || c == 'c' { (cur.0 + x, cur.1 + y) } else { (x, y) };
                        let (sx, sy) = to_px(cur, origin_x, origin_y);
                        let (ex, ey) = to_px(end, origin_x, origin_y);
                        draw_line(buf, w, h, sx, sy, ex, ey, stroke, color);
                        cur = end;
                    }
                }
                'Z' | 'z' => {
                    let (sx, sy) = to_px(cur, origin_x, origin_y);
                    let (ex, ey) = to_px(start, origin_x, origin_y);
                    draw_line(buf, w, h, sx, sy, ex, ey, stroke, color);
                    cur = start;
                }
                _ => {}
            }
        }
    }
}

/// Append PDF operators drawing `name` as a stroked path centered on `(cx, cy)`.
pub fn draw_icon_pdf(stream: &mut String, cx: f32, cy: f32, linewidth: f32, r: f32, g: f32, b: f32, name: &str) {
    let ox = cx - 12.0;
    let oy = cy - 12.0;
    stream.push_str(&format!("{r:.3} {g:.3} {b:.3} RG {linewidth} w\n"));
    for path in IconRenderer::paths(name) {
        let cmds = parse_path(&path);
        let mut cur = (0.0f32, 0.0f32);
        let mut start = (0.0f32, 0.0f32);
        let mut started = false;
        for pc in &cmds {
            let c = pc.cmd;
            let a = &pc.args;
            match c {
                'M' => {
                    if let (Some(&x), Some(&y)) = (a.first(), a.get(1)) {
                        cur = (x, y);
                        start = cur;
                        stream.push_str(&format!("{:.1} {:.1} m\n", ox + cur.0, oy + cur.1));
                        started = true;
                    }
                }
                'm' => {
                    if let (Some(&dx), Some(&dy)) = (a.first(), a.get(1)) {
                        cur = (cur.0 + dx, cur.1 + dy);
                        start = cur;
                        stream.push_str(&format!("{:.1} {:.1} m\n", ox + cur.0, oy + cur.1));
                        started = true;
                    }
                }
                'L' | 'l' | 'H' | 'h' | 'V' | 'v' | 'A' | 'a' | 'C' | 'c' => {
                    let end = resolve_end(c, a, cur);
                    stream.push_str(&format!("{:.1} {:.1} l\n", ox + end.0, oy + end.1));
                    cur = end;
                    let _ = started;
                }
                'Z' | 'z' => {
                    stream.push_str(&format!("{:.1} {:.1} l\n", ox + start.0, oy + start.1));
                    cur = start;
                }
                _ => {}
            }
        }
        stream.push_str("S\n");
    }
}

fn resolve_end(cmd: char, a: &[f32], cur: (f32, f32)) -> (f32, f32) {
    match cmd {
        'L' => (a.first().copied().unwrap_or(cur.0), a.get(1).copied().unwrap_or(cur.1)),
        'l' => (cur.0 + a.first().copied().unwrap_or(0.0), cur.1 + a.get(1).copied().unwrap_or(0.0)),
        'H' => (a.first().copied().unwrap_or(cur.0), cur.1),
        'h' => (cur.0 + a.first().copied().unwrap_or(0.0), cur.1),
        'V' => (cur.0, a.first().copied().unwrap_or(cur.1)),
        'v' => (cur.0, cur.1 + a.first().copied().unwrap_or(0.0)),
        'A' | 'a' => {
            let (x, y) = (a.get(5).copied().unwrap_or(cur.0), a.get(6).copied().unwrap_or(cur.1));
            if cmd == 'a' { (cur.0 + x, cur.1 + y) } else { (x, y) }
        }
        'C' | 'c' => {
            let (x, y) = (a.get(4).copied().unwrap_or(cur.0), a.get(5).copied().unwrap_or(cur.1));
            if cmd == 'c' { (cur.0 + x, cur.1 + y) } else { (x, y) }
        }
        _ => cur,
    }
}

fn set(buf: &mut [u8], w: usize, h: usize, px: i32, py: i32, c: (u8, u8, u8)) {
    if px >= 0 && py >= 0 && (px as usize) < w && (py as usize) < h {
        let i = ((py as usize) * w + px as usize) * 3;
        buf[i] = c.0;
        buf[i + 1] = c.1;
        buf[i + 2] = c.2;
    }
}

fn draw_line(buf: &mut [u8], w: usize, h: usize, x0: i32, y0: i32, x1: i32, y1: i32, stroke: u32, c: (u8, u8, u8)) {
    let dx = (x1 - x0).abs().max(1);
    let dy = (y1 - y0).abs().max(1);
    let steps = dx.max(dy);
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let x = x0 as f32 + (x1 - x0) as f32 * t;
        let y = y0 as f32 + (y1 - y0) as f32 * t;
        for sy in 0..stroke {
            for sx in 0..stroke {
                set(buf, w, h, x as i32 + sx as i32 - (stroke as i32 / 2), y as i32 + sy as i32 - (stroke as i32 / 2), c);
            }
        }
    }
}

fn to_px(pt: (f32, f32), ox: i32, oy: i32) -> (i32, i32) {
    (ox + pt.0.round() as i32, oy + pt.1.round() as i32)
}
