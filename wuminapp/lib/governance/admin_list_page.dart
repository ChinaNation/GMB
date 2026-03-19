import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';

import 'institution_data.dart';

/// 管理员列表页面。
///
/// 展示机构所有管理员的完整 SS58 地址，当前用户标记为"我"。
class AdminListPage extends StatelessWidget {
  const AdminListPage({
    super.key,
    required this.institution,
    required this.admins,
    required this.adminPubkeys,
    required this.badgeColor,
  });

  final InstitutionInfo institution;
  final List<String> admins;
  /// 当前用户导入的所有管理员公钥（小写 hex）。
  final Set<String> adminPubkeys;
  final Color badgeColor;

  static const Color _inkGreen = Color(0xFF0B3D2E);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '管理员列表',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: _inkGreen,
        elevation: 0,
        scrolledUnderElevation: 0.5,
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          // 机构信息
          _buildInstitutionHeader(),
          const SizedBox(height: 16),
          // 管理员总数
          Text(
            '共 ${admins.length} 位管理员　通过阈值 ${institution.internalThreshold}',
            style: TextStyle(fontSize: 13, color: Colors.grey[500]),
          ),
          const SizedBox(height: 12),
          // 管理员列表
          if (admins.isEmpty)
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 24),
              child: Center(
                child: Text(
                  '暂无管理员信息',
                  style: TextStyle(fontSize: 14, color: Colors.grey[400]),
                ),
              ),
            )
          else
            ...List.generate(admins.length, (index) {
              final pubkey = admins[index];
              final isSelf = adminPubkeys.contains(pubkey);
              return _AdminTile(
                index: index + 1,
                pubkeyHex: pubkey,
                isSelf: isSelf,
              );
            }),
        ],
      ),
    );
  }

  Widget _buildInstitutionHeader() {
    return Row(
      children: [
        Container(
          width: 36,
          height: 36,
          decoration: BoxDecoration(
            color: badgeColor.withValues(alpha: 0.12),
            borderRadius: BorderRadius.circular(10),
          ),
          child: Icon(Icons.people_outline, size: 18, color: badgeColor),
        ),
        const SizedBox(width: 10),
        Expanded(
          child: Text(
            institution.name,
            style: const TextStyle(
              fontSize: 15,
              fontWeight: FontWeight.w600,
              color: _inkGreen,
            ),
          ),
        ),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
          decoration: BoxDecoration(
            color: badgeColor.withValues(alpha: 0.10),
            borderRadius: BorderRadius.circular(10),
          ),
          child: Text(
            OrgType.label(institution.orgType),
            style: TextStyle(
              fontSize: 11,
              color: badgeColor,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
      ],
    );
  }
}

class _AdminTile extends StatelessWidget {
  const _AdminTile({
    required this.index,
    required this.pubkeyHex,
    required this.isSelf,
  });

  final int index;
  final String pubkeyHex;
  final bool isSelf;

  String _toSs58() {
    try {
      final bytes = _hexToBytes(pubkeyHex);
      return Keyring().encodeAddress(bytes, 2027);
    } catch (_) {
      return '0x$pubkeyHex';
    }
  }

  static Uint8List _hexToBytes(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = Uint8List(clean.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }

  @override
  Widget build(BuildContext context) {
    final address = _toSs58();

    return Container(
      margin: const EdgeInsets.only(bottom: 6),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      decoration: BoxDecoration(
        color: isSelf
            ? Colors.green.withValues(alpha: 0.06)
            : Colors.grey[50],
        borderRadius: BorderRadius.circular(10),
        border: Border.all(
          color: isSelf
              ? Colors.green.withValues(alpha: 0.3)
              : Colors.grey[200]!,
        ),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 24,
            child: Padding(
              padding: const EdgeInsets.only(top: 2),
              child: Text(
                '$index',
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w600,
                  color: Colors.grey[500],
                ),
              ),
            ),
          ),
          Expanded(
            child: Text(
              address,
              style: TextStyle(
                fontSize: 12,
                fontFamily: 'monospace',
                color: isSelf ? Colors.green[800] : Colors.grey[700],
                height: 1.4,
              ),
            ),
          ),
          const SizedBox(width: 6),
          Column(
            children: [
              if (isSelf)
                Container(
                  margin: const EdgeInsets.only(bottom: 4),
                  padding:
                      const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: Colors.green.withValues(alpha: 0.1),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: const Text(
                    '我',
                    style: TextStyle(
                      fontSize: 11,
                      color: Colors.green,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
              GestureDetector(
                onTap: () {
                  Clipboard.setData(ClipboardData(text: address));
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('地址已复制'),
                      duration: Duration(seconds: 1),
                    ),
                  );
                },
                child: Icon(Icons.copy, size: 16, color: Colors.grey[400]),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
