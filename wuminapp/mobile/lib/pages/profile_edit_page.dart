import 'dart:io';

import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:wuminapp_mobile/services/user_profile_service.dart';

class ProfileEditPage extends StatefulWidget {
  const ProfileEditPage({
    super.key,
    required this.initialState,
  });

  final UserProfileState initialState;

  @override
  State<ProfileEditPage> createState() => _ProfileEditPageState();
}

class _ProfileEditPageState extends State<ProfileEditPage> {
  final ImagePicker _imagePicker = ImagePicker();
  late final TextEditingController _nicknameController;
  String? _avatarPath;

  @override
  void initState() {
    super.initState();
    _nicknameController =
        TextEditingController(text: widget.initialState.nickname);
    _avatarPath = widget.initialState.avatarPath;
  }

  @override
  void dispose() {
    _nicknameController.dispose();
    super.dispose();
  }

  Future<void> _pickAvatarFromGallery() async {
    try {
      final picked = await _imagePicker.pickImage(
        source: ImageSource.gallery,
        maxWidth: 1024,
        maxHeight: 1024,
      );
      if (picked == null || !mounted) {
        return;
      }
      setState(() {
        _avatarPath = picked.path;
      });
    } catch (e) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('访问相册失败：$e')),
      );
    }
  }

  void _save() {
    final nickname = _nicknameController.text.trim();
    if (nickname.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('用户昵称不能为空')),
      );
      return;
    }
    Navigator.of(context).pop(
      UserProfileState(
        nickname: nickname,
        avatarPath: _avatarPath,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('修改用户资料'),
        centerTitle: true,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                _SquareAvatar(path: _avatarPath, size: 84),
                const SizedBox(width: 14),
                OutlinedButton(
                  onPressed: _pickAvatarFromGallery,
                  child: const Text('设置头像'),
                ),
              ],
            ),
            const SizedBox(height: 18),
            TextField(
              controller: _nicknameController,
              decoration: const InputDecoration(
                labelText: '用户昵称',
                hintText: '请输入用户昵称',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 18),
            FilledButton(
              onPressed: _save,
              child: const Text('保存'),
            ),
          ],
        ),
      ),
    );
  }
}

class _SquareAvatar extends StatelessWidget {
  const _SquareAvatar({required this.path, required this.size});

  final String? path;
  final double size;

  @override
  Widget build(BuildContext context) {
    final hasImage = path != null && path!.trim().isNotEmpty;
    final validImage = hasImage && File(path!).existsSync();
    return ClipRRect(
      borderRadius: BorderRadius.circular(10),
      child: Container(
        width: size,
        height: size,
        color: const Color(0xFFE3EFE8),
        child: validImage
            ? Image.file(
                File(path!),
                fit: BoxFit.cover,
              )
            : const Icon(
                Icons.person,
                size: 40,
                color: Color(0xFF0B3D2E),
              ),
      ),
    );
  }
}
