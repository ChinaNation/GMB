import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/legislation/data/law_models.dart';
import 'package:citizenapp/citizen/legislation/data/legislation_api.dart';
import 'package:citizenapp/citizen/legislation/law_reader_page.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 某立法机构的法律列表(ADR-028 P3-1)——`list_laws(tier, scope)`。
///
/// 中文注释:由立法机构详情页「法律原文」入口进入;tier/scope 由机构派生。
/// 列表项标题取公民端默认阅读版本(同一 api 实例缓存,点进阅读器复用不重拉)。
class LawListPage extends StatefulWidget {
  const LawListPage({
    super.key,
    required this.tier,
    required this.scopeCode,
    required this.title,
    this.api,
  });

  final LawTier tier;
  final int scopeCode;

  /// AppBar 标题(机构简称 + 「法律原文」)。
  final String title;
  final LegislationApi? api;

  @override
  State<LawListPage> createState() => _LawListPageState();
}

class _LawItem {
  const _LawItem(
      {required this.lawId, required this.title, required this.status});
  final int lawId;
  final String title;
  final LawStatus status;
}

class _LawListPageState extends State<LawListPage> {
  late final LegislationApi _api = widget.api ?? LegislationApi();

  List<_LawItem> _items = const [];
  bool _loading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final ids = await _api.listLaws(widget.tier, widget.scopeCode);
      final items = await Future.wait(ids.map((id) async {
        final law = await _api.law(id);
        if (law == null) return null;
        final versionId = law.readerVersion;
        final v =
            versionId == null ? null : await _api.lawVersion(id, versionId);
        return _LawItem(
          lawId: id,
          title: (v?.title.isNotEmpty ?? false) ? v!.title : '法律 #$id',
          status: law.status,
        );
      }));
      if (!mounted) return;
      setState(() {
        _items = items.whereType<_LawItem>().toList(growable: false);
        _loading = false;
      });
    } on Object {
      if (mounted) {
        setState(() {
          _loading = false;
          _error = '法律列表读取失败，请检查网络后重试';
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        title: Text(widget.title),
        backgroundColor: AppTheme.surfaceWhite,
        foregroundColor: AppTheme.textPrimary,
        elevation: 0,
      ),
      body: _buildBody(),
    );
  }

  Widget _buildBody() {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(strokeWidth: 2));
    }
    if (_error != null) {
      return Center(
        child:
            Text(_error!, style: const TextStyle(color: AppTheme.textTertiary)),
      );
    }
    if (_items.isEmpty) {
      return const Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.menu_book_outlined,
                size: 44, color: AppTheme.textTertiary),
            SizedBox(height: 12),
            Text('该机构暂无立法',
                style: TextStyle(fontSize: 14, color: AppTheme.textSecondary)),
          ],
        ),
      );
    }
    return ListView.separated(
      padding: const EdgeInsets.all(16),
      itemCount: _items.length,
      separatorBuilder: (_, __) => const SizedBox(height: 10),
      itemBuilder: (context, i) => _lawCard(_items[i]),
    );
  }

  Widget _lawCard(_LawItem item) {
    final statusColor = switch (item.status) {
      LawStatus.effective => AppTheme.success,
      LawStatus.repealed => AppTheme.danger,
      LawStatus.pending => AppTheme.primary,
    };
    return Container(
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: AppTheme.border),
      ),
      child: InkWell(
        onTap: () => Navigator.of(context).push(
          MaterialPageRoute<void>(
            builder: (_) => LawReaderPage(lawId: item.lawId, api: _api),
          ),
        ),
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
          child: Row(
            children: [
              const Icon(Icons.menu_book_outlined,
                  size: 20, color: AppTheme.primaryDark),
              const SizedBox(width: 12),
              Expanded(
                child: Text(item.title,
                    style: const TextStyle(
                        fontSize: 14.5,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.textPrimary)),
              ),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                decoration: BoxDecoration(
                  color: statusColor.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Text(item.status.label,
                    style: TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.w600,
                        color: statusColor)),
              ),
              const SizedBox(width: 4),
              const Icon(Icons.chevron_right,
                  size: 20, color: AppTheme.textTertiary),
            ],
          ),
        ),
      ),
    );
  }
}
