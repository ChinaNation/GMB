import 'package:flutter_test/flutter_test.dart';
import 'package:citizenwallet/signer/action_labels.dart';
import 'package:citizenwallet/signer/field_labels.dart';

void main() {
  group('action labels', () {
    test('已登记 action code 必须有中文动作名', () {
      for (final entry in actionKeyByCode.entries) {
        expect(
          actionLabelForDecodedAction(entry.value),
          isNotNull,
          reason: '0x${entry.key.toRadixString(16)} 缺少中文动作名',
        );
      }
      expect(actionLabelForQrAction(9), '广场账户动作签名');
      expect(actionLabelForQrAction(0x7fff), isNull);
    });
  });

  group('fieldLabelText', () {
    test('公民身份确认(citizen_identity)全部 reviewFields key 有中文标签', () {
      const keys = [
        'cid_number',
        'wallet_account',
        'citizen_age_years',
        'valid_range',
        'citizen_status',
        'residence',
      ];
      for (final key in keys) {
        expect(hasFieldLabel(key), isTrue, reason: key);
      }
    });

    test('公民身份上链交易(register_voting_identity)全部 reviewFields key 有中文标签', () {
      const keys = [
        'actor_cid_number',
        'cid_number',
        'wallet_account',
        'citizen_age_years',
        'valid_range',
        'citizen_status',
        'residence',
      ];
      for (final key in keys) {
        expect(hasFieldLabel(key), isTrue, reason: key);
      }
    });

    test('公民参选身份上链交易全部 reviewFields key 有中文标签', () {
      const keys = [
        'actor_cid_number',
        'identity_level',
        'cid_number',
        'wallet_account',
        'citizen_age_years',
        'valid_range',
        'citizen_status',
        'residence',
        'birth_place',
        'citizen_full_name',
        'citizen_sex',
      ];
      for (final key in keys) {
        expect(hasFieldLabel(key), isTrue, reason: key);
      }
    });

    test('公民身份字段翻译正确', () {
      expect(fieldLabelText('actor_cid_number'), '操作机构CID');
      expect(fieldLabelText('wallet_account'), '公民钱包账户');
      expect(fieldLabelText('citizen_age_years'), '周岁年龄');
      expect(fieldLabelText('valid_range'), '护照有效期');
      expect(fieldLabelText('citizen_status'), '身份状态');
      expect(fieldLabelText('residence'), '居住地');
      expect(fieldLabelText('birth_place'), '出生地');
      expect(fieldLabelText('citizen_full_name'), '公民姓名');
      expect(fieldLabelText('citizen_sex'), '公民性别');
    });

    test('机构协议新增字段翻译正确', () {
      expect(fieldLabelText('institution_account'), '机构账户');
      expect(fieldLabelText('credential_issuer_cid_number'), '凭证签发机构CID');
      expect(fieldLabelText('credential_signer_pubkey'), '凭证签发管理员');
      expect(fieldLabelText('account_names'), '机构账户名称');
      expect(fieldLabelText('effective_at'), '生效时间戳');
    });

    test('链上资产全部 reviewFields key 有中文标签', () {
      const keys = [
        'actor_cid_number',
        'execution_account',
        'asset_id',
        'asset_class',
        'asset_name',
        'asset_symbol',
        'asset_description',
        'decimals',
        'initial_supply_raw',
        'amount_raw',
        'from',
        'to',
        'who',
        'reason_hash',
      ];
      for (final key in keys) {
        expect(hasFieldLabel(key), isTrue, reason: key);
      }
    });

    test('amount_ 前缀按账户名展开', () {
      expect(fieldLabelText('amount_'), '账户金额');
      expect(fieldLabelText('amount_主账户'), '主账户金额');
    });

    test('未登记 key 不允许生成展示兜底', () {
      expect(fieldLabelTextOrNull('never_registered_key'), isNull);
      expect(hasFieldLabel('never_registered_key'), isFalse);
      expect(
        () => fieldLabelText('never_registered_key'),
        throwsStateError,
      );
    });
  });

  group('fieldValueText', () {
    test('approve 转换为赞成/反对', () {
      expect(fieldValueText('approve', 'true'), '赞成');
      expect(fieldValueText('approve', 'false'), '反对');
    });

    test('其他 key 原样返回', () {
      expect(fieldValueText('citizen_age_years', '22周岁'), '22周岁');
    });
  });
}
