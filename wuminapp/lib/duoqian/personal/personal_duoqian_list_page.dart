import 'package:flutter/material.dart';

import '../shared/duoqian_account_list_page.dart';
import '../shared/duoqian_account_type.dart';

/// 个人多签入口页。
class PersonalDuoqianListPage extends StatelessWidget {
  const PersonalDuoqianListPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const DuoqianAccountListPage(
      accountType: DuoqianAccountType.personal,
    );
  }
}
