import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/my/util/amount_format.dart';
import 'institution_role_models.dart';

/// 机构管理员任职卡；只展示管理员钱包、机构岗位、任期和制度来源。
class InstitutionAssignmentCard extends StatelessWidget {
  const InstitutionAssignmentCard({
    super.key,
    required this.assignment,
    this.index,
    this.balanceYuan,
    this.trailing,
  });

  final InstitutionAdminAssignment assignment;
  final int? index;
  final double? balanceYuan;
  final Widget? trailing;
  static const double actionHeight = 30;

  @override
  Widget build(BuildContext context) => Card(
        child: Padding(
          padding: const EdgeInsets.all(12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(children: [
                if (index != null) Text('$index　'),
                Expanded(
                    child: Text(assignment.roleName,
                        style: Theme.of(context).textTheme.titleMedium)),
                if (trailing != null) trailing!,
              ]),
              Text('任期：${assignment.termLabel}'),
              Text('任职来源：${assignment.source.label}'),
              if (assignment.sourceRef.isNotEmpty)
                Text('来源引用：${assignment.sourceRef}'),
              Text('管理员账户：${ss58FromHex(assignment.adminAccount)}'),
              if (balanceYuan != null)
                Text('余额：${AmountFormat.formatThousands(balanceYuan)} 元'),
            ],
          ),
        ),
      );
}
