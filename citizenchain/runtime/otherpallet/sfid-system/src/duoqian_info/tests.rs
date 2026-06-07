use super::filing::same_filing_payload;
use super::types::{
    FilingAccountName, FilingInstitutionName, FilingSfidNumber, InstitutionFilingPayload,
    InstitutionFilingRecord,
};
use super::validate::{validate_payload, FilingValidationError};

fn bounded<T>(bytes: &[u8]) -> T
where
    T: TryFrom<Vec<u8>>,
{
    T::try_from(bytes.to_vec())
        .ok()
        .expect("test bytes fit bound")
}

#[test]
fn validate_payload_rejects_empty_field() {
    let payload = InstitutionFilingPayload {
        sfid_number: bounded::<FilingSfidNumber>(b""),
        institution_name: bounded::<FilingInstitutionName>("机构".as_bytes()),
        account_name: bounded::<FilingAccountName>("主账户".as_bytes()),
    };

    assert_eq!(
        validate_payload(&payload),
        Err(FilingValidationError::EmptySfidNumber)
    );
}

#[test]
fn same_filing_payload_matches_three_fields() {
    let payload = InstitutionFilingPayload {
        sfid_number: bounded::<FilingSfidNumber>(b"AH001-SCB0H-202605070-2026"),
        institution_name: bounded::<FilingInstitutionName>("测试股份有限公司".as_bytes()),
        account_name: bounded::<FilingAccountName>("主账户".as_bytes()),
    };
    let record = InstitutionFilingRecord {
        sfid_number: payload.sfid_number.clone(),
        institution_name: payload.institution_name.clone(),
        account_name: payload.account_name.clone(),
        filed_at_block: 12_u64,
    };

    assert!(same_filing_payload(&record, &payload));
}
