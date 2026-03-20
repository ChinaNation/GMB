import 'package:flutter/material.dart';

void main() {
  runApp(const NodeUiApp());
}

/// 新版 CitizenChain NodeUI 的桌面应用入口。
///
/// 当前阶段先提供清晰的迁移骨架，而不是直接复制旧版 Tauri 页面。
/// 这样可以先把桌面工程、导航结构和页面职责稳定下来，再逐步迁移旧功能。
class NodeUiApp extends StatelessWidget {
  const NodeUiApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'CitizenChain NodeUI',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF0B5D4B),
          brightness: Brightness.light,
        ),
        scaffoldBackgroundColor: const Color(0xFFF6F4EC),
        useMaterial3: true,
      ),
      home: const NodeUiHomePage(),
    );
  }
}

/// 新版 NodeUI 首页先承担“迁移控制台”职责：
/// 1. 明确告诉开发者当前仍以旧版 nodeuitauri 为运行基线；
/// 2. 给出新版 Flutter 需要覆盖的核心模块；
/// 3. 作为后续接入真实节点状态、挖矿、网络与设置页面的落点。
class NodeUiHomePage extends StatelessWidget {
  const NodeUiHomePage({super.key});

  static const List<_RoadmapItem> _roadmap = <_RoadmapItem>[
    _RoadmapItem('首页', '节点状态、链状态、身份与发行摘要'),
    _RoadmapItem('挖矿', '收益面板、出块记录、资源使用情况'),
    _RoadmapItem('网络', '节点网络总览、在线节点、Peer 统计'),
    _RoadmapItem('设置', '奖励地址、GRANDPA、Bootnode 与节点名称'),
    _RoadmapItem('打包', '替换旧版 nodeuitauri 的单安装包桌面发布'),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: const Color(0xFFE5EFE9),
        title: const Text('CitizenChain NodeUI'),
      ),
      body: LayoutBuilder(
        builder: (BuildContext context, BoxConstraints constraints) {
          final bool isWide = constraints.maxWidth >= 980;
          return SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: Center(
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 1100),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: <Widget>[
                    Container(
                      width: double.infinity,
                      padding: const EdgeInsets.all(24),
                      decoration: BoxDecoration(
                        color: Colors.white,
                        borderRadius: BorderRadius.circular(24),
                        boxShadow: const <BoxShadow>[
                          BoxShadow(
                            color: Color(0x14000000),
                            blurRadius: 24,
                            offset: Offset(0, 12),
                          ),
                        ],
                      ),
                      child: const Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: <Widget>[
                          Text(
                            '新版节点桌面 UI 已建立 Flutter Desktop 工程',
                            style: TextStyle(
                              fontSize: 28,
                              fontWeight: FontWeight.w700,
                            ),
                          ),
                          SizedBox(height: 12),
                          Text(
                            '当前可运行版本仍是 citizenchain/nodeuitauri。'
                            ' 新版 nodeui 将按模块迁移首页、挖矿、网络、设置，并最终替换旧版 Tauri 实现。',
                            style: TextStyle(
                              fontSize: 16,
                              height: 1.6,
                              color: Color(0xFF31433D),
                            ),
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 24),
                    Wrap(
                      spacing: 16,
                      runSpacing: 16,
                      children: <Widget>[
                        _StatusCard(
                          title: '当前运行基线',
                          value: 'nodeuitauri',
                          note: '旧版 Tauri 桌面壳继续负责打包与运行',
                          width: isWide ? 340 : double.infinity,
                        ),
                        _StatusCard(
                          title: '新版目录',
                          value: 'nodeui',
                          note: 'Flutter Desktop 正式入口',
                          width: isWide ? 340 : double.infinity,
                        ),
                        _StatusCard(
                          title: '迁移目标',
                          value: '单安装包',
                          note: '保持节点程序与桌面 UI 一体化交付',
                          width: isWide ? 340 : double.infinity,
                        ),
                      ],
                    ),
                    const SizedBox(height: 24),
                    const Text(
                      '功能迁移路线',
                      style: TextStyle(
                        fontSize: 22,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                    const SizedBox(height: 12),
                    ..._roadmap.map(
                      (_RoadmapItem item) => Padding(
                        padding: const EdgeInsets.only(bottom: 12),
                        child: Container(
                          width: double.infinity,
                          padding: const EdgeInsets.all(18),
                          decoration: BoxDecoration(
                            color: const Color(0xFFFFFCF4),
                            borderRadius: BorderRadius.circular(18),
                            border: Border.all(color: const Color(0xFFE7D8B3)),
                          ),
                          child: Row(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: <Widget>[
                              Container(
                                width: 36,
                                height: 36,
                                alignment: Alignment.center,
                                decoration: const BoxDecoration(
                                  color: Color(0xFF0B5D4B),
                                  shape: BoxShape.circle,
                                ),
                                child: Text(
                                  '${_roadmap.indexOf(item) + 1}',
                                  style: const TextStyle(
                                    color: Colors.white,
                                    fontWeight: FontWeight.w700,
                                  ),
                                ),
                              ),
                              const SizedBox(width: 16),
                              Expanded(
                                child: Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: <Widget>[
                                    Text(
                                      item.title,
                                      style: const TextStyle(
                                        fontSize: 18,
                                        fontWeight: FontWeight.w700,
                                      ),
                                    ),
                                    const SizedBox(height: 6),
                                    Text(
                                      item.description,
                                      style: const TextStyle(
                                        fontSize: 15,
                                        height: 1.5,
                                        color: Color(0xFF4C5B55),
                                      ),
                                    ),
                                  ],
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          );
        },
      ),
    );
  }
}

class _StatusCard extends StatelessWidget {
  const _StatusCard({
    required this.title,
    required this.value,
    required this.note,
    required this.width,
  });

  final String title;
  final String value;
  final String note;
  final double width;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: width,
      child: Container(
        padding: const EdgeInsets.all(20),
        decoration: BoxDecoration(
          color: const Color(0xFF103D35),
          borderRadius: BorderRadius.circular(20),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Text(
              title,
              style: const TextStyle(
                color: Color(0xFFB7D8CF),
                fontSize: 14,
              ),
            ),
            const SizedBox(height: 10),
            Text(
              value,
              style: const TextStyle(
                color: Colors.white,
                fontSize: 24,
                fontWeight: FontWeight.w700,
              ),
            ),
            const SizedBox(height: 10),
            Text(
              note,
              style: const TextStyle(
                color: Color(0xFFDDEAE5),
                height: 1.5,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _RoadmapItem {
  const _RoadmapItem(this.title, this.description);

  final String title;
  final String description;
}
