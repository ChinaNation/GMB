//! SFID 机构备案 payload 基础校验。

use super::types::InstitutionFilingPayload;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilingValidationError {
    EmptySfidId,
    EmptyInstitutionName,
    EmptyAccountName,
}

/// 中文注释:第 1 步备案只允许三项机构信息进入链上 payload。
/// 这里先做与 storage 无关的最小非空校验;省级签名、防重放和重复备案校验
/// 后续在 extrinsic 接入时继续放在本目录内扩展。
pub fn validate_payload(payload: &InstitutionFilingPayload) -> Result<(), FilingValidationError> {
    if payload.sfid_id.is_empty() {
        return Err(FilingValidationError::EmptySfidId);
    }
    if payload.institution_name.is_empty() {
        return Err(FilingValidationError::EmptyInstitutionName);
    }
    if payload.account_name.is_empty() {
        return Err(FilingValidationError::EmptyAccountName);
    }
    Ok(())
}
