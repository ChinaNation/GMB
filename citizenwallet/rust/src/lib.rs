//! CitizenWallet 冷钱包原生签名库。
//!
//! 本 crate **只有一行实质代码**：调用共享 crate `citizen-signer` 的导出宏生成
//! 4 个 `#[no_mangle] extern "C"` 入口。sr25519 的派生、签名、验签、私钥擦除、
//! panic 兜底全部在 `citizenchain/crates/citizen-signer` 里，与 CitizenApp 热端
//! **共用同一份源码**——冷热两端派生口径一旦分叉，同一助记词会算出不同账户，
//! 因此这里**绝不允许另写任何密码学实现**。
//!
//! 冷钱包永久离线、只需签名，所以产物是独立的小库（几百 KB），不像热端那样
//! 把签名接口挂进 58MB 的轻节点库里。
//!
//! 符号检查（注意平台差异，用错标志会误判为 0）：
//! - Android/ELF   `llvm-nm -D libcitizenwallet_signer.so    | grep -c citizen_sr25519` 应为 4
//! - iOS/Mach-O    `llvm-nm -g libcitizenwallet_signer.dylib | grep -c citizen_sr25519` 应为 4

citizen_signer::export_citizen_signer_ffi!();
