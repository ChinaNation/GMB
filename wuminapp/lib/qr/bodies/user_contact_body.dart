import 'package:wuminapp_mobile/qr/envelope.dart';

/// kind = user_contact(**固定码**,envelope 顶层无 id / issued_at / expires_at)
class UserContactBody implements QrBody {
  const UserContactBody({
    required this.address,
    required this.name,
  });

  final String address;
  final String name;

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'address': address,
        'name': name,
      };

  static UserContactBody fromJson(Map<String, dynamic> data) {
    final address = data['address'];
    final name = data['name'];
    if (address is! String || address.isEmpty) {
      throw const FormatException('user_contact.address 必填');
    }
    if (name is! String) {
      throw const FormatException('user_contact.name 必填字符串');
    }
    return UserContactBody(address: address, name: name);
  }
}
