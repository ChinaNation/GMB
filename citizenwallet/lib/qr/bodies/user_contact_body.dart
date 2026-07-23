import 'package:citizenwallet/qr/envelope.dart';

class UserContactBody implements QrBody {
  const UserContactBody({
    required this.ss58Address,
    required this.contactName,
  });

  final String ss58Address;
  final String contactName;

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'ss58_address': ss58Address,
        'contact_name': contactName,
      };

  static UserContactBody fromJson(Map<String, dynamic> data) {
    final ss58Address = data['ss58_address'];
    final contactName = data['contact_name'];
    if (ss58Address is! String || ss58Address.isEmpty) {
      throw const FormatException('user_contact.ss58_address 必填');
    }
    if (contactName is! String) {
      throw const FormatException('user_contact.contact_name 必填字符串');
    }
    return UserContactBody(
      ss58Address: ss58Address,
      contactName: contactName,
    );
  }
}
