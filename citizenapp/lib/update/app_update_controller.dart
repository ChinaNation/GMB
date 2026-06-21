import 'dart:io';

import 'package:flutter/foundation.dart';

import 'app_update_manifest.dart';
import 'app_update_service.dart';

enum AppUpdateStatus {
  idle,
  checking,
  available,
  downloading,
  installing,
  error,
}

class AppUpdateState {
  const AppUpdateState({
    required this.status,
    this.currentVersion,
    this.update,
    this.progress = 0,
    this.errorMessage,
    this.downloadedApk,
  });

  const AppUpdateState.idle()
      : status = AppUpdateStatus.idle,
        currentVersion = null,
        update = null,
        progress = 0,
        errorMessage = null,
        downloadedApk = null;

  final AppUpdateStatus status;
  final AppVersionInfo? currentVersion;
  final AppUpdateInfo? update;
  final double progress;
  final String? errorMessage;
  final File? downloadedApk;

  bool get hasUpdate => update != null;

  String get versionLabel {
    final version = currentVersion;
    if (version == null || version.versionName.isEmpty) {
      return 'v...';
    }
    return version.label;
  }
}

class AppUpdateController extends ChangeNotifier {
  AppUpdateController._({AppUpdateService? service})
      : _service = service ?? AppUpdateService();

  static final AppUpdateController instance = AppUpdateController._();

  final AppUpdateService _service;
  AppUpdateState _state = const AppUpdateState.idle();

  AppUpdateState get state => _state;

  Future<void> check({bool force = false}) async {
    if (!force && _state.status == AppUpdateStatus.checking) {
      return;
    }
    if (_state.status == AppUpdateStatus.downloading ||
        _state.status == AppUpdateStatus.installing) {
      return;
    }

    _setState(AppUpdateState(
      status: AppUpdateStatus.checking,
      currentVersion: _state.currentVersion,
      update: _state.update,
    ));

    try {
      final result = await _service.checkForUpdate();
      _setState(AppUpdateState(
        status: result.update == null
            ? AppUpdateStatus.idle
            : AppUpdateStatus.available,
        currentVersion: result.currentVersion,
        update: result.update,
      ));
    } catch (e) {
      debugPrint('[AppUpdate] 检查更新失败: $e');
      _setState(AppUpdateState(
        status: AppUpdateStatus.error,
        currentVersion: _state.currentVersion,
        update: _state.update,
        errorMessage: '检查更新失败：$e',
      ));
    }
  }

  Future<bool> downloadAndInstall() async {
    final update = _state.update;
    if (update == null ||
        _state.status == AppUpdateStatus.downloading ||
        _state.status == AppUpdateStatus.installing) {
      return false;
    }

    _setState(AppUpdateState(
      status: AppUpdateStatus.downloading,
      currentVersion: _state.currentVersion,
      update: update,
    ));

    try {
      final apkFile = await _service.downloadUpdate(
        update,
        onProgress: (received, total) {
          final progress = total == null || total <= 0 ? 0.0 : received / total;
          _setState(AppUpdateState(
            status: AppUpdateStatus.downloading,
            currentVersion: _state.currentVersion,
            update: update,
            progress: progress.clamp(0.0, 1.0).toDouble(),
          ));
        },
      );

      _setState(AppUpdateState(
        status: AppUpdateStatus.installing,
        currentVersion: _state.currentVersion,
        update: update,
        progress: 1,
        downloadedApk: apkFile,
      ));

      final started = await _service.installApk(apkFile);
      _setState(AppUpdateState(
        status: started ? AppUpdateStatus.available : AppUpdateStatus.error,
        currentVersion: _state.currentVersion,
        update: update,
        progress: 1,
        downloadedApk: apkFile,
        errorMessage: started ? null : '请允许安装未知应用后再次点击更新',
      ));
      return started;
    } catch (e) {
      debugPrint('[AppUpdate] 下载或安装更新失败: $e');
      _setState(AppUpdateState(
        status: AppUpdateStatus.error,
        currentVersion: _state.currentVersion,
        update: update,
        errorMessage: '更新失败：$e',
      ));
      return false;
    }
  }

  void _setState(AppUpdateState value) {
    _state = value;
    notifyListeners();
  }
}
