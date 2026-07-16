import 'package:flutter_test/flutter_test.dart';
import 'package:citizenwallet/signer/field_labels.dart';

void main() {
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
        expect(fieldLabelText(key), isNot('未知字段'), reason: key);
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
        expect(fieldLabelText(key), isNot('未知字段'), reason: key);
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
        expect(fieldLabelText(key), isNot('未知字段'), reason: key);
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

    test('amount_ 前缀按账户名展开', () {
      expect(fieldLabelText('amount_'), '账户金额');
      expect(fieldLabelText('amount_主账户'), '主账户金额');
    });

    test('未登记 key 用中文兜底', () {
      expect(fieldLabelText('never_registered_key'), '未知字段');
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
