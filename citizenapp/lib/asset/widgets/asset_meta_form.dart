// 资产元数据表单 widget(框架阶段占位)— AssetIssuePage 子组件。
//
// 后续任务卡 C 实装时承载:
// - 4 个 TextField:name / symbol / description / decimals(数字键盘 0..=18)
// - 1 个 TextField:initial_supply(按 decimals 显示精度提示)
// - 提交前本地黑名单预校验:实时显示「字段命中"USD"等违禁词,请修改」红色提示
// - GMB 余额不足时,initial_supply 输入框下方显示警告

import 'package:flutter/material.dart';

class AssetMetaForm extends StatelessWidget {
  const AssetMetaForm({super.key});

  @override
  Widget build(BuildContext context) {
    // TODO: implement business logic
    return const Placeholder(fallbackHeight: 200);
  }
}
