import 'package:flutter/material.dart';

import 'package:citizenapp/transaction/personal-manage/personal_account_list_page.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 交易页顶部「多签账户」入口卡片。
///
/// 与同目录的多签转账入口卡片同形：左图标 + 标题 + 右向 chevron，整张卡是点击区
/// （chevron 只作指示，不单独包 InkWell）。
class PersonalAccountEntryCard extends StatelessWidget {
  const PersonalAccountEntryCard({super.key, this.onTap});

  /// 默认打开多签账户列表；测试可注入替身断言路由。
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: AppTheme.cardDecoration(),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: onTap ?? () => _openPersonalAccounts(context),
          borderRadius: BorderRadius.circular(AppTheme.radiusMd),
          child: const Padding(
            padding: EdgeInsets.symmetric(horizontal: 12, vertical: 16),
            child: Row(
              children: [
                RotatedBox(
                  // Material 的分享关系图旋转后为“一上两下”圆形节点，
                  // 与确认稿的多签关系结构和线性描边最接近。
                  quarterTurns: 1,
                  child: Icon(
                    Icons.share_outlined,
                    size: 24,
                    color: AppTheme.primary,
                  ),
                ),
                SizedBox(width: 12),
                Expanded(
                  child: Text(
                    '多签账户',
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: 15,
                      fontWeight: FontWeight.w600,
                      color: AppTheme.textPrimary,
                    ),
                  ),
                ),
                // 与链上交易状态行右侧同一颗 chevron。
                Icon(Icons.chevron_right, size: 22),
              ],
            ),
          ),
        ),
      ),
    );
  }

  void _openPersonalAccounts(BuildContext context) {
    Navigator.of(context).push(
      MaterialPageRoute(builder: (_) => const PersonalAccountListPage()),
    );
  }
}
