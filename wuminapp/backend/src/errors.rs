#![allow(dead_code)]

#[derive(Debug)]
pub struct ApiError {
    pub code: u32,
    pub message: &'static str,
}

impl ApiError {
    pub const fn new(code: u32, message: &'static str) -> Self {
        Self { code, message }
    }
}
