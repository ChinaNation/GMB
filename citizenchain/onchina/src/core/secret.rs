//! 敏感字符串封装。
//!
//! 这里只保存运行时必须短暂持有的密钥文本包装,不承载任何业务主数据。

use std::fmt;

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub(crate) struct SensitiveSeed(String);

impl SensitiveSeed {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// 只允许密码学代码读取原始密钥文本,不得写入日志或错误消息。
    #[must_use = "secret material should only be exposed to crypto code paths"]
    pub(crate) fn expose_secret(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for SensitiveSeed {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SensitiveSeed {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl Drop for SensitiveSeed {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl fmt::Debug for SensitiveSeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SensitiveSeed(***)")
    }
}
