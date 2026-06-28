import 'dart:typed_data';

import 'package:citizenapp/citizen/proposal/admins-change/codec/admin_set_change_call_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_set_change_result.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_set_validation.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/signed_extrinsic_builder.dart';

class AdminSetChangeService {
  AdminSetChangeService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  Uint8List buildCallData({
    required AdminAccountState account,
    required String proposerPubkeyHex,
    required List<String> admins,
    required int newThreshold,
  }) {
    final normalized = AdminSetValidation.validate(
      account: account,
      proposerPubkeyHex: proposerPubkeyHex,
      admins: admins,
      newThreshold: newThreshold,
    );
    return AdminSetChangeCallCodec.build(
      institutionCode: account.institutionCode,
      adminKind: account.kind,
      accountId: AdminAccountIdCodec.fromHex(account.accountHex),
      admins: normalized.admins,
      newThreshold: normalized.threshold,
    );
  }

  Future<AdminSetChangeSubmitResult> submit({
    required AdminAccountState account,
    required List<String> admins,
    required int newThreshold,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final callData = buildCallData(
      account: account,
      proposerPubkeyHex: AdminAccountIdCodec.hexEncode(signerPubkey),
      admins: admins,
      newThreshold: newThreshold,
    );
    final result = await SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'AdminSetChange',
    ).signAndSubmit(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onWatchEvent: onWatchEvent,
    );
    return AdminSetChangeSubmitResult(
      txHash: result.txHash,
      usedNonce: result.usedNonce,
    );
  }
}
