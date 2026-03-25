mod governance;
mod home;
mod mining;
mod network;
mod other;
mod settings;
mod shared;

use home::{
    cleanup_on_exit, cleanup_on_startup, get_chain_status, get_node_identity, get_node_status,
    get_total_issuance, get_total_stake, set_node_name, start_node, stop_node, AppState,
    RuntimeState,
};
use governance::{
    build_joint_vote_request, build_propose_transfer_request, build_vote_request,
    check_admin_wallets, check_vote_status, get_governance_overview, get_institution_detail,
    get_institution_proposals, get_next_proposal_id, get_proposal_detail, get_proposal_page,
    submit_propose_transfer, submit_vote,
};
use mining::mining_dashboard::get_mining_dashboard;
use network::network_overview::get_network_overview;
use other::other_tabs::get_other_tabs_content;
use settings::bootnodes_address::{
    get_bootnode_key, get_genesis_bootnode_options, set_bootnode_key,
};
use settings::cold_wallets::{add_cold_wallet, get_cold_wallets, remove_cold_wallet};
use settings::fee_address::{get_reward_wallet, set_reward_wallet};
use settings::grandpa_address::{get_grandpa_key, set_grandpa_key};
use std::sync::Mutex;

fn main() {
    // main.rs 仅负责应用初始化和命令注册，业务逻辑下沉到 home/mining/network/other/settings 模块。
    tauri::Builder::default()
        .manage(AppState(Mutex::new(RuntimeState {
            local_node: None,
            node_key_file: None,
            node_bin_file: None,
        })))
        .invoke_handler(tauri::generate_handler![
            get_node_status,
            start_node,
            stop_node,
            get_reward_wallet,
            set_reward_wallet,
            get_bootnode_key,
            get_grandpa_key,
            set_bootnode_key,
            set_grandpa_key,
            get_genesis_bootnode_options,
            set_node_name,
            get_chain_status,
            get_node_identity,
            get_total_issuance,
            get_total_stake,
            get_mining_dashboard,
            get_network_overview,
            get_other_tabs_content,
            get_governance_overview,
            get_institution_detail,
            get_proposal_page,
            get_proposal_detail,
            get_next_proposal_id,
            get_institution_proposals,
            get_cold_wallets,
            add_cold_wallet,
            remove_cold_wallet,
            check_admin_wallets,
            build_vote_request,
            build_joint_vote_request,
            build_propose_transfer_request,
            submit_propose_transfer,
            submit_vote,
            check_vote_status
        ])
        .setup(|app| {
            cleanup_on_startup(app.handle());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                cleanup_on_exit(app);
            }
        });
}
