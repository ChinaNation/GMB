#![cfg(test)]

use super::*;

#[test]
fn default_asset_allows_all_actions() {
    let account = [7u8; 32];
    assert!(<() as InstitutionAsset<[u8; 32]>>::can_spend(
        &account,
        InstitutionAssetAction::DuoqianTransferExecute,
    ));
    assert!(<() as InstitutionAsset<[u8; 32]>>::can_spend(
        &account,
        InstitutionAssetAction::DuoqianCloseExecute,
    ));
    assert!(<() as InstitutionAsset<[u8; 32]>>::can_spend(
        &account,
        InstitutionAssetAction::OffchainBatchDebit,
    ));
    assert!(<() as InstitutionAsset<[u8; 32]>>::can_spend(
        &account,
        InstitutionAssetAction::OffchainFeeSweepExecute,
    ));
    assert!(<() as InstitutionAsset<[u8; 32]>>::can_spend(
        &account,
        InstitutionAssetAction::NrcSafetyFundTransfer,
    ));
    //  4 个动作
    assert!(<() as InstitutionAsset<[u8; 32]>>::can_spend(
        &account,
        InstitutionAssetAction::L3DepositIn,
    ));
    assert!(<() as InstitutionAsset<[u8; 32]>>::can_spend(
        &account,
        InstitutionAssetAction::L3WithdrawOut,
    ));
    assert!(<() as InstitutionAsset<[u8; 32]>>::can_spend(
        &account,
        InstitutionAssetAction::L2ClearingDebit,
    ));
    assert!(<() as InstitutionAsset<[u8; 32]>>::can_spend(
        &account,
        InstitutionAssetAction::L2FeeCollect,
    ));
}
