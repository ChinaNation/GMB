//! SFID 机构备案记录辅助逻辑。

use super::types::{InstitutionFilingPayload, InstitutionFilingRecord};

/// 中文注释:重复提交时先按三字段判断是否为同一备案内容。
/// 具体是否允许幂等重试,由后续 extrinsic 的 storage 状态机决定。
pub fn same_filing_payload<BlockNumber>(
    record: &InstitutionFilingRecord<BlockNumber>,
    payload: &InstitutionFilingPayload,
) -> bool {
    record.sfid_id == payload.sfid_id
        && record.institution_name == payload.institution_name
        && record.account_name == payload.account_name
}
