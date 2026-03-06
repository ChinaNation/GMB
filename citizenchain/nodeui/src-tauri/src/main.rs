mod node;
mod validation;

use node::{
    get_bootnode_key, get_chain_status, get_genesis_bootnode_options, get_grandpa_key,
    get_mining_dashboard, get_network_overview, get_node_identity, get_node_status,
    get_reward_wallet, set_bootnode_key, set_grandpa_key, set_node_name, set_reward_wallet,
    start_node, stop_node, AppState, RuntimeState,
};
use std::sync::Mutex;
use tauri::Manager;

fn main() {
    // main.rs 仅负责应用初始化和命令注册，业务逻辑下沉到 node.rs。
    tauri::Builder::default()
        .manage(AppState(Mutex::new(RuntimeState {
            local_node: None,
            node_key_file: None,
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
            get_network_overview
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Ok(mut state) = app.state::<AppState>().0.lock() {
                    if let Some(mut child) = state.local_node.take() {
                        #[cfg(unix)]
                        unsafe {
                            let pid = child.id() as i32;
                            if pid > 0 {
                                let _ = libc::kill(-pid, libc::SIGTERM);
                            }
                        }
                        let _ = child.kill();
                        let _ = child.try_wait();
                    }
                    if let Some(path) = state.node_key_file.take() {
                        let _ = std::fs::remove_file(path);
                    }
                }
            }
        });
}
