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
    im, mining, other, settings,
    transaction::multisig_transfer,
};
use std::sync::Mutex;

/// 启动 Tauri 桌面应用。
///
/// Substrate 节点在 setup 阶段后台线程自动启动；用户无启停按钮、无密码框。
pub fn run_desktop() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState(Mutex::new(RuntimeState::default())))
        .invoke_handler(tauri::generate_handler![
            home::identity::get_node_status,
            home::sync_guard::get_sync_guard_status,
            settings::desktop_update::prepare_desktop_update,
            settings::node_mode::get_node_mode,
            settings::node_mode::set_node_mode,
            settings::communication_node::get_communication_node,
            settings::communication_node::set_communication_node_enabled,
            im::commands::get_im_private_node_policy,
            im::commands::get_im_direct_network_capability,
            im::commands::validate_im_node_endpoint,
            im::commands::validate_im_direct_delivery_request,
            im::commands::submit_im_direct_encrypted_envelope,
            im::commands::fetch_im_direct_key_packages,
            im::commands::consume_im_direct_key_package,
            im::commands::register_im_owner_device,
            im::commands::submit_im_encrypted_envelope,
            im::commands::fetch_im_pending_envelopes,
            im::commands::ack_im_envelope,
            im::commands::publish_im_key_package,
            im::commands::fetch_im_key_packages,
            im::commands::consume_im_key_package,
            settings::fee_account::get_reward_wallet,
            settings::fee_account::set_reward_wallet,
            settings::fee_account::get_local_miner_address,
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
            other::other_tabs::get_runtime_constitution_document,
            governance::get_governance_overview,
            governance::get_institution_detail,
            governance::balance_watch::start_governance_balance_watch,
            governance::balance_watch::stop_governance_balance_watch,
            governance::get_proposal_page,
            governance::get_proposal_detail,
            governance::get_next_proposal_id,
            governance::get_institution_proposals,
            governance::get_institution_proposal_page,
            // 双层 ID + 反向索引(spec_version v1)
            governance::get_proposal_display,
            governance::list_proposals_by_org,
            governance::list_proposals_by_institution,
            governance::list_proposals_by_owner,
            governance::admins_change::activation::build_activate_admin_request,
            governance::admins_change::activation::verify_activate_admin,
            governance::admins_change::activation::get_activated_admins,
            governance::admins_change::activation::deactivate_admin,
            governance::admins_change::activation::has_any_activated_admin,
            governance::admins_change::commands::get_admin_account_state,
            governance::admins_change::commands::build_admin_set_change_request,
            governance::admins_change::commands::submit_admin_set_change,
            governance::build_vote_request,
            governance::build_joint_vote_request,
            multisig_transfer::commands::build_multisig_transfer_request,
            multisig_transfer::commands::submit_multisig_transfer,
            multisig_transfer::commands::build_multisig_safety_fund_request,
            multisig_transfer::commands::submit_multisig_safety_fund,
            multisig_transfer::commands::build_multisig_sweep_request,
            multisig_transfer::commands::submit_multisig_sweep,
            governance::runtime_upgrade::commands::build_developer_upgrade_request,
            governance::runtime_upgrade::commands::submit_developer_upgrade,
            governance::runtime_upgrade::commands::build_propose_upgrade_request,
            governance::runtime_upgrade::commands::submit_propose_upgrade,
            governance::submit_vote,
            governance::check_vote_status,
            crate::transaction::onchain_transaction::get_wallets,
            crate::transaction::onchain_transaction::add_wallet,
            crate::transaction::onchain_transaction::remove_wallet,
            crate::transaction::onchain_transaction::set_active_wallet,
            crate::transaction::onchain_transaction::get_wallet_balance,
            crate::transaction::onchain_transaction::build_transfer_request,
            crate::transaction::onchain_transaction::submit_transfer,
            crate::transaction::onchain_transaction::submit_miner_transfer,
            // ─── 清算行 offchain tab ───
            governance::organization_manage::commands::search_eligible_clearing_banks,
            crate::transaction::offchain_transaction::commands::query_clearing_bank_node_info,
            crate::transaction::offchain_transaction::commands::query_local_peer_id,
            crate::transaction::offchain_transaction::commands::test_clearing_bank_endpoint_connectivity,
            crate::transaction::offchain_transaction::commands::build_register_clearing_bank_request,
            crate::transaction::offchain_transaction::commands::submit_register_clearing_bank,
            crate::transaction::offchain_transaction::commands::build_update_clearing_bank_endpoint_request,
            crate::transaction::offchain_transaction::commands::submit_update_clearing_bank_endpoint,
            crate::transaction::offchain_transaction::commands::build_unregister_clearing_bank_request,
            crate::transaction::offchain_transaction::commands::submit_unregister_clearing_bank,
            crate::transaction::offchain_transaction::settlement::commands::build_decrypt_admin_request,
            crate::transaction::offchain_transaction::settlement::commands::verify_and_decrypt_admin,
            crate::transaction::offchain_transaction::settlement::commands::list_decrypted_admins,
            crate::transaction::offchain_transaction::settlement::commands::lock_decrypted_admin,
            governance::organization_manage::commands::fetch_clearing_bank_institution_detail,
            governance::organization_manage::commands::fetch_clearing_bank_institution_proposals,
            governance::organization_manage::commands::fetch_clearing_bank_institution_registration_info,
            governance::organization_manage::commands::build_propose_create_institution_request,
            governance::organization_manage::commands::submit_propose_create_institution
        ])
        .setup(|app| {
            cleanup_on_startup(app.handle());
            // 中文注释：同步守护只读本机 RPC，等待节点启动后再按本机状态自检。
            home::sync_guard::start_sync_guard(app.handle().clone());

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
