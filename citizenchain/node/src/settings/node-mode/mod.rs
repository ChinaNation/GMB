use crate::shared::security;
use serde::{Deserialize, Serialize};
use std::{fs, io::ErrorKind, path::PathBuf};
use tauri::AppHandle;

const NODE_MODE_FILE_NAME: &str = "node-mode.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeMode {
    Archive,
    Normal,
    Communication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeModeImplementationStatus {
    Active,
    Pending,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeModeOption {
    pub mode: NodeMode,
    pub label: &'static str,
    pub implementation_status: NodeModeImplementationStatus,
    pub enabled: bool,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeModeState {
    pub selected_mode: NodeMode,
    pub effective_mode: NodeMode,
    pub options: Vec<NodeModeOption>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredNodeMode {
    selected_mode: NodeMode,
}

impl NodeMode {
    fn from_wire_value(value: &str) -> Result<Self, String> {
        match value.trim() {
            "archive" => Ok(Self::Archive),
            "normal" => Ok(Self::Normal),
            "communication" => Ok(Self::Communication),
            other => Err(format!(
                "全节点模式无效：{other}。可选值为 archive、normal、communication"
            )),
        }
    }

    fn enabled(self) -> bool {
        matches!(self, Self::Archive)
    }

    fn ensure_enabled(self) -> Result<Self, String> {
        if self.enabled() {
            return Ok(self);
        }
        Err("该全节点模式尚未完成，当前不能选择".to_string())
    }
}

fn node_mode_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(security::app_data_dir(app)?.join(NODE_MODE_FILE_NAME))
}

fn load_selected_node_mode(app: &AppHandle) -> Result<NodeMode, String> {
    let path = node_mode_path(app)?;
    let raw = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(NodeMode::Archive),
        Err(err) => return Err(format!("read node mode failed: {err}")),
    };
    let stored: StoredNodeMode =
        serde_json::from_str(&raw).map_err(|err| format!("parse node mode failed: {err}"))?;
    Ok(stored.selected_mode)
}

fn save_selected_node_mode(app: &AppHandle, selected_mode: NodeMode) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(&StoredNodeMode { selected_mode })
        .map_err(|err| format!("encode node mode failed: {err}"))?;
    security::write_text_atomic(&node_mode_path(app)?, &format!("{raw}\n"))
        .map_err(|err| format!("write node mode failed: {err}"))
}

fn build_node_mode_state(selected_mode: NodeMode) -> NodeModeState {
    NodeModeState {
        selected_mode: if selected_mode.enabled() {
            selected_mode
        } else {
            NodeMode::Archive
        },
        // 中文注释：普通全节点、通信全节点的底层裁剪/收件箱能力尚未完成；
        // 当前版本只允许选择归档全节点，避免用户误以为待完成模式已经生效。
        effective_mode: NodeMode::Archive,
        options: node_mode_options(),
    }
}

fn node_mode_options() -> Vec<NodeModeOption> {
    vec![
        NodeModeOption {
            mode: NodeMode::Archive,
            label: "归档全节点",
            implementation_status: NodeModeImplementationStatus::Active,
            enabled: true,
            description: "保存完整链数据，当前版本实际按此模式运行。",
        },
        NodeModeOption {
            mode: NodeMode::Normal,
            label: "普通全节点",
            implementation_status: NodeModeImplementationStatus::Pending,
            enabled: false,
            description: "剪裁历史数据的全节点模式，功能后续完成。",
        },
        NodeModeOption {
            mode: NodeMode::Communication,
            label: "通信全节点",
            implementation_status: NodeModeImplementationStatus::Pending,
            enabled: false,
            description: "只保留通信收件箱所需能力，功能后续完成。",
        },
    ]
}

#[tauri::command]
pub fn get_node_mode(app: AppHandle) -> Result<NodeModeState, String> {
    let selected_mode = load_selected_node_mode(&app)?;
    Ok(build_node_mode_state(selected_mode))
}

#[tauri::command]
pub fn set_node_mode(app: AppHandle, mode: String) -> Result<NodeModeState, String> {
    if let Err(err) = security::append_audit_log(&app, "set_node_mode", "attempt") {
        eprintln!("[审计] set_node_mode attempt 日志写入失败: {err}");
    }

    let selected_mode = NodeMode::from_wire_value(&mode)?.ensure_enabled()?;
    save_selected_node_mode(&app, selected_mode)?;

    if let Err(err) = security::append_audit_log(&app, "set_node_mode", "success") {
        eprintln!("[审计] set_node_mode success 日志写入失败: {err}");
    }
    Ok(build_node_mode_state(selected_mode))
}

#[cfg(test)]
mod tests {
    use super::{build_node_mode_state, NodeMode, NodeModeImplementationStatus};

    #[test]
    fn pending_selected_mode_falls_back_to_archive() {
        let state = build_node_mode_state(NodeMode::Communication);

        assert_eq!(state.selected_mode, NodeMode::Archive);
        assert_eq!(state.effective_mode, NodeMode::Archive);
    }

    #[test]
    fn mode_options_expose_archive_active_and_other_modes_pending() {
        let state = build_node_mode_state(NodeMode::Archive);

        let archive = state
            .options
            .iter()
            .find(|option| option.mode == NodeMode::Archive)
            .expect("archive option exists");
        let normal = state
            .options
            .iter()
            .find(|option| option.mode == NodeMode::Normal)
            .expect("normal option exists");
        let communication = state
            .options
            .iter()
            .find(|option| option.mode == NodeMode::Communication)
            .expect("communication option exists");

        assert_eq!(
            archive.implementation_status,
            NodeModeImplementationStatus::Active
        );
        assert!(archive.enabled);
        assert_eq!(
            normal.implementation_status,
            NodeModeImplementationStatus::Pending
        );
        assert!(!normal.enabled);
        assert_eq!(
            communication.implementation_status,
            NodeModeImplementationStatus::Pending
        );
        assert!(!communication.enabled);
    }

    #[test]
    fn wire_value_rejects_unknown_node_mode() {
        let error = NodeMode::from_wire_value("light").expect_err("unknown mode fails");

        assert!(error.contains("全节点模式无效"));
    }

    #[test]
    fn pending_wire_value_is_not_selectable() {
        let error = NodeMode::from_wire_value("communication")
            .and_then(NodeMode::ensure_enabled)
            .expect_err("pending mode is disabled");

        assert!(error.contains("尚未完成"));
    }
}
