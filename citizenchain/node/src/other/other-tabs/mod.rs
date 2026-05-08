use serde::Serialize;

// 中文注释：白皮书和公民宪法随桌面前端静态资源一起打包，避免仓库私有化后
// GitHub Pages 外链返回 404 导致桌面 tab 不可用。
const WHITEPAPER_URL: &str = "/GMB_README.html";
const CONSTITUTION_URL: &str = "/FRC_README.html";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "contentType")]
pub enum TabContent {
    Iframe { url: String },
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
                content: TabContent::Iframe {
                    url: WHITEPAPER_URL.to_string(),
                },
            },
            OtherTabItem {
                key: "constitution".to_string(),
                title: "公民治理宪法".to_string(),
                content: TabContent::Iframe {
                    url: CONSTITUTION_URL.to_string(),
                },
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
