import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';

class AdminSetChangeQrAdapter {
  AdminSetChangeQrAdapter._();

  static SignDisplay buildDisplay({
    required AdminSubjectState subject,
    required int newAdminCount,
  }) {
    return SignDisplay(
      action: 'propose_admin_set_change',
      summary:
          '${subject.kindLabel} 管理员更换：${subject.admins.length} 人 -> $newAdminCount 人',
      fields: [
        SignDisplayField(
            key: 'subject_id',
            label: '主体ID',
            value: '0x${subject.subjectIdHex}'),
        SignDisplayField(
            key: 'admin_count', label: '管理员人数', value: '$newAdminCount'),
        SignDisplayField(
            key: 'threshold', label: '内部阈值', value: '${subject.threshold}'),
      ],
    );
  }
}
