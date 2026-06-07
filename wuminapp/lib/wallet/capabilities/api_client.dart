import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:http/http.dart' as http;
import 'package:wuminapp_mobile/sfid_api_config.dart';

class HealthStatus {
  const HealthStatus({
    required this.service,
    required this.version,
    required this.status,
  });

  final String service;
  final String version;
  final String status;
}

class AdminCatalogEntryResponse {
  const AdminCatalogEntryResponse({
    required this.pubkeyHex,
    required this.roleName,
    required this.institutionName,
    required this.org,
  });

  final String pubkeyHex;
  final String roleName;
  final String institutionName;
  final String org;
}

class AdminCatalogResponse {
  const AdminCatalogResponse({
    required this.source,
    required this.updatedAt,
    required this.institutionCount,
    required this.adminCount,
    required this.entries,
  });

  final String source;
  final int updatedAt;
  final int institutionCount;
  final int adminCount;
  final List<AdminCatalogEntryResponse> entries;
}

/// SFID 人口快照响应。
///
/// ADR-008 step3:凭证双层匹配。SFID 后端在签发人口快照时下发
/// (province, signer_admin_pubkey) — 链端 RuntimePopulationSnapshotVerifier
/// 走 `sheng_signing_pubkey_for_admin(province, admin_pubkey)` 双映射查派生
/// 公钥;无对应记录直接拒签。wuminapp 在线端透传到 chain extrinsic,不二次校验。
class PopulationSnapshotResponse {
  const PopulationSnapshotResponse({
    required this.genesisHash,
    required this.eligibleTotal,
    required this.snapshotNonce,
    required this.signature,
    required this.who,
    required this.province,
    required this.signerAdminPubkey,
  });

  final String genesisHash;
  final int eligibleTotal;

  /// 快照 nonce（UTF-8 字符串，直接作为 BoundedVec<u8> 提交）。
  final String snapshotNonce;

  /// sr25519 签名（hex 编码，提交时需解码为原始字节）。
  final String signature;

  /// 归一化后的账户公钥 hex。
  final String who;

  /// 签发 admin 所属省份(UTF-8 中文,如 "安徽省")。
  /// SFID 后端按登录省管理员路由下发,链端 SCALE 末尾必填字段。
  final String province;

  /// 签发本凭证的省管理员 admin pubkey(0x 小写 hex,32 字节)。
  final String signerAdminPubkey;
}

/// SFID 机构注册凭证。
///
/// 中文注释:这些字段只用于链端验签和防重放,不得混入 subject_property/sub_type/parent_sfid_number
/// 等业务分类字段。
class InstitutionRegistrationCredential {
  const InstitutionRegistrationCredential({
    required this.genesisHash,
    required this.registerNonce,
    required this.province,
    required this.signerAdminPubkey,
    required this.signature,
  });

  final String genesisHash;
  final String registerNonce;
  final String province;
  final String signerAdminPubkey;
  final String signature;
}

/// SFID 机构链端注册信息。
class InstitutionRegistrationInfoResponse {
  const InstitutionRegistrationInfoResponse({
    required this.sfidNumber,
    required this.institutionName,
    required this.accountNames,
    required this.credential,
  });

  final String sfidNumber;
  final String institutionName;
  final List<String> accountNames;
  final InstitutionRegistrationCredential credential;
}

class ApiClient {
  ApiClient() : _baseUrl = SfidApiConfig.defaultBaseUrl;

  final String _baseUrl;

  Map<String, String> _headers({
    bool includeContentType = false,
  }) {
    final out = <String, String>{};
    if (includeContentType) {
      out['Content-Type'] = 'application/json';
    }
    return out;
  }

  Future<HealthStatus> fetchHealth() async {
    final uri = Uri.parse('$_baseUrl/api/v1/health');
    final response = await http.get(uri, headers: _headers());
    if (response.statusCode != 200) {
      throw Exception('health check failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final data = payload['data'] as Map<String, dynamic>;
    return HealthStatus(
      service: data['service'] as String? ?? '-',
      version: data['version'] as String? ?? '-',
      status: data['status'] as String? ?? '-',
    );
  }

  String _normalizePubkeyHex(String value) {
    final trimmed = value.trim();
    if (trimmed.isEmpty) {
      throw Exception('pubkey is empty');
    }
    return trimmed.startsWith('0x') ? trimmed : '0x$trimmed';
  }

  Future<AdminCatalogResponse> fetchAdminCatalog() async {
    final uri = Uri.parse('$_baseUrl/api/v1/admins/catalog');
    final response = await http.get(
      uri,
      headers: _headers(),
    );
    if (response.statusCode != 200) {
      throw Exception('admin catalog failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception('admin catalog rejected: code=$code message=$message');
    }

    final data = payload['data'];
    if (data is! Map<String, dynamic>) {
      throw Exception('admin catalog invalid response: missing data');
    }
    final rawEntries = data['entries'];
    if (rawEntries is! List) {
      throw Exception('admin catalog invalid response: missing entries');
    }
    final entries = <AdminCatalogEntryResponse>[];
    for (final item in rawEntries) {
      if (item is! Map) {
        continue;
      }
      final m = item.map((k, v) => MapEntry(k.toString(), v));
      final pubkey = (m['pubkey_hex']?.toString() ?? '').trim().toLowerCase();
      final role = (m['role_name']?.toString() ?? '').trim();
      final institutionName = (m['institution_name']?.toString() ?? '').trim();
      if (pubkey.isEmpty || role.isEmpty || institutionName.isEmpty) {
        continue;
      }
      entries.add(
        AdminCatalogEntryResponse(
          pubkeyHex: pubkey.startsWith('0x') ? pubkey.substring(2) : pubkey,
          roleName: role,
          institutionName: institutionName,
          org: (m['org']?.toString() ?? 'unknown').trim().toLowerCase(),
        ),
      );
    }

    return AdminCatalogResponse(
      source: data['source']?.toString() ?? 'chain',
      updatedAt: (data['updated_at'] as num?)?.toInt() ?? 0,
      institutionCount: (data['institution_count'] as num?)?.toInt() ?? 0,
      adminCount: (data['admin_count'] as num?)?.toInt() ?? 0,
      entries: entries,
    );
  }

  /// 获取公民人口快照（eligible_total + nonce + signature）。
  ///
  /// 用于联合投票提案创建时附带人口证明。
  Future<PopulationSnapshotResponse> fetchPopulationSnapshot(
      String accountPubkeyHex) async {
    final normalized = _normalizePubkeyHex(accountPubkeyHex);
    final uri = Uri.parse(
        '$_baseUrl/api/v1/app/voters/count?account_pubkey=$normalized');
    http.Response response;
    try {
      response = await http.get(
        uri,
        headers: _headers(),
      );
    } on SocketException catch (_) {
      if (Platform.isAndroid || Platform.isIOS) {
        throw Exception(SfidApiConfig.connectionErrorMessage(_baseUrl));
      }
      rethrow;
    }
    if (response.statusCode != 200) {
      throw Exception('population snapshot failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception(
        'population snapshot rejected: code=$code message=$message',
      );
    }

    final data = payload['data'];
    if (data is! Map<String, dynamic>) {
      throw Exception('population snapshot invalid response: missing data');
    }

    final province = (data['province']?.toString() ?? '').trim();
    final signerAdminPubkeyRaw =
        (data['signer_admin_pubkey']?.toString() ?? '').trim();
    if (province.isEmpty) {
      throw Exception('population snapshot 缺少 province 字段(ADR-008 step3 必填)');
    }
    if (signerAdminPubkeyRaw.isEmpty) {
      throw Exception(
          'population snapshot 缺少 signer_admin_pubkey 字段(ADR-008 step3 必填)');
    }
    // 机读字段统一为 0x 小写 hex。
    final signerAdminPubkey = signerAdminPubkeyRaw.startsWith('0x')
        ? signerAdminPubkeyRaw.toLowerCase()
        : '0x${signerAdminPubkeyRaw.toLowerCase()}';

    return PopulationSnapshotResponse(
      eligibleTotal: (data['eligible_total'] as num?)?.toInt() ?? 0,
      snapshotNonce: (data['snapshot_nonce']?.toString() ?? '').trim(),
      genesisHash: (data['genesis_hash']?.toString() ?? '').trim(),
      signature: (data['signature']?.toString() ?? '').trim(),
      who: (data['who']?.toString() ?? '').trim(),
      province: province,
      signerAdminPubkey: signerAdminPubkey,
    );
  }

  /// 查询机构下所有多签账户。
  ///
  /// 调用 SFID 后端 `GET /api/v1/app/institutions/:sfid_number/accounts`，
  /// 返回机构名称 + 账户列表（account_name / duoqian_address / chain_status）。
  Future<InstitutionAccountsResponse> fetchInstitutionAccounts(
      String sfidNumber) async {
    final trimmed = sfidNumber.trim();
    if (trimmed.isEmpty) {
      throw Exception('SFID ID 不能为空');
    }
    final uri = Uri.parse(
        '$_baseUrl/api/v1/app/institutions/${Uri.encodeComponent(trimmed)}/accounts');
    http.Response response;
    try {
      response = await http
          .get(uri, headers: _headers())
          .timeout(const Duration(seconds: 15));
    } on TimeoutException catch (_) {
      throw Exception('查询超时，请检查网络连接');
    } on SocketException catch (_) {
      if (Platform.isAndroid || Platform.isIOS) {
        throw Exception(SfidApiConfig.connectionErrorMessage(_baseUrl));
      }
      rethrow;
    }
    if (response.statusCode == 404) {
      throw Exception('未找到该 SFID 机构');
    }
    if (response.statusCode != 200) {
      throw Exception('查询机构账户失败: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception('查询机构账户被拒: code=$code message=$message');
    }

    final data = payload['data'];
    if (data is! Map<String, dynamic>) {
      throw Exception('查询机构账户: 响应缺少 data 字段');
    }

    final rawAccounts = data['accounts'];
    final accounts = <InstitutionAccountEntry>[];
    if (rawAccounts is List) {
      for (final item in rawAccounts) {
        if (item is! Map) continue;
        final m = item.map((k, v) => MapEntry(k.toString(), v));
        accounts.add(InstitutionAccountEntry(
          accountName: (m['account_name']?.toString() ?? '').trim(),
          duoqianAddress: m['duoqian_address']?.toString(),
          // 中文注释:SFID 后端公开接口返回 SCREAMING_SNAKE_CASE；
          // 这里兼容旧口径 Pending/Confirmed/Failed，统一折叠成同一套状态。
          chainStatus: InstitutionAccountEntry.normalizeChainStatus(
            m['chain_status']?.toString(),
          ),
        ));
      }
    }

    return InstitutionAccountsResponse(
      sfidNumber: (data['sfid_number']?.toString() ?? trimmed).trim(),
      institutionName: (data['institution_name']?.toString() ?? '').trim(),
      accounts: accounts,
    );
  }

  /// 查询机构链端注册信息。
  ///
  /// 调用 SFID 后端 `GET /api/v1/app/institutions/:sfid_number/registration-info`。
  /// 该接口是 `OrganizationManage.propose_create_institution` 的唯一凭证来源。
  Future<InstitutionRegistrationInfoResponse> fetchInstitutionRegistrationInfo(
      String sfidNumber) async {
    final trimmed = sfidNumber.trim();
    if (trimmed.isEmpty) {
      throw Exception('SFID ID 不能为空');
    }
    final uri = Uri.parse(
        '$_baseUrl/api/v1/app/institutions/${Uri.encodeComponent(trimmed)}/registration-info');
    http.Response response;
    try {
      response = await http
          .get(uri, headers: _headers())
          .timeout(const Duration(seconds: 15));
    } on TimeoutException catch (_) {
      throw Exception('查询注册凭证超时，请检查网络连接');
    } on SocketException catch (_) {
      if (Platform.isAndroid || Platform.isIOS) {
        throw Exception(SfidApiConfig.connectionErrorMessage(_baseUrl));
      }
      rethrow;
    }
    if (response.statusCode == 404) {
      throw Exception('未找到该 SFID 机构');
    }
    if (response.statusCode != 200) {
      throw Exception('查询机构注册凭证失败: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception('查询机构注册凭证被拒: code=$code message=$message');
    }

    final data = payload['data'];
    if (data is! Map<String, dynamic>) {
      throw Exception('机构注册凭证响应缺少 data 字段');
    }
    final credential = data['credential'];
    if (credential is! Map) {
      throw Exception('机构注册凭证响应缺少 credential 字段');
    }
    final credentialMap = credential.map((k, v) => MapEntry(k.toString(), v));

    final rawAccountNames = data['account_names'];
    if (rawAccountNames is! List) {
      throw Exception('机构注册凭证响应缺少 account_names 字段');
    }
    final accountNames = rawAccountNames
        .map((v) => v.toString().trim())
        .where((v) => v.isNotEmpty)
        .toList(growable: false);
    if (accountNames.isEmpty) {
      throw Exception('机构注册凭证 account_names 为空');
    }

    final institutionName = (data['institution_name']?.toString() ?? '').trim();
    final registerNonce =
        (credentialMap['register_nonce']?.toString() ?? '').trim();
    final province = (credentialMap['province']?.toString() ?? '').trim();
    final signerAdminPubkeyRaw =
        (credentialMap['signer_admin_pubkey']?.toString() ?? '').trim();
    final signature = (credentialMap['signature']?.toString() ?? '').trim();
    if (institutionName.isEmpty) {
      throw Exception('机构注册凭证 institution_name 为空');
    }
    if (registerNonce.isEmpty) {
      throw Exception('机构注册凭证 register_nonce 为空');
    }
    if (province.isEmpty) {
      throw Exception('机构注册凭证 province 为空');
    }
    if (signature.isEmpty) {
      throw Exception('机构注册凭证 signature 为空');
    }

    final signerAdminPubkey =
        _normalizePubkeyHex(signerAdminPubkeyRaw).toLowerCase();
    final signerClean = signerAdminPubkey.substring(2);
    if (signerClean.length != 64 ||
        !RegExp(r'^[0-9a-f]+$').hasMatch(signerClean)) {
      throw Exception('机构注册凭证 signer_admin_pubkey 必须为 32 字节 hex');
    }
    return InstitutionRegistrationInfoResponse(
      sfidNumber: (data['sfid_number']?.toString() ?? trimmed).trim(),
      institutionName: institutionName,
      accountNames: accountNames,
      credential: InstitutionRegistrationCredential(
        genesisHash: (credentialMap['genesis_hash']?.toString() ?? '').trim(),
        registerNonce: registerNonce,
        province: province,
        signerAdminPubkey: signerAdminPubkey,
        signature: signature.startsWith('0x')
            ? signature.toLowerCase()
            : '0x${signature.toLowerCase()}',
      ),
    );
  }

  /// 从 SFID 获取公民投票凭证。
  ///
  /// 公民投票时，App 先从 SFID 获取投票资格证明（binding_id + vote_nonce + signature），
  /// 再将凭证提交到链上。
  Future<VoteCredentialResponse> fetchVoteCredential(
      String accountPubkeyHex, int proposalId) async {
    final normalized = _normalizePubkeyHex(accountPubkeyHex);
    final uri = Uri.parse('$_baseUrl/api/v1/app/vote/credential');
    final body = jsonEncode({
      'account_pubkey': normalized,
      'proposal_id': proposalId,
    });
    http.Response response;
    try {
      response = await http.post(
        uri,
        headers: _headers(includeContentType: true),
        body: body,
      );
    } on SocketException catch (_) {
      if (Platform.isAndroid || Platform.isIOS) {
        throw Exception(SfidApiConfig.connectionErrorMessage(_baseUrl));
      }
      rethrow;
    }
    if (response.statusCode != 200) {
      throw Exception('vote credential failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception(
        'vote credential rejected: code=$code message=$message',
      );
    }

    final data = payload['data'];
    if (data is! Map<String, dynamic>) {
      throw Exception('vote credential invalid response: missing data');
    }

    return VoteCredentialResponse(
      genesisHash: (data['genesis_hash']?.toString() ?? '').trim(),
      who: (data['who']?.toString() ?? '').trim(),
      bindingId: (data['binding_id']?.toString() ?? '').trim(),
      proposalId: (data['proposal_id'] as num?)?.toInt() ?? proposalId,
      voteNonce: (data['vote_nonce']?.toString() ?? '').trim(),
      signature: (data['signature']?.toString() ?? '').trim(),
    );
  }
}

class VoteCredentialResponse {
  final String genesisHash;
  final String who;
  final String bindingId;
  final int proposalId;
  final String voteNonce;
  final String signature;

  VoteCredentialResponse({
    required this.genesisHash,
    required this.who,
    required this.bindingId,
    required this.proposalId,
    required this.voteNonce,
    required this.signature,
  });
}

/// 机构下单个多签账户条目。
class InstitutionAccountEntry {
  const InstitutionAccountEntry({
    required this.accountName,
    this.duoqianAddress,
    required this.chainStatus,
  });

  /// 账户名称（链上 name 字段）。
  final String accountName;

  /// 链上派生的多签地址（hex，上链成功后才有值）。
  final String? duoqianAddress;

  /// 链上状态：`INACTIVE` / `PENDING` / `REGISTERED` / `FAILED`。
  final String chainStatus;

  bool get isRegistered => chainStatus == 'REGISTERED';

  static String normalizeChainStatus(String? raw) {
    final status = raw?.trim();
    switch (status) {
      case 'INACTIVE':
      case 'NOT_ON_CHAIN':
        return 'INACTIVE';
      case 'PENDING':
      case 'PENDING_ON_CHAIN':
        return 'PENDING';
      case 'REGISTERED':
      case 'ACTIVE_ON_CHAIN':
        return 'REGISTERED';
      case 'FAILED':
      case 'REVOKED_ON_CHAIN':
        return 'FAILED';
      case 'Pending':
        return 'PENDING';
      case 'Confirmed':
        return 'REGISTERED';
      case 'Failed':
        return 'FAILED';
      default:
        return 'PENDING';
    }
  }
}

/// 机构账户列表响应。
class InstitutionAccountsResponse {
  const InstitutionAccountsResponse({
    required this.sfidNumber,
    required this.institutionName,
    required this.accounts,
  });

  final String sfidNumber;
  final String institutionName;
  final List<InstitutionAccountEntry> accounts;
}
