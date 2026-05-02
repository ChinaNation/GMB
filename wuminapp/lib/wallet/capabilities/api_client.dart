import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:http/http.dart' as http;

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
    required this.institutionIdHex,
    required this.org,
  });

  final String pubkeyHex;
  final String roleName;
  final String institutionName;
  final String institutionIdHex;
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
  /// feedback_pubkey_format_rule.md:内部统一 0x 小写 hex。
  final String signerAdminPubkey;
}

class ApiClient {
  ApiClient({String? baseUrl}) : _baseUrl = baseUrl ?? _defaultBaseUrl;

  final String _baseUrl;

  static String get _defaultBaseUrl {
    const fromDefine = String.fromEnvironment('WUMINAPP_API_BASE_URL');
    if (fromDefine.isNotEmpty) {
      return fromDefine;
    }
    return 'http://127.0.0.1:8787';
  }

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

  /// 注册投票账户（带 sr25519 签名证明私钥所有权）。
  Future<void> registerVoteAccount({
    required String address,
    required String pubkeyHex,
    required String signatureHex,
    required String signMessage,
  }) async {
    final normalized = _normalizePubkeyHex(pubkeyHex);
    final uri = Uri.parse('$_baseUrl/api/v1/app/vote-account/register');
    http.Response response;
    try {
      response = await http
          .post(
            uri,
            headers: _headers(includeContentType: true),
            body: jsonEncode({
              'address': address,
              'pubkey': normalized,
              'signature': signatureHex.startsWith('0x')
                  ? signatureHex
                  : '0x$signatureHex',
              'sign_message': signMessage,
            }),
          )
          .timeout(const Duration(seconds: 15));
    } on TimeoutException catch (_) {
      throw Exception('注册请求超时，请检查网络连接');
    } on SocketException catch (_) {
      if ((Platform.isAndroid || Platform.isIOS) &&
          _baseUrl.contains('127.0.0.1')) {
        throw Exception(
          '当前使用$_baseUrl，手机真机无法访问本机回环地址。请用 --dart-define=WUMINAPP_API_BASE_URL=http://<电脑局域网IP>:8787',
        );
      }
      rethrow;
    }
    if (response.statusCode != 200) {
      throw Exception('vote account register failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception(
        'vote account register rejected: code=$code message=$message',
      );
    }
  }

  /// 查询投票账户绑定状态。
  ///
  /// [walletAddress] 必须是 SS58 格式地址（后端按 address 参数接收并解析）。
  Future<VoteAccountStatusResponse> queryVoteAccountStatus(
      String walletAddress) async {
    final addr = walletAddress.trim();
    if (addr.isEmpty) {
      throw Exception('walletAddress is empty');
    }
    final uri =
        Uri.parse('$_baseUrl/api/v1/app/vote-account/status?address=$addr');
    http.Response response;
    try {
      response = await http
          .get(uri, headers: _headers())
          .timeout(const Duration(seconds: 15));
    } on TimeoutException catch (_) {
      throw Exception('状态查询超时，请检查网络连接');
    } on SocketException catch (_) {
      if ((Platform.isAndroid || Platform.isIOS) &&
          _baseUrl.contains('127.0.0.1')) {
        throw Exception(
          '当前使用$_baseUrl，手机真机无法访问本机回环地址。请用 --dart-define=WUMINAPP_API_BASE_URL=http://<电脑局域网IP>:8787',
        );
      }
      rethrow;
    }
    if (response.statusCode != 200) {
      throw Exception(
          'vote account status query failed: ${response.statusCode}');
    }

    final payload = jsonDecode(response.body) as Map<String, dynamic>;
    final code = payload['code'] as int? ?? -1;
    final message = payload['message']?.toString() ?? 'unknown';
    if (code != 0) {
      throw Exception(
        'vote account status rejected: code=$code message=$message',
      );
    }

    final data = payload['data'];
    if (data is! Map<String, dynamic>) {
      throw Exception('vote account status invalid response: missing data');
    }

    return VoteAccountStatusResponse(
      status: (data['status']?.toString() ?? 'unset').trim(),
      address: data['address']?.toString(),
      sfidCode: data['sfid_code']?.toString(),
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
      final institutionId =
          (m['institution_id_hex']?.toString() ?? '').trim().toLowerCase();
      if (pubkey.isEmpty || role.isEmpty || institutionName.isEmpty) {
        continue;
      }
      entries.add(
        AdminCatalogEntryResponse(
          pubkeyHex: pubkey.startsWith('0x') ? pubkey.substring(2) : pubkey,
          roleName: role,
          institutionName: institutionName,
          institutionIdHex: institutionId,
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
      if ((Platform.isAndroid || Platform.isIOS) &&
          _baseUrl.contains('127.0.0.1')) {
        throw Exception(
          '当前使用$_baseUrl，手机真机无法访问本机回环地址。请用 --dart-define=WUMINAPP_API_BASE_URL=http://<电脑局域网IP>:8787',
        );
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
      throw Exception(
          'population snapshot 缺少 province 字段(ADR-008 step3 必填)');
    }
    if (signerAdminPubkeyRaw.isEmpty) {
      throw Exception(
          'population snapshot 缺少 signer_admin_pubkey 字段(ADR-008 step3 必填)');
    }
    // feedback_pubkey_format_rule.md:统一 0x 小写 hex。
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
  /// 调用 SFID 后端 `GET /api/v1/app/institution/:sfid_id/accounts`，
  /// 返回机构名称 + 账户列表（account_name / duoqian_address / chain_status）。
  Future<InstitutionAccountsResponse> fetchInstitutionAccounts(
      String sfidId) async {
    final trimmed = sfidId.trim();
    if (trimmed.isEmpty) {
      throw Exception('SFID ID 不能为空');
    }
    final uri = Uri.parse(
        '$_baseUrl/api/v1/app/institution/${Uri.encodeComponent(trimmed)}/accounts');
    http.Response response;
    try {
      response = await http
          .get(uri, headers: _headers())
          .timeout(const Duration(seconds: 15));
    } on TimeoutException catch (_) {
      throw Exception('查询超时，请检查网络连接');
    } on SocketException catch (_) {
      if ((Platform.isAndroid || Platform.isIOS) &&
          _baseUrl.contains('127.0.0.1')) {
        throw Exception(
          '当前使用$_baseUrl，手机真机无法访问本机回环地址。请用 --dart-define=WUMINAPP_API_BASE_URL=http://<电脑局域网IP>:8787',
        );
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
      sfidId: (data['sfid_id']?.toString() ?? trimmed).trim(),
      institutionName: (data['institution_name']?.toString() ?? '').trim(),
      accounts: accounts,
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
      if ((Platform.isAndroid || Platform.isIOS) &&
          _baseUrl.contains('127.0.0.1')) {
        throw Exception(
          '当前使用$_baseUrl，手机真机无法访问本机回环地址。请用 --dart-define=WUMINAPP_API_BASE_URL=http://<电脑局域网IP>:8787',
        );
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

class VoteAccountStatusResponse {
  const VoteAccountStatusResponse({
    required this.status,
    this.address,
    this.sfidCode,
  });

  /// "pending" | "bound" | "unset"
  final String status;
  final String? address;
  final String? sfidCode;
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
      case 'PENDING':
      case 'REGISTERED':
      case 'FAILED':
        return status!;
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
    required this.sfidId,
    required this.institutionName,
    required this.accounts,
  });

  final String sfidId;
  final String institutionName;
  final List<InstitutionAccountEntry> accounts;
}
