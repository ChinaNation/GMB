mod home;
mod mining;
mod network;
mod other;
mod rpc;
mod settings;
mod validation;

use home::home_node::{
    cleanup_on_exit, get_chain_status, get_node_identity, get_node_status, set_node_name,
    start_node, stop_node, AppState, RuntimeState,
};
use mining::mining_dashboard::get_mining_dashboard;
use network::network_overview::get_network_overview;
use other::other_tabs::get_other_tabs_content;
use settings::bootnodes_address::{
    get_bootnode_key, get_genesis_bootnode_options, set_bootnode_key,
};
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
            get_mining_dashboard,
            get_network_overview,
            get_other_tabs_content
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                cleanup_on_exit(app);
            }
        });
}
