//! WASM entry points (`katSVG WASM`).
//!
//! F9: exports the engine as a wasm32-unknown-unknown module using plain C-ABI
//! `#[no_mangle]` functions — no wasm-bindgen required. A JS glue page can call
//! these to render an SVG in-browser.
//!
//! Memory model: caller calls `alloc_buf` once, writes the prompt bytes into the
//! returned pointer, calls `render_svg`, then reads `result_len` and copies the
//! SVG bytes from `result_ptr`.

use std::cell::UnsafeCell;
use std::os::raw::c_char;

const SCRATCH_CAP: usize = 2 << 20; // 2 MiB prompt+result scratch

/// Sync wrapper around `UnsafeCell` for wasm single-threaded use.
struct WasmStatic<T>(UnsafeCell<T>);
unsafe impl<T> Sync for WasmStatic<T> {}

const fn ws<T>(v: T) -> WasmStatic<T> {
    WasmStatic(UnsafeCell::new(v))
}

static SCRATCH: WasmStatic<Vec<u8>> = ws(Vec::new());
static RESULT_LEN: WasmStatic<usize> = ws(0);

fn scratch() -> &'static mut Vec<u8> {
    unsafe { &mut *SCRATCH.0.get() }
}

/// Allocate the scratch buffer. Call once before use.
#[unsafe(no_mangle)]
pub extern "C" fn alloc_buf() -> *mut c_char {
    unsafe {
        scratch().resize(SCRATCH_CAP, 0);
        unsafe {
            *RESULT_LEN.0.get() = 0;
        }
        scratch().as_mut_ptr() as *mut c_char
    }
}

/// Render a prompt (written into scratch at `ptr`, `len` bytes) to SVG.
/// The SVG bytes are placed at the start of the scratch buffer; `result_len`
/// reports the length. Returns 0 on success, negative on error.
#[unsafe(no_mangle)]
pub extern "C" fn render_svg(ptr: *mut c_char, len: usize) -> i32 {
    unsafe {
        let s = scratch();
        if s.is_empty() {
            return -1;
        }
        let prompt_bytes = std::slice::from_raw_parts(ptr as *const u8, len);
        let prompt = String::from_utf8_lossy(prompt_bytes);
        let router = crate::InfographicIntentRouter::new();
        let spec = router.parse_and_route(&prompt);
        let svg = crate::SVGVectorRenderer::render(&spec);
        let bytes = svg.as_bytes();
        if bytes.len() > s.len() {
            return -2;
        }
        s[..bytes.len()].copy_from_slice(bytes);
        unsafe {
            *RESULT_LEN.0.get() = bytes.len();
        }
        0
    }
}

/// Length of the last render result (in bytes).
#[unsafe(no_mangle)]
pub extern "C" fn result_len() -> usize {
    unsafe { *RESULT_LEN.0.get() }
}

/// Pointer to the start of the result (== scratch base after alloc_buf).
#[unsafe(no_mangle)]
pub extern "C" fn result_ptr() -> *mut c_char {
    unsafe { scratch().as_mut_ptr() as *mut c_char }
}
