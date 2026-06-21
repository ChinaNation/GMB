import 'package:citizenapp/governance/admins-change/models/admin_account.dart';
import 'package:citizenapp/governance/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/qr/bodies/sign_request_body.dart';

class AdminSetChangeQrAdapter {
  AdminSetChangeQrAdapter._();

  static SignDisplay buildDisplay({
    required AdminAccountState account,
    required List<String> admins,
    required int newThreshold,
  }) {
    final normalizedAdmins = admins
        .map((admin) => '0x${AdminAccountIdCodec.normalizeHex(admin)}')
        .join(',');
    return SignDisplay(
      action: 'propose_admin_set_change',
      summary:
          '${account.kindLabel} 管理员更换：${account.admins.length} 人 -> ${admins.length} 人',
      fields: [
        SignDisplayField(key: 'org', label: '组织类型', value: account.orgLabel),
        SignDisplayField(
            key: 'account', label: '管理员账户', value: '0x${account.accountHex}'),
        SignDisplayField(
            key: 'admins', label: '新管理员', value: normalizedAdmins),
        SignDisplayField(
          key: 'new_threshold',
          label: '新阈值',
          value: '$newThreshold/${admins.length}',
        ),
      ],
    );
  }
}
