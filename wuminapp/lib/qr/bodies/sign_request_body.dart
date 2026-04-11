import 'package:wuminapp_mobile/qr/envelope.dart';

/// kind = sign_request
///
/// 由 wuminapp 热钱包生成,wumin 冷钱包扫码后展示交易摘要并签名。
class SignRequestBody implements QrBody {
  const SignRequestBody({
    required this.address,
    required this.pubkey,
    required this.sigAlg,
    required this.payloadHex,
    required this.specVersion,
    required this.display,
  });

  final String address;
  final String pubkey;
  final String sigAlg;
  final String payloadHex;
  final int specVersion;
  final SignDisplay display;

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'address': address,
        'pubkey': pubkey,
        'sig_alg': sigAlg,
        'payload_hex': payloadHex,
        'spec_version': specVersion,
        'display': display.toJson(),
      };

  static SignRequestBody fromJson(Map<String, dynamic> data) {
    final address = data['address'];
    final pubkey = data['pubkey'];
    final sigAlg = data['sig_alg'];
    final payloadHex = data['payload_hex'];
    final specVersion = data['spec_version'];
    final display = data['display'];
    if (address is! String || address.isEmpty) {
      throw const FormatException('sign_request.address 必填');
    }
    if (pubkey is! String || !pubkey.startsWith('0x')) {
      throw const FormatException('sign_request.pubkey 必填 0x hex');
    }
    if (sigAlg != 'sr25519') {
      throw const FormatException('sign_request.sig_alg 必须为 sr25519');
    }
    if (payloadHex is! String || !payloadHex.startsWith('0x')) {
      throw const FormatException('sign_request.payload_hex 必填 0x hex');
    }
    if (payloadHex.length > 32768) {
      throw const FormatException('sign_request.payload_hex 超过 32768 字符');
    }
    if (specVersion is! int) {
      throw const FormatException('sign_request.spec_version 必填整数');
    }
    if (display is! Map<String, dynamic>) {
      throw const FormatException('sign_request.display 必填对象');
    }
    return SignRequestBody(
      address: address,
      pubkey: pubkey,
      sigAlg: sigAlg,
      payloadHex: payloadHex,
      specVersion: specVersion,
      display: SignDisplay.fromJson(display),
    );
  }
}

class SignDisplay {
  const SignDisplay({
    required this.action,
    required this.summary,
    this.fields = const [],
  });

  final String action;
  final String summary;
  final List<SignDisplayField> fields;

  Map<String, dynamic> toJson() => <String, dynamic>{
        'action': action,
        'summary': summary,
        'fields': fields.map((f) => f.toJson()).toList(),
      };

  static SignDisplay fromJson(Map<String, dynamic> data) {
    final action = data['action'];
    final summary = data['summary'];
    final fields = data['fields'];
    if (action is! String || action.isEmpty) {
      throw const FormatException('display.action 必填');
    }
    if (summary is! String || summary.isEmpty) {
      throw const FormatException('display.summary 必填');
    }
    final parsedFields = <SignDisplayField>[];
    if (fields is List) {
      for (final f in fields) {
        if (f is Map<String, dynamic>) {
          parsedFields.add(SignDisplayField.fromJson(f));
        }
      }
    }
    return SignDisplay(
      action: action,
      summary: summary,
      fields: parsedFields,
    );
  }
}

class SignDisplayField {
  const SignDisplayField({this.key, required this.label, required this.value});

  /// 可选的英文字段标识，用于与 PayloadDecoder 解码结果交叉比对。
  /// 链端 signing.rs 会发送此字段，wumin/wuminapp 生成时可省略。
  final String? key;
  final String label;
  final String value;

  Map<String, dynamic> toJson() {
    final map = <String, dynamic>{
      'label': label,
      'value': value,
    };
    if (key != null) map['key'] = key;
    return map;
  }

  static SignDisplayField fromJson(Map<String, dynamic> data) {
    final label = data['label'];
    final value = data['value'];
    if (label is! String || value is! String) {
      throw const FormatException('display.fields[*] 需含 label 与 value');
    }
    final key = data['key'];
    return SignDisplayField(
      key: key is String ? key : null,
      label: label,
      value: value,
    );
  }
}
