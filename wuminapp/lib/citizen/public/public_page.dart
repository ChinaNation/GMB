import 'dart:async';

import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/citizen/public/city_institution_list_page.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_repository.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_provinces.dart';
import 'package:wuminapp_mobile/citizen/public/public_institution_detail_page.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 公民-公权 tab:公权机构目录浏览 + 订阅(ADR-018 §九 卡B)。
///
/// 中文注释:左侧导航——「关注」**钉顶固定不滚**,下方 43 省可上下滚动;展示去"省"
/// (匹配仍用全名)。右侧按选中项展示:关注=我订阅的机构(跨省扁平);某省=该省市列表
/// (点市进机构列表)。**读全程本地优先(秒开),在线增量丢后台**,绝不阻塞转圈。
class PublicPage extends StatefulWidget {
  const PublicPage({super.key, this.repository, this.walletPubkeyProvider});

  /// 测试注入;生产为本地 Isar 实现。
  final PublicInstitutionRepository? repository;

  /// 活动钱包公钥(关注分组按公钥隔离);测试可注入,默认读 WalletManager。
  final Future<String?> Function()? walletPubkeyProvider;

  @override
  State<PublicPage> createState() => _PublicPageState();
}

/// 关注分组的固定标签(置顶,非省份)。
const String _kFollowGroup = '关注';

class _PublicPageState extends State<PublicPage> {
  late final PublicInstitutionRepository _repo =
      widget.repository ?? PublicInstitutionRepository();

  List<String> _provinces = const [];
  String _selected = _kFollowGroup;
  String? _activePubkey;

  List<String> _cities = const [];
  List<PublicInstitutionEntity> _subscribed = const [];
  bool _contentLoading = true;
  String? _contentError;

  @override
  void initState() {
    super.initState();
    _bootstrap();
  }

  Future<String?> _resolvePubkey() async {
    final provider = widget.walletPubkeyProvider;
    if (provider != null) return provider();
    return (await WalletManager().getWallet())?.pubkeyHex;
  }

  Future<void> _bootstrap() async {
    // 首启后台灌入数据包基线(库空才灌,非阻塞)。
    unawaited(_repo.ensureBundleLoaded());
    final pubkey = await _resolvePubkey();
    if (!mounted) return;
    setState(() {
      _activePubkey = pubkey;
      // 省份是固定行政区(43 省),始终全显,与数据是否加载无关。
      _provinces = publicProvinceNames();
    });
    await _selectGroup(_kFollowGroup);
  }

  Future<void> _selectGroup(String group) async {
    setState(() {
      _selected = group;
      _contentLoading = true;
      _contentError = null;
    });
    if (group == _kFollowGroup) {
      final subs = _activePubkey == null
          ? <PublicInstitutionEntity>[]
          : await _repo.listSubscribed(_activePubkey!);
      if (!mounted) return;
      setState(() {
        _subscribed = subs;
        _contentLoading = false;
      });
      return;
    }
    // 省:**先读本地秒显**(不等网络),再后台增量刷新。
    final localCities = await _repo.listCities(group);
    if (!mounted) return;
    setState(() {
      _cities = localCities;
      _contentLoading = false;
    });
    unawaited(_refreshProvince(group));
  }

  /// 后台增量刷新某省;成功后静默刷新市列表,失败仅在本地空时提示。
  Future<void> _refreshProvince(String province) async {
    try {
      await _repo.refreshProvince(province);
      if (!mounted || _selected != province) return;
      final cities = await _repo.listCities(province);
      if (!mounted || _selected != province) return;
      setState(() => _cities = cities);
    } on Exception {
      if (!mounted || _selected != province) return;
      if (_cities.isEmpty) {
        setState(() => _contentError = '目录同步失败,请检查 SFID 连接后重试');
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // 标题与治理 tab"治理机构"对称。
        const Padding(
          padding: EdgeInsets.fromLTRB(16, 16, 16, 12),
          child: Text(
            '公权机构',
            style: TextStyle(
              fontSize: 22,
              fontWeight: FontWeight.w700,
              color: AppTheme.textPrimary,
            ),
          ),
        ),
        Expanded(
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _ProvinceRail(
                provinces: _provinces,
                selected: _selected,
                onSelect: _selectGroup,
              ),
              Expanded(child: _buildContent()),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildContent() {
    if (_contentLoading) {
      return const Center(child: CircularProgressIndicator(strokeWidth: 2));
    }
    if (_selected == _kFollowGroup) {
      return _buildFollowList();
    }
    return _buildCityList();
  }

  Widget _buildFollowList() {
    if (_subscribed.isEmpty) {
      return _emptyHint(
        icon: Icons.bookmark_border,
        title: '还没有关注的公权机构',
        subtitle: '进入机构详情页,点右上角订阅即可加入关注',
      );
    }
    return ListView.separated(
      padding: const EdgeInsets.symmetric(vertical: 8),
      itemCount: _subscribed.length,
      separatorBuilder: (_, __) =>
          const Divider(height: 1, color: AppTheme.divider),
      itemBuilder: (context, i) {
        final inst = _subscribed[i];
        return _InstitutionTile(
          title: inst.shortName?.isNotEmpty == true
              ? inst.shortName!
              : inst.institutionName,
          subtitle: '${provinceDisplayName(inst.province)} · ${inst.city}',
          onTap: () => _openDetail(inst.sfidNumber),
        );
      },
    );
  }

  Widget _buildCityList() {
    if (_cities.isEmpty) {
      return _emptyHint(
        icon: _contentError != null
            ? Icons.cloud_off_outlined
            : Icons.location_city_outlined,
        title: _contentError ?? '$_selected 暂无可显示的公权机构',
        subtitle: _contentError != null ? '稍后重试,或先在桌面端生成数据包' : '目录尚未同步或该省无机构数据',
      );
    }
    return ListView.separated(
      padding: const EdgeInsets.symmetric(vertical: 8),
      itemCount: _cities.length,
      separatorBuilder: (_, __) =>
          const Divider(height: 1, color: AppTheme.divider),
      itemBuilder: (context, i) {
        final city = _cities[i];
        return _InstitutionTile(
          title: city,
          subtitle: '查看 $city 公权机构',
          trailing: const Icon(Icons.chevron_right,
              color: AppTheme.textTertiary, size: 20),
          onTap: () => _openCity(_selected, city),
        );
      },
    );
  }

  void _openCity(String province, String city) {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => CityInstitutionListPage(
          province: province,
          city: city,
          repository: _repo,
        ),
      ),
    );
  }

  void _openDetail(String sfidNumber) {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => PublicInstitutionDetailPage(
          sfidNumber: sfidNumber,
          repository: _repo,
        ),
      ),
    );
  }

  Widget _emptyHint({
    required IconData icon,
    required String title,
    required String subtitle,
  }) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 44, color: AppTheme.textTertiary),
            const SizedBox(height: 12),
            Text(title,
                textAlign: TextAlign.center,
                style: const TextStyle(
                    fontSize: 15,
                    fontWeight: FontWeight.w600,
                    color: AppTheme.textSecondary)),
            const SizedBox(height: 6),
            Text(subtitle,
                textAlign: TextAlign.center,
                style: const TextStyle(
                    fontSize: 12.5, color: AppTheme.textTertiary)),
          ],
        ),
      ),
    );
  }
}

/// 左侧竖向导航:关注**钉顶固定**,下方省份可上下滚动。无竖线、无独立面板,
/// 直接在页面背景上;选中态用文字加粗变色 + 轻量圆角 pill(方案A)。
class _ProvinceRail extends StatelessWidget {
  const _ProvinceRail({
    required this.provinces,
    required this.selected,
    required this.onSelect,
  });

  final List<String> provinces;
  final String selected;
  final ValueChanged<String> onSelect;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 88,
      child: Column(
        children: [
          // 关注:钉顶固定,不随省份滚动。
          Padding(
            padding: const EdgeInsets.fromLTRB(6, 4, 6, 2),
            child: _railItem(_kFollowGroup, selected == _kFollowGroup),
          ),
          const Divider(height: 9, indent: 14, endIndent: 14),
          Expanded(
            child: ListView.builder(
              padding: const EdgeInsets.fromLTRB(6, 0, 6, 12),
              itemCount: provinces.length,
              itemBuilder: (context, i) {
                final name = provinces[i];
                return Padding(
                  padding: const EdgeInsets.only(bottom: 4),
                  child: _railItem(name, name == selected),
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Widget _railItem(String name, bool active) {
    return InkWell(
      borderRadius: BorderRadius.circular(10),
      onTap: () => onSelect(name),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 150),
        // width 撑满:让"关注"(在 Column 里)与省份(在 ListView 里)选中背景同宽。
        width: double.infinity,
        padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 6),
        decoration: BoxDecoration(
          color: active ? AppTheme.surfaceElevated : Colors.transparent,
          borderRadius: BorderRadius.circular(10),
        ),
        child: Text(
          // 展示去"省";onSelect 仍传全名 name 用于查询。
          name == _kFollowGroup ? name : provinceDisplayName(name),
          textAlign: TextAlign.center,
          style: TextStyle(
            fontSize: active ? 18 : 16,
            fontWeight: active ? FontWeight.w700 : FontWeight.w500,
            color: active ? AppTheme.primary : AppTheme.textSecondary,
          ),
        ),
      ),
    );
  }
}

/// 机构/市通用行。
class _InstitutionTile extends StatelessWidget {
  const _InstitutionTile({
    required this.title,
    required this.subtitle,
    required this.onTap,
    this.trailing,
  });

  final String title;
  final String subtitle;
  final VoidCallback onTap;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      onTap: onTap,
      title: Text(
        title,
        style: const TextStyle(
          fontSize: 15,
          fontWeight: FontWeight.w600,
          color: AppTheme.textPrimary,
        ),
      ),
      subtitle: Text(
        subtitle,
        style: const TextStyle(fontSize: 12.5, color: AppTheme.textTertiary),
      ),
      trailing: trailing,
    );
  }
}
