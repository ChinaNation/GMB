import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:http/http.dart' as http;
import 'package:citizenapp/cid_api_config.dart';

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
    required this.adminGroupName,
    required this.cidFullName,
    required this.org,
  });

  final String pubkeyHex;
  final String adminGroupName;
  final String cidFullName;
  final String org;
}

class AdminCatalogResponse {
  const AdminCatalogResponse({
    required this.source,
    required this.updatedAt,
    required this.institutionCount,
    required this.adminsLen,
    required this.entries,
  });

  final String source;
  final int updatedAt;
  final int institutionCount;
  final int adminsLen;
  final List<AdminCatalogEntryResponse> entries;
}

/// CID 人口快照响应。
///
/// CID 人口快照凭证统一使用签发机构模型。
///
/// 链端按 issuer_main_account 读取 admins-change 的 admins 真源,确认
/// signer_pubkey 属于该签发机构管理员后再验签。
class PopulationSnapshotResponse {
  const PopulationSnapshotResponse({
    required this.genesisHash,
    required this.eligibleTotal,
    required this.snapshotNonce,
    required this.signature,
    required this.who,
    required this.issuerCidNumber,
    required this.issuerMainAccount,
    required this.signerPubkey,
    required this.scopeProvinceName,
    required this.scopeCityName,
  });

  final String genesisHash;
  final int eligibleTotal;

  /// 快照 nonce（UTF-8 字符串，直接作为 BoundedVec<u8> 提交）。
  final String snapshotNonce;

  /// sr25519 签名（hex 编码，提交时需解码为原始字节）。
  final String signature;

  /// 归一化后的账户公钥 hex。
  final String who;

  final String issuerCidNumber;
  final String issuerMainAccount;
  final String signerPubkey;
  final String scopeProvinceName;
  final String scopeCityName;
}

/// CID 机构注册凭证。
///
/// 中文注释:这些字段只用于链端验签和防重放,不得混入 subject_property/sub_type/parent_cid_number
/// 等业务分类字段。
class InstitutionRegistrationCredential {
  const InstitutionRegistrationCredential({
    required this.genesisHash,
    required this.registerNonce,
    required this.issuerCidNumber,
    required this.issuerMainAccount,
    required this.signerPubkey,
    required this.scopeProvinceName,
    required this.scopeCityName,
    required this.signature,
  });

  final String genesisHash;
  final String registerNonce;
  final String issuerCidNumber;
  final String issuerMainAccount;
  final String signerPubkey;
  final String scopeProvinceName;
  final String scopeCityName;
  final String signature;
}

/// CID 机构链端注册信息。
class InstitutionRegistrationInfoResponse {
  const InstitutionRegistrationInfoResponse({
    required this.cidNumber,
    required this.cidFullName,
    required this.cidShortName,
    required this.institutionCode,
    required this.accountNames,
    required this.credential,
  });

  final String cidNumber;
  final String cidFullName;
  final String cidShortName;
  final String institutionCode;
  final List<String> accountNames;
  final InstitutionRegistrationCredential credential;
}

class ApiClient {
  ApiClient() : _baseUrl = CidApiConfig.defaultBaseUrl;

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
      final adminGroup = (m['admin_group_name']?.toString() ?? '').trim();
      final cidFullName = (m['cid_full_name']?.toString() ?? '').trim();
      if (pubkey.isEmpty || adminGroup.isEmpty || cidFullName.isEmpty) {
        continue;
      }
      entries.add(
        AdminCatalogEntryResponse(
          pubkeyHex: pubkey.startsWith('0x') ? pubkey.substring(2) : pubkey,
          adminGroupName: adminGroup,
          cidFullName: cidFullName,
          org: (m['org']?.toString() ?? 'unknown').trim().toLowerCase(),
        ),
      );
    }

    return AdminCatalogResponse(
      source: data['source']?.toString() ?? 'chain',
      updatedAt: (data['updated_at'] as num?)?.toInt() ?? 0,
      institutionCount: (data['institution_count'] as num?)?.toInt() ?? 0,
      adminsLen: (data['admins_len'] as num?)?.toInt() ?? 0,
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
        throw Exception(CidApiConfig.connectionErrorMessage(_baseUrl));
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

    final issuerCidNumber =
        (data['issuer_cid_number']?.toString() ?? '').trim();
    final issuerMainAccountRaw =
        (data['issuer_main_account']?.toString() ?? '').trim();
    final signerPubkeyRaw = (data['signer_pubkey']?.toString() ?? '').trim();
    final scopeProvinceName =
        (data['scope_province_name']?.toString() ?? '').trim();
    final scopeCityName = (data['scope_city_name']?.toString() ?? '').trim();
    if (issuerCidNumber.isEmpty) {
      throw Exception('population snapshot 缺少 issuer_cid_number 字段');
    }
    if (issuerMainAccountRaw.isEmpty) {
      throw Exception('population snapshot 缺少 issuer_main_account 字段');
    }
    if (signerPubkeyRaw.isEmpty) {
      throw Exception('population snapshot 缺少 signer_pubkey 字段');
    }
    if (scopeProvinceName.isEmpty) {
      throw Exception('population snapshot 缺少 scope_province_name 字段');
    }

    return PopulationSnapshotResponse(
      eligibleTotal: (data['eligible_total'] as num?)?.toInt() ?? 0,
      snapshotNonce: (data['snapshot_nonce']?.toString() ?? '').trim(),
      genesisHash: (data['genesis_hash']?.toString() ?? '').trim(),
      signature: (data['signature']?.toString() ?? '').trim(),
      who: (data['who']?.toString() ?? '').trim(),
      issuerCidNumber: issuerCidNumber,
      issuerMainAccount:
          _normalizePubkeyHex(issuerMainAccountRaw).toLowerCase(),
      signerPubkey: _normalizePubkeyHex(signerPubkeyRaw).toLowerCase(),
      scopeProvinceName: scopeProvinceName,
      scopeCityName: scopeCityName,
    );
  }

  /// 查询机构下所有多签账户。
  ///
  /// 调用 OnChina 后端 `GET /api/v1/app/institutions/:cid_number/accounts`，
  /// 返回机构全称/简称 + 账户列表（account_name / account / chain_status）。
  Future<InstitutionAccountsResponse> fetchInstitutionAccounts(
      String cidNumber) async {
    final trimmed = cidNumber.trim();
    if (trimmed.isEmpty) {
      throw Exception('CID ID 不能为空');
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
        throw Exception(CidApiConfig.connectionErrorMessage(_baseUrl));
      }
      rethrow;
    }
    if (response.statusCode == 404) {
      throw Exception('未找到该 CID 机构');
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
          account: m['account']?.toString(),
          // 中文注释:OnChina 后端公开接口返回 SCREAMING_SNAKE_CASE；
          // 这里兼容旧口径 Pending/Confirmed/Failed，统一折叠成同一套状态。
          chainStatus: InstitutionAccountEntry.normalizeChainStatus(
            m['chain_status']?.toString(),
          ),
        ));
      }
    }

    return InstitutionAccountsResponse(
      cidNumber: (data['cid_number']?.toString() ?? trimmed).trim(),
      cidFullName: (data['cid_full_name']?.toString() ?? '').trim(),
      cidShortName: (data['cid_short_name']?.toString() ?? '').trim(),
      accounts: accounts,
    );
  }

  /// 查询机构链端注册信息。
  ///
  /// 调用 OnChina 后端 `GET /api/v1/app/institutions/:cid_number/registration-info`。
  /// 该接口是 `PublicManage/PrivateManage.propose_create_{public,private}_institution` 的唯一凭证来源。
  Future<InstitutionRegistrationInfoResponse> fetchInstitutionRegistrationInfo(
      String cidNumber) async {
    final trimmed = cidNumber.trim();
    if (trimmed.isEmpty) {
      throw Exception('CID ID 不能为空');
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
        throw Exception(CidApiConfig.connectionErrorMessage(_baseUrl));
      }
      rethrow;
    }
    if (response.statusCode == 404) {
      throw Exception('未找到该 CID 机构');
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

    final cidFullName = (data['cid_full_name']?.toString() ?? '').trim();
    final cidShortName = (data['cid_short_name']?.toString() ?? '').trim();
    final institutionCode = (data['institution_code']?.toString() ?? '').trim();
    final registerNonce =
        (credentialMap['register_nonce']?.toString() ?? '').trim();
    final issuerCidNumber =
        (credentialMap['issuer_cid_number']?.toString() ?? '').trim();
    final issuerMainAccountRaw =
        (credentialMap['issuer_main_account']?.toString() ?? '').trim();
    final signerPubkeyRaw =
        (credentialMap['signer_pubkey']?.toString() ?? '').trim();
    final scopeProvinceName =
        (credentialMap['scope_province_name']?.toString() ?? '').trim();
    final scopeCityName =
        (credentialMap['scope_city_name']?.toString() ?? '').trim();
    final signature = (credentialMap['signature']?.toString() ?? '').trim();
    if (cidFullName.isEmpty) {
      throw Exception('机构注册凭证 cid_full_name 为空');
    }
    if (cidShortName.isEmpty) {
      throw Exception('机构注册凭证 cid_short_name 为空');
    }
    if (institutionCode.isEmpty) {
      throw Exception('机构注册凭证 institution_code 为空');
    }
    if (registerNonce.isEmpty) {
      throw Exception('机构注册凭证 register_nonce 为空');
    }
    if (issuerCidNumber.isEmpty) {
      throw Exception('机构注册凭证 issuer_cid_number 为空');
    }
    if (issuerMainAccountRaw.isEmpty) {
      throw Exception('机构注册凭证 issuer_main_account 为空');
    }
    if (signerPubkeyRaw.isEmpty) {
      throw Exception('机构注册凭证 signer_pubkey 为空');
    }
    if (scopeProvinceName.isEmpty) {
      throw Exception('机构注册凭证 scope_province_name 为空');
    }
    if (signature.isEmpty) {
      throw Exception('机构注册凭证 signature 为空');
    }

    final issuerMainAccount =
        _normalizePubkeyHex(issuerMainAccountRaw).toLowerCase();
    final signerPubkey = _normalizePubkeyHex(signerPubkeyRaw).toLowerCase();
    for (final entry in {
      'issuer_main_account': issuerMainAccount,
      'signer_pubkey': signerPubkey,
    }.entries) {
      final clean = entry.value.substring(2);
      if (clean.length != 64 || !RegExp(r'^[0-9a-f]+$').hasMatch(clean)) {
        throw Exception('机构注册凭证 ${entry.key} 必须为 32 字节 hex');
      }
    }
    return InstitutionRegistrationInfoResponse(
      cidNumber: (data['cid_number']?.toString() ?? trimmed).trim(),
      cidFullName: cidFullName,
      cidShortName: cidShortName,
      institutionCode: institutionCode,
      accountNames: accountNames,
      credential: InstitutionRegistrationCredential(
        genesisHash: (credentialMap['genesis_hash']?.toString() ?? '').trim(),
        registerNonce: registerNonce,
        issuerCidNumber: issuerCidNumber,
        issuerMainAccount: issuerMainAccount,
        signerPubkey: signerPubkey,
        scopeProvinceName: scopeProvinceName,
        scopeCityName: scopeCityName,
        signature: signature.startsWith('0x')
            ? signature.toLowerCase()
            : '0x${signature.toLowerCase()}',
      ),
    );
  }

  /// 从 CID 获取公民投票凭证。
  ///
  /// 公民投票时，App 先从 CID 获取投票资格证明（binding_id + vote_nonce + signature），
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
        throw Exception(CidApiConfig.connectionErrorMessage(_baseUrl));
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
      issuerCidNumber: (data['issuer_cid_number']?.toString() ?? '').trim(),
      issuerMainAccount: _normalizePubkeyHex(
          (data['issuer_main_account']?.toString() ?? '').trim()),
      signerPubkey:
          _normalizePubkeyHex((data['signer_pubkey']?.toString() ?? '').trim()),
      scopeProvinceName: (data['scope_province_name']?.toString() ?? '').trim(),
      scopeCityName: (data['scope_city_name']?.toString() ?? '').trim(),
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
  final String issuerCidNumber;
  final String issuerMainAccount;
  final String signerPubkey;
  final String scopeProvinceName;
  final String scopeCityName;
  final String signature;

  VoteCredentialResponse({
    required this.genesisHash,
    required this.who,
    required this.bindingId,
    required this.proposalId,
    required this.voteNonce,
    required this.issuerCidNumber,
    required this.issuerMainAccount,
    required this.signerPubkey,
    required this.scopeProvinceName,
    required this.scopeCityName,
    required this.signature,
  });
}

/// 机构下单个多签账户条目。
class InstitutionAccountEntry {
  const InstitutionAccountEntry({
    required this.accountName,
    this.account,
    required this.chainStatus,
  });

  /// 账户名称（链上 name 字段）。
  final String accountName;

  /// 链上派生的多签账户（hex，上链成功后才有值）。
  final String? account;

  /// 链上状态：`Pending` / `Active` / `Closed` / `Failed`（全端统一取值）。
  final String chainStatus;

  bool get isRegistered => chainStatus == 'Active';

  /// 把 CID app 接口返回的各种 main_chain_status 取值统一映射为
  /// `Pending` / `Active` / `Closed` / `Failed`（CANON 第 5 条）：
  ///   NotOnChain / PendingOnChain → Pending
  ///   ActiveOnChain               → Active
  ///   RevokedOnChain              → Closed
  ///   失败                         → Failed
  static String normalizeChainStatus(String? raw) {
    final status = raw?.trim();
    switch (status) {
      case 'NotOnChain':
      case 'NOT_ON_CHAIN':
      case 'PendingOnChain':
      case 'PENDING_ON_CHAIN':
      case 'Pending':
        return 'Pending';
      case 'ActiveOnChain':
      case 'ACTIVE_ON_CHAIN':
      case 'Active':
      case 'Confirmed':
        return 'Active';
      case 'RevokedOnChain':
      case 'REVOKED_ON_CHAIN':
      case 'Closed':
        return 'Closed';
      case 'Failed':
      case 'FAILED':
        return 'Failed';
      default:
        return 'Pending';
    }
  }
}

/// 机构账户列表响应。
class InstitutionAccountsResponse {
  const InstitutionAccountsResponse({
    required this.cidNumber,
    required this.cidFullName,
    required this.cidShortName,
    required this.accounts,
  });

  final String cidNumber;
  final String cidFullName;
  final String cidShortName;
  final List<InstitutionAccountEntry> accounts;
}
