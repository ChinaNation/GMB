import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';

import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/services/profile_asset_service.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/user/user_service.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 编辑本人公开资料：展示名 + 签名 + 头像 + 背景。
///
/// 头像/背景上传到 R2（不上链），保存时随 `PUT /profile` 写入 object_key。
/// 首次打开若本地存有旧头像/背景（SharedPreferences），预载为待迁移资产，
/// 保存成功后清本地私有副本（零残留）。
class CitizenProfileEditPage extends StatefulWidget {
  const CitizenProfileEditPage({
    super.key,
    required this.ownerAccount,
    this.initialProfile,
    this.api,
    this.sessionProvider,
    this.assetService,
    this.imagePicker,
  });

  final String ownerAccount;
  final CitizenProfile? initialProfile;
  final CitizenProfileApi? api;
  final SquareSessionProvider? sessionProvider;
  final ProfileAssetService? assetService;
  final ImagePicker? imagePicker;

  @override
  State<CitizenProfileEditPage> createState() => _CitizenProfileEditPageState();
}

class _PendingImage {
  const _PendingImage({required this.bytes, required this.contentType});

  final Uint8List bytes;
  final String contentType;
}

class _CitizenProfileEditPageState extends State<CitizenProfileEditPage> {
  static const int _displayNameMax = 40;
  static const int _bioMax = 160;

  late final CitizenProfileApi _api;
  late final SquareSessionProvider _sessionProvider;
  late final ProfileAssetService _assetService;
  late final ImagePicker _imagePicker;
  late final TextEditingController _nameController;
  late final TextEditingController _bioController;

  _PendingImage? _pendingAvatar;
  _PendingImage? _pendingBanner;
  bool _migratedAvatar = false;
  bool _migratedBanner = false;
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    _api = widget.api ?? CitizenProfileApi();
    _sessionProvider = widget.sessionProvider ?? SquareSessionProvider.instance;
    _assetService = widget.assetService ?? ProfileAssetService();
    _imagePicker = widget.imagePicker ?? ImagePicker();
    _nameController =
        TextEditingController(text: widget.initialProfile?.displayName ?? '');
    _bioController =
        TextEditingController(text: widget.initialProfile?.bio ?? '');
    _loadLocalMigration();
  }

  @override
  void dispose() {
    _nameController.dispose();
    _bioController.dispose();
    super.dispose();
  }

  /// 若 R2 尚无头像/背景、而本地存有旧图，预载为待迁移资产。best-effort，异常忽略。
  Future<void> _loadLocalMigration() async {
    try {
      final local = await UserProfileService().getState();
      if (widget.initialProfile?.avatarObjectKey == null) {
        final avatar = await _readLocalImage(local.avatarPath);
        if (avatar != null && mounted) {
          setState(() {
            _pendingAvatar = avatar;
            _migratedAvatar = true;
          });
        }
      }
      if (widget.initialProfile?.bannerObjectKey == null) {
        final banner = await _readLocalImage(local.backgroundPath);
        if (banner != null && mounted) {
          setState(() {
            _pendingBanner = banner;
            _migratedBanner = true;
          });
        }
      }
    } on Exception {
      // 本地无 SharedPreferences 或读取失败：跳过迁移。
    }
  }

  Future<_PendingImage?> _readLocalImage(String? path) async {
    if (path == null || path.trim().isEmpty) return null;
    final file = File(path);
    if (!file.existsSync()) return null;
    final bytes = await file.readAsBytes();
    return _PendingImage(bytes: bytes, contentType: _contentTypeForPath(path));
  }

  Future<void> _pickImage(bool isAvatar) async {
    try {
      final picked = await _imagePicker.pickImage(
        source: ImageSource.gallery,
        maxWidth: isAvatar ? 1024 : 1600,
        maxHeight: isAvatar ? 1024 : 1600,
        imageQuality: 85,
      );
      if (picked == null || !mounted) return;
      final bytes = await picked.readAsBytes();
      final pending = _PendingImage(
        bytes: bytes,
        contentType: _contentTypeForPath(picked.path),
      );
      setState(() {
        if (isAvatar) {
          _pendingAvatar = pending;
          _migratedAvatar = false;
        } else {
          _pendingBanner = pending;
          _migratedBanner = false;
        }
      });
    } on Exception catch (error) {
      _snack('选择图片失败：$error');
    }
  }

  Future<void> _save() async {
    if (_saving) return;
    setState(() => _saving = true);
    try {
      final session = await _sessionProvider.ensureSession();
      if (session == null) {
        _snack('请先在「我的 → 我的钱包」创建热钱包');
        return;
      }

      String? avatarKey;
      String? avatarHash;
      if (_pendingAvatar != null) {
        final result = await _assetService.upload(
          session: session,
          kind: 'avatar',
          bytes: _pendingAvatar!.bytes,
          contentType: _pendingAvatar!.contentType,
        );
        avatarKey = result.objectKey;
        avatarHash = result.contentHash;
      }

      String? bannerKey;
      String? bannerHash;
      if (_pendingBanner != null) {
        final result = await _assetService.upload(
          session: session,
          kind: 'banner',
          bytes: _pendingBanner!.bytes,
          contentType: _pendingBanner!.contentType,
        );
        bannerKey = result.objectKey;
        bannerHash = result.contentHash;
      }

      final updated = await _api.updateProfile(
        session: session,
        displayName: _nameController.text.trim(),
        bio: _bioController.text.trim(),
        avatarObjectKey: avatarKey,
        avatarContentHash: avatarHash,
        bannerObjectKey: bannerKey,
        bannerContentHash: bannerHash,
      );

      await _clearMigratedLocals(
        clearAvatar: _migratedAvatar && avatarKey != null,
        clearBanner: _migratedBanner && bannerKey != null,
      );

      if (!mounted) return;
      Navigator.of(context).pop(updated);
    } on SquareApiException catch (error) {
      if (!mounted) return;
      _snack(error.message);
    } on Exception {
      if (!mounted) return;
      _snack('保存失败，请重试');
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  Future<void> _clearMigratedLocals({
    required bool clearAvatar,
    required bool clearBanner,
  }) async {
    try {
      final service = UserProfileService();
      if (clearAvatar) await service.updateAvatarPath(null);
      if (clearBanner) await service.updateBackgroundPath(null);
    } on Exception {
      // 迁移后清本地失败不阻断保存结果。
    }
  }

  void _snack(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message)),
    );
  }

  String _contentTypeForPath(String path) {
    final lower = path.toLowerCase();
    if (lower.endsWith('.png')) return 'image/png';
    if (lower.endsWith('.webp')) return 'image/webp';
    return 'image/jpeg';
  }

  @override
  Widget build(BuildContext context) {
    final avatarKey = widget.initialProfile?.avatarObjectKey;
    final bannerKey = widget.initialProfile?.bannerObjectKey;
    return Scaffold(
      appBar: AppBar(
        title: const Text('编辑资料'),
        centerTitle: true,
        actions: [
          TextButton(
            onPressed: _saving ? null : _save,
            child: _saving
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Text('保存'),
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          _AssetRow(
            label: '背景',
            width: double.infinity,
            height: 120,
            radius: AppTheme.radiusMd,
            preview: _pendingBanner?.bytes,
            networkUrl: bannerKey == null ? null : _api.mediaUrl(bannerKey),
            onTap: () => _pickImage(false),
          ),
          const SizedBox(height: 16),
          _AssetRow(
            label: '头像',
            width: 84,
            height: 84,
            radius: 16,
            preview: _pendingAvatar?.bytes,
            networkUrl: avatarKey == null ? null : _api.mediaUrl(avatarKey),
            onTap: () => _pickImage(true),
          ),
          const SizedBox(height: 20),
          TextField(
            controller: _nameController,
            maxLength: _displayNameMax,
            decoration: const InputDecoration(
              labelText: '展示名',
              hintText: '留空则显示钱包名',
            ),
          ),
          const SizedBox(height: 16),
          TextField(
            controller: _bioController,
            maxLength: _bioMax,
            maxLines: 4,
            decoration: const InputDecoration(
              labelText: '个性签名',
              alignLabelWithHint: true,
            ),
          ),
        ],
      ),
    );
  }
}

class _AssetRow extends StatelessWidget {
  const _AssetRow({
    required this.label,
    required this.width,
    required this.height,
    required this.radius,
    required this.preview,
    required this.networkUrl,
    required this.onTap,
  });

  final String label;
  final double width;
  final double height;
  final double radius;
  final Uint8List? preview;
  final String? networkUrl;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Text(
          label,
          style: const TextStyle(fontSize: 15, color: AppTheme.textPrimary),
        ),
        const Spacer(),
        InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(radius),
          child: Container(
            width: width == double.infinity ? 200 : width,
            height: height,
            decoration: BoxDecoration(
              color: AppTheme.surfaceElevated,
              borderRadius: BorderRadius.circular(radius),
              border: Border.all(color: AppTheme.border),
            ),
            clipBehavior: Clip.antiAlias,
            child: _buildContent(),
          ),
        ),
      ],
    );
  }

  Widget _buildContent() {
    final bytes = preview;
    if (bytes != null) {
      return Image.memory(bytes, fit: BoxFit.cover);
    }
    final url = networkUrl;
    if (url != null) {
      return Image.network(
        url,
        fit: BoxFit.cover,
        errorBuilder: (_, __, ___) => _placeholder(),
      );
    }
    return _placeholder();
  }

  Widget _placeholder() {
    return const Center(
      child: Icon(Icons.add_a_photo_outlined,
          size: 26, color: AppTheme.textTertiary),
    );
  }
}
