import 'package:flutter/material.dart';

import '../../ui/app_theme.dart';

/// 媒体来源。emoji / 贴纸面板归步骤3,不在此。
enum ChatMediaSource {
  galleryImage,
  cameraPhoto,
  galleryVideo,
  cameraVideo,
  file,
}

/// 点击输入栏加号后弹出的媒体来源选择弹层。返回用户所选来源;取消返回 null。
Future<ChatMediaSource?> showChatMediaSourceSheet(BuildContext context) {
  return showModalBottomSheet<ChatMediaSource>(
    context: context,
    backgroundColor: AppTheme.surfaceCard,
    showDragHandle: true,
    builder: (context) => const SafeArea(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          _SourceTile(
            icon: Icons.photo_library_rounded,
            label: '相册照片',
            source: ChatMediaSource.galleryImage,
          ),
          _SourceTile(
            icon: Icons.photo_camera_rounded,
            label: '拍照',
            source: ChatMediaSource.cameraPhoto,
          ),
          _SourceTile(
            icon: Icons.video_library_rounded,
            label: '相册视频',
            source: ChatMediaSource.galleryVideo,
          ),
          _SourceTile(
            icon: Icons.videocam_rounded,
            label: '录像',
            source: ChatMediaSource.cameraVideo,
          ),
          _SourceTile(
            icon: Icons.insert_drive_file_rounded,
            label: '文件',
            source: ChatMediaSource.file,
          ),
          SizedBox(height: 8),
        ],
      ),
    ),
  );
}

class _SourceTile extends StatelessWidget {
  const _SourceTile({
    required this.icon,
    required this.label,
    required this.source,
  });

  final IconData icon;
  final String label;
  final ChatMediaSource source;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      leading: Icon(icon, color: AppTheme.textPrimary),
      title: Text(
        label,
        style: const TextStyle(fontSize: 15, color: AppTheme.textPrimary),
      ),
      onTap: () => Navigator.of(context).pop(source),
    );
  }
}
