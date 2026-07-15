import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/institution/institution.dart';
import 'package:citizenapp/citizen/institution/institution_detail_page.dart';
import 'package:citizenapp/citizen/institution/institution_repository.dart';
import 'package:citizenapp/citizen/public/data/public_provinces.dart';
import 'package:citizenapp/citizen/legislation/law_reader_page.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 立法 tab 视图(ADR-028 P3-1)。
///
/// 固定顶部 5 卡(公民宪法整行 + 国家立法院/国家教委会 + 国家众议会/国家参议会)
/// +「省市立法机构」标签(不滚);下方省竖导航(去关注组)+ 选中省的 省立法院/省众议会/
/// 省参议会 + 该省全部市立法会。机构卡 → 统一详情页;宪法卡 → 条款项阅读器。
class LegislationTab extends StatefulWidget {
  const LegislationTab({super.key, this.repository});

  final InstitutionRepository? repository;

  @override
  State<LegislationTab> createState() => _LegislationTabState();
}

/// 立法宪法 law_id 固定为 0。
const int _kConstitutionLawId = 0;

// 国家级立法机构码(顶部卡)。
const String _codeNlg = 'NLG'; // 国家立法院
const String _codeNed = 'NED'; // 国家公民教育委员会
const String _codeNrp = 'NRP'; // 国家众议会
const String _codeNsn = 'NSN'; // 国家参议会

/// 顶部卡图标 chip 规格(方案二·五色语义):圆角方形浅底 + 深色图标。
class _CardIcon {
  const _CardIcon(this.icon, this.bg, this.fg);
  final IconData icon;
  final Color bg; // chip 底色(浅)
  final Color fg; // 图标色(深)
}

// 公民宪法卡:书本 + 翠绿。
const _CardIcon _constitutionIcon =
    _CardIcon(Icons.menu_book, Color(0xFFE1F5EE), Color(0xFF0F6E56));

// 四院卡:机构码 → 图标 chip,颜色按机构固定(与展示位置解耦)。
const Map<String, _CardIcon> _nationalIcons = {
  _codeNlg:
      _CardIcon(Icons.account_balance, Color(0xFFFAEEDA), Color(0xFF854F0B)),
  _codeNed: _CardIcon(Icons.school, Color(0xFFE6F1FB), Color(0xFF185FA5)),
  _codeNsn: _CardIcon(Icons.gavel, Color(0xFFEEEDFE), Color(0xFF3C3489)),
  _codeNrp: _CardIcon(Icons.groups, Color(0xFFEAF3DE), Color(0xFF3B6D11)),
};

// 省内立法机构码(省导航右侧内容),按展示顺序。
const List<String> _provinceCodeOrder = ['PLG', 'PRP', 'PSN', 'CLEG'];
const Set<String> _provinceCodes = {'PLG', 'PRP', 'PSN', 'CLEG'};

class _LegislationTabState extends State<LegislationTab> {
  late final InstitutionRepository _repo =
      widget.repository ?? InstitutionRepository();

  /// 国家级机构(code → Institution),缺失则该卡占位。
  final Map<String, Institution> _national = {};

  List<PublicProvinceItem> _provinces = const [];
  String? _selectedProvince;
  List<Institution> _provinceContent = const [];
  bool _contentLoading = true;

  @override
  void initState() {
    super.initState();
    _bootstrap();
  }

  Future<void> _bootstrap() async {
    _provinces = publicProvinceItems();
    final nationals =
        await _repo.listByCodes({_codeNlg, _codeNed, _codeNrp, _codeNsn});
    for (final inst in nationals) {
      _national[inst.institutionCode] = inst;
    }
    if (!mounted) return;
    setState(() {});
    if (_provinces.isNotEmpty) {
      await _selectProvince(_provinces.first.code);
    } else {
      if (mounted) setState(() => _contentLoading = false);
    }
  }

  Future<void> _selectProvince(String provinceCode) async {
    setState(() {
      _selectedProvince = provinceCode;
      _contentLoading = true;
    });
    final rows =
        await _repo.listByProvinceAndCodes(provinceCode, _provinceCodes);
    final sorted = [...rows]..sort((a, b) {
        final oa = _provinceCodeOrder.indexOf(a.institutionCode);
        final ob = _provinceCodeOrder.indexOf(b.institutionCode);
        if (oa != ob) return oa.compareTo(ob);
        return a.cidShortNameOrFullName.compareTo(b.cidShortNameOrFullName);
      });
    if (!mounted || _selectedProvince != provinceCode) return;
    setState(() {
      _provinceContent = sorted;
      _contentLoading = false;
    });
  }

  void _openDetail(String cidNumber) {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) =>
            InstitutionDetailPage(cidNumber: cidNumber, repository: _repo),
      ),
    );
  }

  void _openConstitution() {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => const LawReaderPage(lawId: _kConstitutionLawId),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // ── 固定顶部(不滚)──
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 12, 16, 0),
          child: Column(
            children: [
              _constitutionCard(),
              const SizedBox(height: 8),
              Row(children: [
                Expanded(child: _nationalCard(_codeNlg, '国家立法院')),
                const SizedBox(width: 8),
                Expanded(child: _nationalCard(_codeNed, '国家教委会')),
              ]),
              const SizedBox(height: 8),
              Row(children: [
                Expanded(child: _nationalCard(_codeNsn, '国家参议会')),
                const SizedBox(width: 8),
                Expanded(child: _nationalCard(_codeNrp, '国家众议会')),
              ]),
              const SizedBox(height: 14),
              const Align(
                alignment: Alignment.centerLeft,
                child: Text('省市立法机构',
                    style: TextStyle(
                        fontSize: 15,
                        fontWeight: FontWeight.w700,
                        color: AppTheme.textPrimary)),
              ),
              const SizedBox(height: 6),
            ],
          ),
        ),
        // ── 省导航 body(左省栏 + 右内容,各自滚)──
        Expanded(
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _provinceRail(),
              Expanded(child: _provinceContentView()),
            ],
          ),
        ),
      ],
    );
  }

  Widget _constitutionCard() {
    return InkWell(
      onTap: _openConstitution,
      borderRadius: BorderRadius.circular(12),
      child: Container(
        decoration: BoxDecoration(
          color: AppTheme.surfaceCard,
          borderRadius: BorderRadius.circular(12),
          border:
              Border.all(color: AppTheme.primaryDark.withValues(alpha: 0.22)),
        ),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
        child: Row(
          children: [
            _iconChip(_constitutionIcon, size: 38),
            const SizedBox(width: 12),
            const Expanded(
              child: Text('公民宪法',
                  style: TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.w700,
                      color: AppTheme.primaryDark)),
            ),
            const Icon(Icons.chevron_right,
                size: 20, color: AppTheme.textTertiary),
          ],
        ),
      ),
    );
  }

  Widget _nationalCard(String code, String fallbackLabel) {
    final inst = _national[code];
    final label = inst != null ? inst.cidShortNameOrFullName : fallbackLabel;
    final enabled = inst != null;
    final spec = _nationalIcons[code]!;
    return InkWell(
      onTap: enabled ? () => _openDetail(inst.cidNumber) : null,
      borderRadius: BorderRadius.circular(12),
      child: Container(
        decoration: BoxDecoration(
          color: AppTheme.surfaceCard,
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: AppTheme.border),
        ),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
        child: Row(
          children: [
            _iconChip(spec, size: 34, enabled: enabled),
            const SizedBox(width: 10),
            Expanded(
              child: Text(label,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                      fontSize: 13,
                      height: 1.15,
                      fontWeight: FontWeight.w600,
                      color: enabled
                          ? AppTheme.textPrimary
                          : AppTheme.textTertiary)),
            ),
            const Icon(Icons.chevron_right,
                size: 18, color: AppTheme.textTertiary),
          ],
        ),
      ),
    );
  }

  /// 圆角方形图标 chip(方案二):浅底 + 深图标;禁用态转灰。
  Widget _iconChip(_CardIcon spec, {double size = 34, bool enabled = true}) {
    return Container(
      width: size,
      height: size,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        color: enabled ? spec.bg : AppTheme.surfaceElevated,
        borderRadius: BorderRadius.circular(10),
      ),
      child: Icon(spec.icon,
          size: size * 0.52, color: enabled ? spec.fg : AppTheme.textTertiary),
    );
  }

  Widget _provinceRail() {
    return SizedBox(
      width: 84,
      child: ListView.builder(
        padding: const EdgeInsets.fromLTRB(6, 0, 6, 12),
        itemCount: _provinces.length,
        itemBuilder: (context, i) {
          final p = _provinces[i];
          final active = p.code == _selectedProvince;
          return Padding(
            padding: const EdgeInsets.only(bottom: 4),
            child: InkWell(
              borderRadius: BorderRadius.circular(10),
              onTap: () => _selectProvince(p.code),
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 150),
                width: double.infinity,
                padding:
                    const EdgeInsets.symmetric(vertical: 11, horizontal: 6),
                decoration: BoxDecoration(
                  color: active ? AppTheme.surfaceElevated : Colors.transparent,
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Text(p.provinceDisplayName,
                    textAlign: TextAlign.center,
                    style: TextStyle(
                        fontSize: active ? 16 : 15,
                        fontWeight: active ? FontWeight.w700 : FontWeight.w500,
                        color: active
                            ? AppTheme.primary
                            : AppTheme.textSecondary)),
              ),
            ),
          );
        },
      ),
    );
  }

  Widget _provinceContentView() {
    if (_contentLoading) {
      return const Center(child: CircularProgressIndicator(strokeWidth: 2));
    }
    if (_provinceContent.isEmpty) {
      return const Center(
        child: Text('该省暂无立法机构数据',
            style: TextStyle(fontSize: 13, color: AppTheme.textTertiary)),
      );
    }
    return ListView.separated(
      padding: const EdgeInsets.fromLTRB(4, 0, 12, 12),
      itemCount: _provinceContent.length,
      separatorBuilder: (_, __) =>
          const Divider(height: 1, color: AppTheme.divider),
      itemBuilder: (context, i) {
        final inst = _provinceContent[i];
        return ListTile(
          dense: true,
          title: Text(inst.cidShortNameOrFullName,
              style: const TextStyle(
                  fontSize: 14,
                  fontWeight: FontWeight.w600,
                  color: AppTheme.textPrimary)),
          trailing: const Icon(Icons.chevron_right,
              color: AppTheme.textTertiary, size: 20),
          onTap: () => _openDetail(inst.cidNumber),
        );
      },
    );
  }
}
