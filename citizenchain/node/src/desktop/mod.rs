//! 桌面 GUI 模块（Tauri）。
//!
//! 将 Substrate 区块链节点和 Tauri 桌面界面合并为单一程序。
//! 节点生命周期与 App 进程绑定：
//! - App 启动 → setup 后台线程自动 `start_node_blocking`
//! - 用户关窗（红 X / Cmd+Q / 菜单 Quit / 系统关闭）→ App 退出 → `RunEvent::Exit` 触发 `cleanup_on_exit`
//! - macOS 黄色横线为系统原生 minimize，不影响节点和进程，无需拦截
//! 三平台（macOS / Windows / Linux）行为统一：关窗即退出软件即停节点。
//! 桌面端各功能模块已扁平化到 crate 根层，例如 `crate::governance` 与 `crate::settings`。

pub(crate) mod node_runner;

use crate::{
    governance,
    home::{self, cleanup_on_exit, cleanup_on_startup, AppState, RuntimeState},
    mining, other, settings,
};
use std::sync::Mutex;

/// 启动 Tauri 桌面应用。
///
/// Substrate 节点在 setup 阶段后台线程自动启动；用户无启停按钮、无密码框。
pub fn run_desktop() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState(Mutex::new(RuntimeState::default())))
        .invoke_handler(tauri::generate_handler![
            home::identity::get_node_status,
            settings::fee_address::get_reward_wallet,
            settings::fee_address::set_reward_wallet,
            settings::fee_address::get_local_miner_address,
            settings::bootnodes_address::get_bootnode_key,
            settings::grandpa_address::get_grandpa_key,
            settings::bootnodes_address::set_bootnode_key,
            settings::grandpa_address::set_grandpa_key,
            settings::bootnodes_address::get_genesis_bootnode_options,
            home::rpc::get_chain_status,
            home::identity::get_node_identity,
            home::rpc::get_total_issuance,
            home::rpc::get_total_stake,
            mining::dashboard::get_mining_dashboard,
            mining::network_overview::get_network_overview,
            other::other_tabs::get_other_tabs_content,
            governance::get_governance_overview,
            governance::get_institution_detail,
            governance::balance_watch::start_governance_balance_watch,
            governance::balance_watch::stop_governance_balance_watch,
            governance::get_proposal_page,
            governance::get_proposal_detail,
            governance::get_next_proposal_id,
            governance::get_institution_proposals,
            governance::get_institution_proposal_page,
            governance::activation::build_activate_admin_request,
            governance::activation::verify_activate_admin,
            governance::activation::get_activated_admins,
            governance::activation::deactivate_admin,
            governance::activation::has_any_activated_admin,
            governance::build_vote_request,
            governance::build_joint_vote_request,
            governance::build_propose_transfer_request,
            governance::submit_propose_transfer,
            governance::build_developer_upgrade_request,
            governance::submit_developer_upgrade,
            governance::build_propose_upgrade_request,
            governance::submit_propose_upgrade,
            governance::submit_vote,
            governance::check_vote_status,
            governance::build_propose_sweep_request,
            governance::submit_propose_sweep,
            governance::build_propose_safety_fund_request,
            governance::submit_propose_safety_fund,
            // Phase 3: safety_fund/sweep 投票统一走 governance::build_vote_request。
            home::transaction::get_wallets,
            home::transaction::add_wallet,
            home::transaction::remove_wallet,
            home::transaction::set_active_wallet,
            home::transaction::get_wallet_balance,
            home::transaction::build_transfer_request,
            home::transaction::submit_transfer,
            home::transaction::submit_miner_transfer,
            // ─── 清算行 offchain tab(ADR-007 Step 2 阶段 B) ───
            crate::offchain::duoqian_manage::commands::search_eligible_clearing_banks,
            crate::offchain::offchain_transaction::commands::query_clearing_bank_node_info,
            crate::offchain::offchain_transaction::commands::query_local_peer_id,
            crate::offchain::offchain_transaction::commands::test_clearing_bank_endpoint_connectivity,
            crate::offchain::offchain_transaction::commands::build_register_clearing_bank_request,
            crate::offchain::offchain_transaction::commands::submit_register_clearing_bank,
            crate::offchain::offchain_transaction::commands::build_update_clearing_bank_endpoint_request,
            crate::offchain::offchain_transaction::commands::submit_update_clearing_bank_endpoint,
            crate::offchain::offchain_transaction::commands::build_unregister_clearing_bank_request,
            crate::offchain::offchain_transaction::commands::submit_unregister_clearing_bank,
            crate::offchain::settlement::commands::build_decrypt_admin_request,
            crate::offchain::settlement::commands::verify_and_decrypt_admin,
            crate::offchain::settlement::commands::list_decrypted_admins,
            crate::offchain::settlement::commands::lock_decrypted_admin,
            crate::offchain::duoqian_manage::commands::fetch_clearing_bank_institution_detail,
            crate::offchain::duoqian_manage::commands::fetch_clearing_bank_institution_proposals,
            crate::offchain::duoqian_manage::commands::fetch_clearing_bank_institution_registration_info,
            crate::offchain::duoqian_manage::commands::build_propose_create_institution_request,
            crate::offchain::duoqian_manage::commands::submit_propose_create_institution
        ])
        .setup(|app| {
            cleanup_on_startup(app.handle());

            // 自动启动节点。在后台线程跑，避免阻塞 setup 让窗口慢出现。
            // start_node_blocking 内部带 5s + 2s 等待，前端通过 get_node_status 轮询自然刷新。
            let app_handle = app.handle().clone();
            std::thread::Builder::new()
                .name("auto-start-node".into())
                .spawn(move || {
                    if let Err(e) = home::start_node_blocking(app_handle) {
                        eprintln!("[节点] 自动启动失败: {e}");
                    }
                })
                .expect("spawn auto-start-node thread failed");

            Ok(())
        })
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("启动公民链失败")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                cleanup_on_exit(app);
            }
        });
}
