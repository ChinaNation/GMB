use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "contentType")]
pub enum TabContent {
    Document,
    Text { text: String },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherTabItem {
    pub key: String,
    pub title: String,
    #[serde(flatten)]
    pub content: TabContent,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherTabsPayload {
    pub tabs: Vec<OtherTabItem>,
}

#[tauri::command]
pub fn get_other_tabs_content() -> Result<OtherTabsPayload, String> {
    Ok(OtherTabsPayload {
        tabs: vec![
            OtherTabItem {
                key: "whitepaper".to_string(),
                title: "白皮书".to_string(),
                content: TabContent::Document,
            },
            OtherTabItem {
                key: "constitution".to_string(),
                title: "公民治理宪法".to_string(),
                content: TabContent::Document,
            },
            OtherTabItem {
                key: "party".to_string(),
                title: "公民党".to_string(),
                content: TabContent::Text {
                    text: "更多功能开发中。".to_string(),
                },
            },
        ],
    })
}
