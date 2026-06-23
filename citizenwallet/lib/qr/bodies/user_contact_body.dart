import 'package:citizenwallet/qr/envelope.dart';

class UserContactBody implements QrBody {
  const UserContactBody({
    required this.address,
    required this.contactName,
  });

  final String address;
  final String contactName;

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'address': address,
        'contact_name': contactName,
      };

  static UserContactBody fromJson(Map<String, dynamic> data) {
    final address = data['address'];
    final contactName = data['contact_name'];
    if (address is! String || address.isEmpty) {
      throw const FormatException('user_contact.address 必填');
    }
    if (contactName is! String) {
      throw const FormatException('user_contact.contact_name 必填字符串');
    }
    return UserContactBody(address: address, contactName: contactName);
  }
}
