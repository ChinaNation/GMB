#[cfg(feature = "std")]
fn main() {
    // When WASM_FILE is set, skip wasm-builder and use the pre-built WASM binary directly.
    // This ensures all platforms use the same WASM, producing identical genesis hashes.
    if let Ok(wasm_file) = std::env::var("WASM_FILE") {
        let out_dir =
            std::env::var("OUT_DIR").expect("OUT_DIR not set");
        let dest = std::path::Path::new(&out_dir).join("wasm_binary.rs");

        // Normalize path for include_bytes! (works on all platforms)
        let wasm_path = std::path::Path::new(&wasm_file)
            .canonicalize()
            .unwrap_or_else(|e| panic!("WASM_FILE path not found: {wasm_file}: {e}"));
        let wasm_path_str = wasm_path.display();

        std::fs::write(
            &dest,
            format!(
                r#"pub const WASM_BINARY: Option<&[u8]> = Some(include_bytes!("{wasm_path_str}"));
pub const WASM_BINARY_BLOATY: Option<&[u8]> = Some(include_bytes!("{wasm_path_str}"));
"#,
            ),
        )
        .expect("Failed to write wasm_binary.rs");
    } else {
        substrate_wasm_builder::WasmBuilder::build_using_defaults();
    }
}

#[cfg(not(feature = "std"))]
fn main() {}
