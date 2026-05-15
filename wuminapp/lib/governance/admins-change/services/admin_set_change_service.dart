import 'dart:typed_data';

import 'package:wuminapp_mobile/governance/admins-change/codec/admin_set_change_call_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/codec/subject_id_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/models/admin_set_change_result.dart';
import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/admin_set_validation.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/signed_extrinsic_builder.dart';

class AdminSetChangeService {
  AdminSetChangeService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  Uint8List buildCallData({
    required AdminSubjectState subject,
    required String proposerPubkeyHex,
    required List<String> newAdmins,
    required int newThreshold,
  }) {
    final normalized = AdminSetValidation.validate(
      subject: subject,
      proposerPubkeyHex: proposerPubkeyHex,
      newAdmins: newAdmins,
      newThreshold: newThreshold,
    );
    return AdminSetChangeCallCodec.build(
      org: subject.org,
      subjectId: AdminSubjectIdCodec.fromHex(subject.subjectIdHex),
      newAdmins: normalized.admins,
      newThreshold: normalized.threshold,
    );
  }

  Future<AdminSetChangeSubmitResult> submit({
    required AdminSubjectState subject,
    required List<String> newAdmins,
    required int newThreshold,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final callData = buildCallData(
      subject: subject,
      proposerPubkeyHex: AdminSubjectIdCodec.hexEncode(signerPubkey),
      newAdmins: newAdmins,
      newThreshold: newThreshold,
    );
    final result = await SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'AdminsChange',
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
