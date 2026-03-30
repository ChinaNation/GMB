//! 桌面 GUI 模块（Tauri）。
//!
//! 将 Substrate 区块链节点和 Tauri 桌面界面合并为单一程序。
//! 启动时同时运行 Tauri 窗口和 Substrate 节点服务。

pub(crate) mod governance;
pub(crate) mod home;
pub(crate) mod mining;
pub(crate) mod network;
pub(crate) mod other;
pub(crate) mod settings;
pub(crate) mod shared;
pub(crate) mod node_runner;

use home::{cleanup_on_exit, cleanup_on_startup, AppState, RuntimeState};
use std::sync::Mutex;

/// 启动 Tauri 桌面应用。
///
/// Substrate 节点在用户点击"启动节点"时在进程内启动（不再作为子进程）。
pub fn run_desktop() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState(Mutex::new(RuntimeState::default())))
        .invoke_handler(tauri::generate_handler![
            home::identity::get_node_status,
            home::process::start_node,
            home::process::stop_node,
            settings::fee_address::get_reward_wallet,
            settings::fee_address::set_reward_wallet,
            settings::bootnodes_address::get_bootnode_key,
            settings::grandpa_address::get_grandpa_key,
            settings::bootnodes_address::set_bootnode_key,
            settings::grandpa_address::set_grandpa_key,
            settings::bootnodes_address::get_genesis_bootnode_options,
            home::identity::set_node_name,
            home::rpc::get_chain_status,
            home::identity::get_node_identity,
            home::rpc::get_total_issuance,
            home::rpc::get_total_stake,
            mining::mining_dashboard::get_mining_dashboard,
            network::network_overview::get_network_overview,
            other::other_tabs::get_other_tabs_content,
            governance::get_governance_overview,
            governance::get_institution_detail,
            governance::get_proposal_page,
            governance::get_proposal_detail,
            governance::get_next_proposal_id,
            governance::get_institution_proposals,
            settings::cold_wallets::get_cold_wallets,
            settings::cold_wallets::add_cold_wallet,
            settings::cold_wallets::remove_cold_wallet,
            governance::check_admin_wallets,
            governance::build_vote_request,
            governance::build_joint_vote_request,
            governance::build_propose_transfer_request,
            governance::submit_propose_transfer,
            governance::build_developer_upgrade_request,
            governance::submit_developer_upgrade,
            governance::build_propose_upgrade_request,
            governance::submit_propose_upgrade,
            governance::submit_vote,
            governance::check_vote_status
        ])
        .setup(|app| {
            cleanup_on_startup(app.handle());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("启动公民链失败")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                cleanup_on_exit(app);
            }
        });
}
