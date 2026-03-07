use serde::Serialize;

const WHITEPAPER_URL: &str = "https://chinanation.github.io/GMB/GMB_README.html";
const CONSTITUTION_URL: &str = "https://chinanation.github.io/GMB/FRC_README.html";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherTabItem {
    pub key: String,
    pub title: String,
    pub content_type: String,
    pub url: Option<String>,
    pub text: Option<String>,
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
                content_type: "iframe".to_string(),
                url: Some(WHITEPAPER_URL.to_string()),
                text: None,
            },
            OtherTabItem {
                key: "constitution".to_string(),
                title: "公民治理宪法".to_string(),
                content_type: "iframe".to_string(),
                url: Some(CONSTITUTION_URL.to_string()),
                text: None,
            },
            OtherTabItem {
                key: "party".to_string(),
                title: "公民党".to_string(),
                content_type: "text".to_string(),
                url: None,
                text: Some("公民党内容入口（待接入）。".to_string()),
            },
        ],
    })
}
