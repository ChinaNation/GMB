use std::path::PathBuf;
use std::process::Command;

fn main() {
    // CI 中先单独编译 node 并准备 sidecar，再用 SKIP_NODE_BUILD=1 cargo tauri build
    // 避免 tauri build 过程中重复编译 node。
    if std::env::var("SKIP_NODE_BUILD").is_err() {
        build_chain_node();
    }
    ensure_frontend_dist_dir();
    tauri_build::build();
}

fn build_chain_node() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let chain_dir = manifest_dir
        .parent() // nodeui/
        .and_then(|p| p.parent()) // citizenchain/
        .expect("cannot resolve citizenchain root");
    let binaries_dir = manifest_dir.join("binaries");
    let target_triple =
        std::env::var("TARGET").unwrap_or_else(|_| "aarch64-apple-darwin".to_string());

    let chain_manifest = chain_dir.join("Cargo.toml");
    let node_bin = chain_dir.join("target/release/node");
    let sidecar = binaries_dir.join("citizenchain-node");

    // 构建参数
    let mut args = vec![
        "build",
        "--release",
        "--manifest-path",
        chain_manifest.to_str().expect("invalid path"),
        "-p",
        "node",
    ];

    eprintln!("[build.rs] 编译链节点...");

    let status = Command::new("cargo")
        .args(&args)
        .status()
        .expect("failed to run cargo build for chain node");

    if !status.success() {
        panic!("chain node compilation failed");
    }

    // 复制二进制
    if node_bin.exists() {
        // 先确保 sidecar 目录存在，避免首次构建时 copy 直接因缺目录失败。
        std::fs::create_dir_all(&binaries_dir)
            .unwrap_or_else(|e| panic!("create sidecar dir failed: {e}"));

        std::fs::copy(&node_bin, &sidecar)
            .unwrap_or_else(|e| panic!("copy node binary failed: {e}"));

        // 按当前编译目标生成 sidecar 文件名，保持与打包脚本命名一致。
        let sidecar_arch = binaries_dir.join(format!("citizenchain-node-{target_triple}"));
        std::fs::copy(&node_bin, &sidecar_arch)
            .unwrap_or_else(|e| panic!("copy node binary (arch) failed: {e}"));

        // sha256
        if let Ok(sha_output) = Command::new("shasum")
            .args(["-a", "256"])
            .arg(&sidecar)
            .output()
        {
            let sha_line = String::from_utf8_lossy(&sha_output.stdout);
            let sha = sha_line.split_whitespace().next().unwrap_or("");
            let sha_file = binaries_dir.join("citizenchain-node.sha256");
            let _ = std::fs::write(&sha_file, sha);
        }

        eprintln!("[build.rs] 链节点已更新");
    }
}

fn ensure_frontend_dist_dir() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let frontend_dist = manifest_dir
        .parent() // nodeui/
        .expect("cannot resolve nodeui dir")
        .join("frontend/dist");

    // Tauri 在编译期会校验 frontendDist 是否存在，因此这里先补齐目录。
    std::fs::create_dir_all(&frontend_dist)
        .unwrap_or_else(|e| panic!("create frontend dist dir failed: {e}"));
}
