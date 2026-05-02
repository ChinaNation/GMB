#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[clap(flatten)]
    pub run: sc_cli::RunCmd,

    /// PoW 挖矿线程数。默认使用 CPU 可用并行度。设为 0 禁用挖矿。
    #[arg(long, value_name = "COUNT")]
    pub mining_threads: Option<usize>,

    /// GPU 挖矿设备编号（默认 0）。需编译时启用 gpu-mining feature。
    #[arg(long, value_name = "INDEX")]
    pub gpu_device: Option<usize>,

    /// 强制禁用 GPU 挖矿，即使编译了 gpu-mining feature。
    #[arg(long)]
    pub no_gpu: bool,

    /// 扫码支付 Step 2b-ii-β-2-b 新增:把本节点以清算行角色启动,
    /// 参数为清算行**主账户 SS58 地址**。若不设则节点不启动清算行组件
    /// (RPC / ledger / settlement/listener 全部跳过)。
    #[arg(long, value_name = "BANK_MAIN_SS58")]
    pub clearing_bank: Option<String>,

    /// 扫码支付 Step 2b-ii-β-2-b 新增:解锁 `offchain::settlement::keystore` 里清算行
    /// 管理员 sr25519 私钥的密码。不提供时签名密钥保持 `None`,节点只保留
    /// 查询 RPC;扫码提交需要生成 L2 ACK 签名,packer 上链也需要批次签名,
    /// 二者都会 fail-fast,直到密码提供并重启。
    #[arg(long, value_name = "PASSWORD")]
    pub clearing_bank_password: Option<String>,

    /// 扫码支付 Step 2b-iii-b 新增:`offchain::settlement::reserve` 主账对账触发周期(秒)。
    /// 缺省 300(5 分钟)。设为 0 则关闭对账 worker(不推荐,仅用于排障)。
    /// 仅在 `--clearing-bank` 生效时启用。
    #[arg(long, value_name = "SECS")]
    pub clearing_reserve_monitor_interval_secs: Option<u64>,
}

#[derive(Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommand {
    /// Key management cli utilities
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// Build a chain specification.
    /// DEPRECATED: `build-spec` command will be removed after 1/04/2026. Use `export-chain-spec`
    /// command instead.
    #[deprecated(
        note = "build-spec command will be removed after 1/04/2026. Use export-chain-spec command instead"
    )]
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Export the chain specification.
    ExportChainSpec(sc_cli::ExportChainSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// Sub-commands concerned with benchmarking.
    #[command(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Db meta columns information.
    ChainInfo(sc_cli::ChainInfoCmd),
}
