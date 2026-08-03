//! Shared SVG path tokenizer for icon rendering (`katSVG Icon Paths`).
//!
//! Parses the spaced command syntax used by the icon corpus into a flat list of
//! `(command, [x, y...])` segments, so raster and PDF renderers share one
//! tokenizer instead of each re-implementing path parsing.

/// One parsed path command.
#[derive(Debug, Clone)]
pub struct PathCmd {
    /// Command letter (M/L/H/V/C/A/Z/m/l/h/v/c/a/z).
    pub cmd: char,
    /// Numeric parameters (negative numbers supported).
    pub args: Vec<f32>,
}

/// Tokenize a path string into command segments.
/// Handles "M 13 2 L 3 14 h 9 l -1 8 ..." (space-separated) and also compact
/// forms where a command letter is glued to a number ("M13 2").
pub fn parse_path(d: &str) -> Vec<PathCmd> {
    let mut out = Vec::new();
    let chars: Vec<char> = d.chars().collect();
    let mut i = 0;
    let n = chars.len();
    let mut current_cmd: Option<char> = None;
    let mut args: Vec<f32> = Vec::new();

    while i < n {
        let c = chars[i];
        if c.is_ascii_alphabetic() {
            flush(&mut out, current_cmd.take(), &mut args);
            current_cmd = Some(c);
            i += 1;
        } else if c.is_ascii_digit() || c == '-' || c == '.' {
            // parse a number (with optional sign/decimal)
            let mut num = String::new();
            if c == '-' {
                num.push('-');
                i += 1;
            } else if c == '.' {
                num.push('0');
                num.push('.');
                i += 1;
            }
            while i < n && (chars[i].is_ascii_digit() || chars[i] == '.') {
                num.push(chars[i]);
                i += 1;
            }
            if let Ok(v) = num.parse::<f32>() {
                args.push(v);
            }
            // If this number followed an M and we already have a pair, and the
            // next char is a number (implicit L), keep collecting.
            if current_cmd.is_some() && matches!(current_cmd.unwrap(), 'M' | 'm') && args.len() >= 2 {
                // multiple coordinate pairs after M → implicit L
            }
        } else if c == ',' || c == ' ' || c == '\t' || c == '\n' {
            i += 1;
        } else {
            i += 1;
        }
    }
    flush(&mut out, current_cmd.take(), &mut args);

    // Expand implicit-L after M/m: a lone M with >2 args becomes M + L pairs.
    expand_multi_pair(&mut out);
    out
}

fn flush(out: &mut Vec<PathCmd>, cmd: Option<char>, args: &mut Vec<f32>) {
    if let Some(c) = cmd {
        if !args.is_empty() {
            out.push(PathCmd { cmd: c, args: std::mem::take(args) });
        } else {
            out.push(PathCmd { cmd: c, args: Vec::new() });
        }
    }
    args.clear();
}

/// After parsing, an `M x y x2 y2 x3 y3` sequence means M followed by implicit
/// L segments. Expand into explicit M + L commands so renderers are simple.
fn expand_multi_pair(out: &mut Vec<PathCmd>) {
    let mut i = 0;
    while i < out.len() {
        let cmd = out[i].cmd;
        let args = &out[i].args;
        if (cmd == 'M' || cmd == 'm') && args.len() > 2 {
            let first = args[0];
            let second = args[1];
            let mut rest: Vec<f32> = args[2..].to_vec();
            let mut j = i + 1;
            // insert L commands for the remaining pairs
            while rest.len() >= 2 {
                let lx = rest.remove(0);
                let ly = rest.remove(0);
                out.insert(j, PathCmd { cmd: 'L', args: vec![lx, ly] });
                j += 1;
            }
            out[i].args = vec![first, second];
        }
        i += 1;
    }
}
