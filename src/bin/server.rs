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
<button type="submit">Generate</button></form>
<p><a href="/demo" style="color:#3B82F6">Open interactive demo →</a></p></body></html>"#;

/// F7: single-page interactive demo — live SVG preview + 4 format downloads.
const DEMO_PAGE: &str = r##"<!doctype html><html><head><meta charset="utf-8"><title>katSVG live demo</title>
<style>
body{font-family:system-ui;background:#0B0F19;color:#eee;margin:0;padding:24px}
.wrap{max-width:900px;margin:0 auto}
textarea{width:100%;padding:12px;border-radius:8px;border:1px solid #333;background:#111827;color:#eee;font-size:16px;font-family:inherit;resize:vertical;min-height:60px}
.btns{display:flex;gap:8px;margin:12px 0;flex-wrap:wrap}
.btns a{color:#fff;text-decoration:none;padding:10px 18px;border-radius:8px;font-weight:700;background:#3B82F6}
.btns a.pdf{background:#EF4444}.btns a.png{background:#10B981}.btns a.pptx{background:#F59E0B}
#preview{margin-top:16px;border:1px solid #2a2a2a;border-radius:12px;background:#fff;min-height:300px;display:flex;align-items:center;justify-content:center;overflow:auto}
#preview svg{max-width:100%;height:auto}
.meta{color:#9CA3AF;font-size:12px;margin-top:8px}
</style></head>
<body><div class="wrap">
<h1>katSVG — interactive demo</h1>
<p style="color:#9CA3AF">Type a prompt — the preview updates as you type. Download any format when ready.</p>
<textarea id="p" autofocus placeholder="e.g. Q3 KPI dashboard: revenue: 124M, users: 12M in navy">Q3 KPI dashboard: revenue: 124M, users: 12M, margin: 28% in navy</textarea>
<div class="btns">
  <a id="dl-svg" href="#" download="infographic.svg">SVG</a>
  <a id="dl-pdf" class="pdf" href="#" download="infographic.pdf">PDF</a>
  <a id="dl-png" class="png" href="#" download="infographic.png">PNG</a>
  <a id="dl-pptx" class="pptx" href="#" download="infographic.pptx">PPTX</a>
</div>
<div id="preview"><p style="color:#666">loading…</p></div>
<div class="meta" id="meta"></div>
</div>
<script>
const inp=document.getElementById('p'),prev=document.getElementById('preview'),meta=document.getElementById('meta');
let t;
function enc(s){return encodeURIComponent(s)}
function refresh(){
  const q=inp.value.trim(); if(!q){prev.innerHTML='<p style="color:#666">type a prompt…</p>';return;}
  prev.innerHTML='<p style="color:#666">rendering…</p>';
  fetch('/render?prompt='+enc(q)+'&format=all').then(r=>r.json()).then(d=>{
    const t0=performance.now();
    prev.innerHTML=d.svg_preview;
    const el=prev.querySelector('svg'); if(el){el.style.maxWidth='100%';el.style.height='auto';}
    meta.textContent='SVG '+Math.round(d.svg_b64.length*0.75)+'B · PDF '+Math.round(d.pdf_b64.length*0.75)+'B · PNG '+Math.round(d.png_b64.length*0.75)+'B · PPTX '+Math.round(d.pptx_b64.length*0.75)+'B · '+(performance.now()-t0).toFixed(0)+'ms';
    document.getElementById('dl-svg').href='data:image/svg+xml;base64,'+d.svg_b64;
    document.getElementById('dl-pdf').href='data:application/pdf;base64,'+d.pdf_b64;
    document.getElementById('dl-png').href='data:image/png;base64,'+d.png_b64;
    document.getElementById('dl-pptx').href='data:application/vnd.openxmlformats-officedocument.presentationml.presentation;base64,'+d.pptx_b64;
  }).catch(e=>{prev.innerHTML='<p style="color:#f87171">error: '+e+'</p>';});
}
inp.addEventListener('input',()=>{clearTimeout(t);t=setTimeout(refresh,250);});
refresh();
</script></body></html>"##;

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
        ("GET", "/demo") => (status_ok(), DEMO_PAGE.as_bytes().to_vec()),
        ("GET", t) if t.starts_with("/render") => {
            let query = t.split_once('?').map(|(_, q)| q).unwrap_or("");
            let params = parse_query(query);
            let prompt = params.get("prompt").cloned().unwrap_or_default();
            let format = params
                .get("format")
                .cloned()
                .unwrap_or_else(|| "svg".into());
            let spec = router.parse_and_route(&prompt);
            match format.as_str() {
                "all" => {
                    // JSON envelope: base64 of all 4 formats + SVG preview
                    let all = all_formats_json(&spec);
                    (status_ok(), all.into_bytes())
                }
                "preview" => {
                    let page = preview_page(&spec);
                    (status_ok(), page.into_bytes())
                }
                f => match render_bytes(&spec, f) {
                    Some(bytes) => (status_ok(), bytes),
                    None => (status_bad(), format!("bad format: {f}").into_bytes()),
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
        match format.as_str() {
            "all" => "application/json",
            "preview" => "text/html; charset=utf-8",
            f => content_type(f),
        }
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

fn b64(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(n >> 6) as usize & 63] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[n as usize & 63] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// JSON envelope with all 4 formats base64-encoded + SVG for inline preview.
fn all_formats_json(spec: &katsvg_engine::InfographicLayoutSpec) -> String {
    let svg = katsvg_engine::SVGVectorRenderer::render(spec);
    let pdf = katsvg_engine::PDFVectorExporter::generate_pdf_bytes(spec);
    let png = katsvg_engine::PNGRasterExporter::generate_png_bytes(spec);
    let pptx = katsvg_engine::PPTXPresentationExporter::generate_pptx_bytes(spec);
    serde_json::json!({
        "spec": spec,
        "svg_b64": b64(svg.as_bytes()),
        "pdf_b64": b64(&pdf),
        "png_b64": b64(&png),
        "pptx_b64": b64(&pptx),
        "svg_preview": svg,
    })
    .to_string()
}

/// HTML preview page embedding the SVG inline + download links.
fn preview_page(spec: &katsvg_engine::InfographicLayoutSpec) -> String {
    let svg = katsvg_engine::SVGVectorRenderer::render(spec);
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>katSVG preview</title></head>\
         <body style=\"font-family:system-ui;background:#0B0F19;color:#eee;padding:24px\">\
         <h2>katSVG — {}</h2>\
         <div>{}</div>\
         <p><a href=\"/render?prompt=.\">back</a></p></body></html>",
        spec.title, svg
    )
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

fn status_ok() -> String {
    "200 OK".to_string()
}
fn status_bad() -> String {
    "400 Bad Request".to_string()
}
fn status_not_found() -> String {
    "404 Not Found".to_string()
}

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
