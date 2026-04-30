import 'package:flutter/material.dart';

import '../shared/duoqian_account_list_page.dart';
import '../shared/duoqian_account_type.dart';

/// 机构多签入口页。
class InstitutionDuoqianListPage extends StatelessWidget {
  const InstitutionDuoqianListPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const DuoqianAccountListPage(
      accountType: DuoqianAccountType.institution,
    );
  }
}
