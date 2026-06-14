import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/citizen/public/data/public_institution_repository.dart';
import 'package:wuminapp_mobile/citizen/public/public_institution_detail_page.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';

/// 某市公权机构列表(ADR-018 §九 卡B)。
///
/// 中文注释:读本地 repo,展示该市全部公权机构简要信息;点进详情页(卡C)。
class CityInstitutionListPage extends StatefulWidget {
  const CityInstitutionListPage({
    super.key,
    required this.province,
    required this.city,
    required this.repository,
  });

  final String province;
  final String city;
  final PublicInstitutionRepository repository;

  @override
  State<CityInstitutionListPage> createState() =>
      _CityInstitutionListPageState();
}

class _CityInstitutionListPageState extends State<CityInstitutionListPage> {
  List<PublicInstitutionEntity> _items = const [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final items = await widget.repository
        .listInstitutionsByCity(widget.province, widget.city);
    if (!mounted) return;
    setState(() {
      _items = items;
      _loading = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        title: Text('${widget.city}公权机构'),
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
    if (_items.isEmpty) {
      return Center(
        child: Text(
          '${widget.city}暂无公权机构数据',
          style: const TextStyle(color: AppTheme.textTertiary),
        ),
      );
    }
    return ListView.separated(
      padding: const EdgeInsets.symmetric(vertical: 8),
      itemCount: _items.length,
      separatorBuilder: (_, __) =>
          const Divider(height: 1, color: AppTheme.divider),
      itemBuilder: (context, i) {
        final inst = _items[i];
        final title = inst.shortName?.isNotEmpty == true
            ? inst.shortName!
            : inst.institutionName;
        return ListTile(
          title: Text(
            title,
            style: const TextStyle(
              fontSize: 15,
              fontWeight: FontWeight.w600,
              color: AppTheme.textPrimary,
            ),
          ),
          subtitle: Text(
            _typeLabel(inst.institutionCode),
            style:
                const TextStyle(fontSize: 12.5, color: AppTheme.textTertiary),
          ),
          trailing: const Icon(Icons.chevron_right,
              color: AppTheme.textTertiary, size: 20),
          onTap: () => Navigator.of(context).push(
            MaterialPageRoute<void>(
              builder: (_) => PublicInstitutionDetailPage(
                sfidNumber: inst.sfidNumber,
                repository: widget.repository,
              ),
            ),
          ),
        );
      },
    );
  }

  String _typeLabel(String code) {
    switch (code) {
      case 'ZF':
        return '政府';
      case 'LF':
        return '立法';
      case 'SF':
        return '司法';
      case 'JC':
        return '检察';
      case 'JY':
        return '教育';
      default:
        return '公权机构';
    }
  }
}
