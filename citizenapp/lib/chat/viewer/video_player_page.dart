import 'dart:io';

import 'package:flutter/material.dart';
import 'package:video_player/video_player.dart';

import '../media/media_gallery_saver.dart';

/// 视频播放页:video_player 播放 + 保存到相册。
///
/// 视频播放依赖 native 平台通道,以真机/集成验证为准(单测不驱动 native controller)。
class VideoPlayerPage extends StatefulWidget {
  const VideoPlayerPage({
    super.key,
    required this.filePath,
    required this.fileName,
    this.onSaveToGallery,
  });

  final String filePath;
  final String fileName;
  final GallerySaveFn? onSaveToGallery;

  @override
  State<VideoPlayerPage> createState() => _VideoPlayerPageState();
}

class _VideoPlayerPageState extends State<VideoPlayerPage> {
  late final VideoPlayerController _controller;
  bool _ready = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _controller = VideoPlayerController.file(File(widget.filePath));
    _controller.initialize().then((_) {
      if (!mounted) return;
      setState(() => _ready = true);
      _startPlayback();
    }).catchError((Object error) {
      if (!mounted) return;
      setState(() => _error = '无法播放该视频');
    });
  }

  void _startPlayback() {
    _controller.setLooping(true);
    _controller.play();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    final saver =
        widget.onSaveToGallery ?? const MediaGallerySaver().saveToGallery;
    final ok =
        await saver(filePath: widget.filePath, fileName: widget.fileName);
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(ok ? '已保存到相册' : '保存失败')),
    );
  }

  void _togglePlay() {
    setState(() {
      _controller.value.isPlaying ? _controller.pause() : _controller.play();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        backgroundColor: Colors.black,
        foregroundColor: Colors.white,
        elevation: 0,
        actions: [
          IconButton(
            tooltip: '保存到相册',
            icon: const Icon(Icons.download_rounded),
            onPressed: _save,
          ),
        ],
      ),
      body: Center(child: _body()),
    );
  }

  Widget _body() {
    if (_error != null) {
      return Text(_error!, style: const TextStyle(color: Colors.white70));
    }
    if (!_ready) {
      return const CircularProgressIndicator(color: Colors.white);
    }
    return GestureDetector(
      onTap: _togglePlay,
      child: AspectRatio(
        aspectRatio: _controller.value.aspectRatio,
        child: VideoPlayer(_controller),
      ),
    );
  }
}
