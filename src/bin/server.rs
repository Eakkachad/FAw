//! katSVG HTTP Server (`katsvg-server`)
//!
//! Minimal std-only HTTP wrapper around the engine (P8). Exposes:
//!
//! - `GET /`                        → HTML form
//! - `GET /render?prompt=...&format=svg|pdf|png|pptx` → binary document with
//!   correct Content-Type (deterministic, no external deps).
//!
//! ```bash
//! cargo run --release --bin server -- 8787
//! curl "http://127.0.0.1:8787/render?prompt=Q3%20KPI%20dashboard&format=svg"
//! ```

use katsvg_engine::InfographicIntentRouter;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

const HTML_FORM: &str = r#"<!doctype html><html><head><meta charset="utf-8"><title>katSVG</title>
<style>body{font-family:system-ui;background:#0B0F19;color:#F9FAFB;display:grid;place-items:center;min-height:100vh}input{width:560px;padding:12px;border-radius:8px;border:1px solid #333;background:#111827;color:#F9FAFB;font-size:16px}button{padding:12px 24px;border:0;border-radius:8px;background:#3B82F6;color:#fff;font-weight:700;cursor:pointer}</style></head>
<body><form action="/render" method="get"><h1>katSVG Engine</h1>
<input name="prompt" placeholder='e.g. Q3 KPI dashboard: revenue: 124M, users: 12M in navy' autofocus/>
<button type="submit">Generate</button></form></body></html>"#;

fn content_type(fmt: &str) -> &'static str {
    match fmt {
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "application/octet-stream",
    }
}

fn handle(mut stream: TcpStream, router: &InfographicIntentRouter) {
    let mut buf = [0u8; 8192];
    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return,
    };
    let req = String::from_utf8_lossy(&buf[..n]).to_string();
    let mut lines = req.lines();

    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let target = parts.next().unwrap_or("/");

    let (status, body): (String, Vec<u8>) = match (method, target) {
        ("GET", "/") => (status_ok(), HTML_FORM.as_bytes().to_vec()),
        ("GET", t) if t.starts_with("/render") => {
            let query = t.split_once('?').map(|(_, q)| q).unwrap_or("");
            let params = parse_query(query);
            let prompt = params.get("prompt").cloned().unwrap_or_default();
            let format = params.get("format").cloned().unwrap_or_else(|| "svg".into());
            match router.parse_and_route(&prompt) {
                spec => match render_bytes(&spec, &format) {
                    Some(bytes) => (status_ok(), bytes),
                    None => (status_bad(), format!("bad format: {format}").into_bytes()),
                },
            }
        }
        _ => (status_not_found(), b"404".to_vec()),
    };

    let content_type = if target.starts_with("/render") {
        let format = parse_query(target.split_once('?').map(|(_, q)| q).unwrap_or(""))
            .get("format")
            .cloned()
            .unwrap_or_else(|| "svg".into());
        content_type(&format)
    } else {
        "text/html; charset=utf-8"
    };

    let header = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        content_type,
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(&body);
}

fn render_bytes(spec: &katsvg_engine::InfographicLayoutSpec, format: &str) -> Option<Vec<u8>> {
    match format {
        "svg" => Some(katsvg_engine::SVGVectorRenderer::render(spec).into_bytes()),
        "pdf" => Some(katsvg_engine::PDFVectorExporter::generate_pdf_bytes(spec)),
        "png" => Some(katsvg_engine::PNGRasterExporter::generate_png_bytes(spec)),
        "pptx" => Some(katsvg_engine::PPTXPresentationExporter::generate_pptx_bytes(spec)),
        _ => None,
    }
}

fn parse_query(q: &str) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    for pair in q.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            out.insert(percent_decode(k), percent_decode(v));
        }
    }
    out
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = &s[i + 1..i + 3];
            if let Ok(v) = u8::from_str_radix(hex, 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn status_ok() -> String { "200 OK".to_string() }
fn status_bad() -> String { "400 Bad Request".to_string() }
fn status_not_found() -> String { "404 Not Found".to_string() }

fn main() {
    let port: u16 = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(8787);

    let router = Arc::new(InfographicIntentRouter::new());
    let listener = TcpListener::bind(("127.0.0.1", port)).expect("bind failed");
    println!("katSVG server listening on http://127.0.0.1:{}", port);

    for stream in listener.incoming().flatten() {
        let router = Arc::clone(&router);
        std::thread::spawn(move || handle(stream, &router));
    }
}
