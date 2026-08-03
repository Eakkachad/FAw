//! WASM binary entry — produces a real `.wasm` that exports the engine.
//!
//! ```bash
//! cargo build --release --target wasm32-unknown-unknown --bin katsvg-wasm
//! ```
//! The resulting `.wasm` exports `alloc_buf`, `render_svg`, `result_len`,
//! `result_ptr` for JS glue (see `src/wasm.rs`).

fn main() {
    // Call the library export so the linker retains the `#[no_mangle]` fns.
    #[cfg(target_arch = "wasm32")]
    unsafe {
        katsvg_engine::wasm::alloc_buf();
    }
}
