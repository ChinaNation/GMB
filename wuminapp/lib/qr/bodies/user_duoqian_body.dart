import 'package:wuminapp_mobile/qr/envelope.dart';

/// kind = user_duoqian(**固定码**,envelope 顶层无 id / issued_at / expires_at)
class UserDuoqianBody implements QrBody {
  const UserDuoqianBody({
    required this.address,
    required this.name,
    required this.proposalId,
  });

  final String address;
  final String name;
  final int proposalId;

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'address': address,
        'name': name,
        'proposal_id': proposalId,
      };

  static UserDuoqianBody fromJson(Map<String, dynamic> data) {
    final address = data['address'];
    final name = data['name'];
    final proposalId = data['proposal_id'];
    if (address is! String || address.isEmpty) {
      throw const FormatException('user_duoqian.address 必填');
    }
    if (name is! String) {
      throw const FormatException('user_duoqian.name 必填字符串');
    }
    if (proposalId is! int) {
      throw const FormatException('user_duoqian.proposal_id 必填整数');
    }
    return UserDuoqianBody(
      address: address,
      name: name,
      proposalId: proposalId,
    );
  }
}
