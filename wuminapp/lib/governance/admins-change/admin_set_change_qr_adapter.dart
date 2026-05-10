import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/governance/admins-change/codec/subject_id_codec.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';

class AdminSetChangeQrAdapter {
  AdminSetChangeQrAdapter._();

  static SignDisplay buildDisplay({
    required AdminSubjectState subject,
    required List<String> newAdmins,
  }) {
    final normalizedAdmins = newAdmins
        .map((admin) => '0x${AdminSubjectIdCodec.normalizeHex(admin)}')
        .join(',');
    return SignDisplay(
      action: 'propose_admin_set_change',
      summary:
          '${subject.kindLabel} 管理员更换：${subject.admins.length} 人 -> ${newAdmins.length} 人',
      fields: [
        SignDisplayField(key: 'org', label: '组织类型', value: subject.orgLabel),
        SignDisplayField(
            key: 'subject', label: '管理员主体', value: '0x${subject.subjectIdHex}'),
        SignDisplayField(
            key: 'new_admins', label: '新管理员', value: normalizedAdmins),
      ],
    );
  }
}
