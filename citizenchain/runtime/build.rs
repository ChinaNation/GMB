#[cfg(feature = "std")]
fn main() {
    // 强制每次编译都重新运行 build.rs，确保 WASM 是最新的。
    println!("cargo:rerun-if-env-changed=WASM_FILE");
    println!("cargo:rerun-if-env-changed=WASM_BUILD_FROM_SOURCE");

    if let Ok(wasm_file) = std::env::var("WASM_FILE") {
        // ── 使用 CI 预编译的 WASM（三端 CI、本地脚本）──
        let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
        let dest = std::path::Path::new(&out_dir).join("wasm_binary.rs");

        let wasm_path = std::path::Path::new(&wasm_file)
            .canonicalize()
            .unwrap_or_else(|e| panic!("WASM_FILE 路径无效: {wasm_file}: {e}"));
        let wasm_path_str = wasm_path.display().to_string().replace('\\', "/");

        std::fs::write(
            &dest,
            format!(
                r#"pub const WASM_BINARY: Option<&[u8]> = Some(include_bytes!("{wasm_path_str}"));
pub const WASM_BINARY_BLOATY: Option<&[u8]> = Some(include_bytes!("{wasm_path_str}"));
"#,
            ),
        )
        .expect("写入 wasm_binary.rs 失败");

        eprintln!("使用 CI WASM: {wasm_path_str}");
    } else if std::env::var("WASM_BUILD_FROM_SOURCE").is_ok() {
        // ── WASM CI 专用：从源码编译 WASM（仅 WASM CI workflow 使用）──
        substrate_wasm_builder::WasmBuilder::build_using_defaults();
    } else {
        // ── 既没有 WASM_FILE 也没有 WASM_BUILD_FROM_SOURCE：拒绝编译 ──
        panic!(
            "\n\n错误：WASM_FILE 环境变量未设置。\n\
             所有节点必须使用 CI 编译的统一 WASM，不允许本地编译。\n\
             请通过以下方式启动：\n\
             - 本地开发：cd ~/GMB && ./citizenchain/scripts/run.sh\n\
             - 全新创世：cd ~/GMB && ./citizenchain/scripts/clean-run.sh\n\n"
        );
    }
}

#[cfg(not(feature = "std"))]
fn main() {}
