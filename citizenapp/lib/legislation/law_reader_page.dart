import 'package:flutter/material.dart';

import 'package:citizenapp/legislation/data/block_clock.dart';
import 'package:citizenapp/legislation/data/law_models.dart';
import 'package:citizenapp/legislation/data/legislation_api.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 法律条款项阅读器(ADR-028 P3-1)——宪法与普通法律共用。
///
/// 中文注释:渲染 章>节>条>款>项;宪法(tier=宪法)对不可修改条款渲染徽章、双语可切;
/// 顶部显状态 + 生效日期(块号经 [BlockClock] 换算)。读链:law + law_version(当前版本)
/// + (宪法)immutableManifest。
class LawReaderPage extends StatefulWidget {
  const LawReaderPage({
    super.key,
    required this.lawId,
    this.api,
    this.clock,
  });

  final int lawId;
  final LegislationApi? api;
  final BlockClock? clock;

  @override
  State<LawReaderPage> createState() => _LawReaderPageState();
}

class _LawReaderPageState extends State<LawReaderPage> {
  late final LegislationApi _api = widget.api ?? LegislationApi();
  late final BlockClock _clock = widget.clock ?? BlockClock();

  Law? _law;
  LawVersion? _version;
  ImmutableManifest? _manifest;
  String? _effectiveDate;
  bool _loading = true;
  String? _error;
  bool _showEn = false;
  final Set<int> _expanded = {};

  /// 当前查看的版本号(默认=当前生效版本;版本史可切到历史版本)。
  int _selectedVersion = 0;

  bool get _isConstitution => _law?.tier == LawTier.constitution;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final law = await _api.law(widget.lawId);
      if (law == null) {
        if (mounted) {
          setState(() {
            _loading = false;
            _error = '未找到该法律';
          });
        }
        return;
      }
      final version = await _api.lawVersion(law.lawId, law.currentVersion);
      final manifest = law.tier == LawTier.constitution
          ? await _api.immutableManifest()
          : null;
      String? effDate;
      if (version != null) {
        effDate = BlockClock.formatDate(await _clock.dateOf(version.effectiveAt));
      }
      if (!mounted) return;
      setState(() {
        _law = law;
        _version = version;
        _selectedVersion = law.currentVersion;
        _manifest = manifest;
        _effectiveDate = effDate;
        // 默认展开第一章,长法律不至于一片空白。
        if (version != null && version.chapters.isNotEmpty) {
          _expanded.add(version.chapters.first.number);
        }
        _loading = false;
      });
    } on Object {
      if (mounted) {
        setState(() {
          _loading = false;
          _error = '法律读取失败，请检查网络后重试';
        });
      }
    }
  }

  /// 切换到历史版本(版本史)。重新拉取该版本正文并重置展开态。
  Future<void> _changeVersion(int version) async {
    final law = _law;
    if (law == null || version == _selectedVersion) return;
    setState(() => _loading = true);
    try {
      final v = await _api.lawVersion(law.lawId, version);
      final effDate = v == null
          ? null
          : BlockClock.formatDate(await _clock.dateOf(v.effectiveAt));
      if (!mounted) return;
      setState(() {
        _version = v;
        _selectedVersion = version;
        _effectiveDate = effDate;
        _expanded
          ..clear()
          ..addAll(
              v != null && v.chapters.isNotEmpty ? [v.chapters.first.number] : const []);
        _loading = false;
      });
    } on Object {
      if (mounted) {
        setState(() {
          _loading = false;
          _error = '版本读取失败，请检查网络后重试';
        });
      }
    }
  }

  String _t(String zh, String? en) => (_showEn && en != null && en.isNotEmpty) ? en : zh;

  @override
  Widget build(BuildContext context) {
    final v = _version;
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        title: Text(v == null ? '法律' : _t(v.title, v.titleEn)),
        backgroundColor: AppTheme.surfaceWhite,
        foregroundColor: AppTheme.textPrimary,
        elevation: 0,
        actions: [
          if (_isConstitution && v?.titleEn != null)
            TextButton(
              onPressed: () => setState(() => _showEn = !_showEn),
              child: Text(_showEn ? '中' : 'EN',
                  style: const TextStyle(
                      fontSize: 13, fontWeight: FontWeight.w600)),
            ),
        ],
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
        child: Text(_error!, style: const TextStyle(color: AppTheme.textTertiary)),
      );
    }
    final v = _version;
    final law = _law;
    if (v == null || law == null) {
      return const Center(child: Text('暂无正文'));
    }
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        _header(law, v),
        const SizedBox(height: 12),
        ...v.chapters.map(_chapterTile),
      ],
    );
  }

  Widget _header(Law law, LawVersion v) {
    final statusColor = switch (law.status) {
      LawStatus.effective => AppTheme.success,
      LawStatus.repealed => AppTheme.danger,
      LawStatus.pending => AppTheme.primary,
    };
    final dateLabel = switch (law.status) {
      LawStatus.effective => '生效中',
      LawStatus.repealed => '已废止',
      LawStatus.pending => '将于 ${_effectiveDate ?? '—'} 生效',
    };
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: AppTheme.primary.withValues(alpha: 0.18)),
      ),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
            decoration: BoxDecoration(
              color: statusColor.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(8),
            ),
            child: Text('${law.tier.label} · v${v.version}',
                style: TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    color: statusColor)),
          ),
          // 表决类型徽章(创世宪法 proposalId=0 不显示)。
          if (v.proposalId != 0) ...[
            const SizedBox(width: 6),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
              decoration: BoxDecoration(
                color: AppTheme.primary.withValues(alpha: 0.08),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Text(v.voteTypeEnum.label,
                  style: const TextStyle(
                      fontSize: 11,
                      fontWeight: FontWeight.w600,
                      color: AppTheme.primary)),
            ),
          ],
          const SizedBox(width: 10),
          Expanded(
            child: Text(dateLabel,
                style: const TextStyle(
                    fontSize: 12.5, color: AppTheme.textSecondary)),
          ),
          // 版本史:多版本时可切换查看历史版本。
          if (law.currentVersion > 1) _versionMenu(law),
        ],
      ),
    );
  }

  Widget _versionMenu(Law law) {
    return PopupMenuButton<int>(
      tooltip: '版本历史',
      onSelected: _changeVersion,
      itemBuilder: (_) => [
        for (var ver = law.currentVersion; ver >= 1; ver--)
          PopupMenuItem<int>(
            value: ver,
            child: Text(
              ver == law.currentVersion ? 'v$ver(当前)' : 'v$ver',
              style: TextStyle(
                fontWeight: ver == _selectedVersion
                    ? FontWeight.w700
                    : FontWeight.w400,
              ),
            ),
          ),
      ],
      child: const Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.history, size: 16, color: AppTheme.textSecondary),
          SizedBox(width: 2),
          Text('版本史',
              style: TextStyle(fontSize: 12, color: AppTheme.textSecondary)),
        ],
      ),
    );
  }

  Widget _chapterTile(LawChapter ch) {
    final expanded = _expanded.contains(ch.number);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        InkWell(
          onTap: () => setState(() {
            if (expanded) {
              _expanded.remove(ch.number);
            } else {
              _expanded.add(ch.number);
            }
          }),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 10),
            child: Row(
              children: [
                Expanded(
                  child: Text('第${ch.number}章 ${_t(ch.title, ch.titleEn)}',
                      style: const TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.w700,
                          color: AppTheme.primaryDark)),
                ),
                Icon(expanded ? Icons.keyboard_arrow_down : Icons.chevron_right,
                    color: AppTheme.textSecondary),
              ],
            ),
          ),
        ),
        if (expanded)
          ...ch.sections.map(_sectionTile),
        const Divider(height: 1, color: AppTheme.divider),
      ],
    );
  }

  Widget _sectionTile(LawSection sec) {
    return Padding(
      padding: const EdgeInsets.only(left: 6, top: 6, bottom: 6),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (sec.title.isNotEmpty)
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 6),
              child: Text('第${sec.number}节 ${_t(sec.title, sec.titleEn)}',
                  style: const TextStyle(
                      fontSize: 14,
                      fontWeight: FontWeight.w600,
                      color: AppTheme.textSecondary)),
            ),
          ...sec.articles.map(_articleTile),
        ],
      ),
    );
  }

  Widget _articleTile(LawArticle art) {
    final immutable = _manifest?.isImmutable(art.number) ?? false;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Flexible(
                child: Text('第${art.number}条 ${_t(art.title, art.titleEn)}',
                    style: const TextStyle(
                        fontSize: 14.5,
                        fontWeight: FontWeight.w700,
                        color: AppTheme.textPrimary)),
              ),
              if (immutable) ...[
                const SizedBox(width: 8),
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
                  decoration: BoxDecoration(
                    color: AppTheme.danger.withValues(alpha: 0.1),
                    borderRadius: BorderRadius.circular(6),
                  ),
                  child: const Text('不可修改',
                      style: TextStyle(
                          fontSize: 10,
                          fontWeight: FontWeight.w600,
                          color: AppTheme.danger)),
                ),
              ],
            ],
          ),
          if (art.body.isNotEmpty) ...[
            const SizedBox(height: 4),
            Text(_t(art.body, art.bodyEn),
                style: const TextStyle(
                    fontSize: 13.5, height: 1.6, color: AppTheme.textPrimary)),
          ],
          ...art.clauses.map((c) => Padding(
                padding: const EdgeInsets.only(top: 6, left: 8),
                child: Text('${_clauseNo(c.number)} ${_t(c.text, c.textEn)}',
                    style: const TextStyle(
                        fontSize: 13,
                        height: 1.6,
                        color: AppTheme.textSecondary)),
              )),
        ],
      ),
    );
  }

  static String _clauseNo(int n) {
    const cn = ['一', '二', '三', '四', '五', '六', '七', '八', '九', '十'];
    final label = (n >= 1 && n <= 10) ? cn[n - 1] : '$n';
    return '第$label款';
  }
}
