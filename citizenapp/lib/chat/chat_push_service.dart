import 'dart:async';
import 'dart:io';

import 'package:firebase_core/firebase_core.dart';
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:shared_preferences/shared_preferences.dart';

const _wakeSendersKey = 'chat.push.wake_senders';
// Firebase 客户端标识属于公开应用配置；构建参数仍可用于独立环境覆盖。
const _firebaseApiKey = String.fromEnvironment(
  'FIREBASE_API_KEY',
  defaultValue: 'AIzaSyBfXLIwqGoOX_h75MxZYcorJncT3uSZrm4',
);
const _firebaseProjectId = String.fromEnvironment(
  'FIREBASE_PROJECT_ID',
  defaultValue: 'citizenapp-23542',
);
const _firebaseSenderId = String.fromEnvironment(
  'FIREBASE_MESSAGING_SENDER_ID',
  defaultValue: '124593150477',
);
const _firebaseAndroidAppId = String.fromEnvironment(
  'FIREBASE_ANDROID_APP_ID',
  defaultValue: '1:124593150477:android:ba7e25d5229dde5bba1344',
);
const _firebaseIosAppId = String.fromEnvironment(
  'FIREBASE_IOS_APP_ID',
  defaultValue: '1:124593150477:ios:ff23b6c7ee249ecfba1344',
);

FirebaseOptions _firebaseOptions() {
  final appId = Platform.isIOS ? _firebaseIosAppId : _firebaseAndroidAppId;
  if (_firebaseApiKey.isEmpty ||
      _firebaseProjectId.isEmpty ||
      _firebaseSenderId.isEmpty ||
      appId.isEmpty) {
    throw StateError('Chat 推送缺少 Firebase 构建参数');
  }
  return FirebaseOptions(
    apiKey: _firebaseApiKey,
    appId: appId,
    messagingSenderId: _firebaseSenderId,
    projectId: _firebaseProjectId,
    iosBundleId: Platform.isIOS ? 'org.citizenapp' : null,
  );
}

Future<void> ensureChatFirebaseReady() async {
  if (Firebase.apps.isEmpty) {
    await Firebase.initializeApp(options: _firebaseOptions());
  }
}

class ChatPushToken {
  const ChatPushToken({required this.provider, required this.token});

  final String provider;
  final String token;
}

/// 平台推送只负责唤醒本地重试，不承载消息、会话或附件内容。
class ChatPushService {
  ChatPushService();

  final StreamController<String> _wakeController =
      StreamController<String>.broadcast();
  final StreamController<ChatPushToken> _tokenController =
      StreamController<ChatPushToken>.broadcast();
  StreamSubscription<RemoteMessage>? _messageSubscription;
  StreamSubscription<RemoteMessage>? _openedSubscription;
  StreamSubscription<String>? _tokenSubscription;

  Stream<String> get wakeSenders => _wakeController.stream;
  Stream<ChatPushToken> get tokenChanges => _tokenController.stream;

  Future<ChatPushToken> initialize() async {
    final token = await readToken(requestPermission: true);
    _messageSubscription ??= FirebaseMessaging.onMessage.listen(_handleMessage);
    _openedSubscription ??=
        FirebaseMessaging.onMessageOpenedApp.listen(_handleMessage);
    _tokenSubscription ??= FirebaseMessaging.instance.onTokenRefresh.listen(
      (_) async {
        try {
          _tokenController.add(await readToken(requestPermission: false));
        } catch (_) {
          // Token 刷新读取失败时等待下一次平台回调或 Chat 初始化重试。
        }
      },
    );
    final initial = await FirebaseMessaging.instance.getInitialMessage();
    if (initial != null) _handleMessage(initial);
    return token;
  }

  /// 后台唤醒只读取已有平台 Token，不触发权限弹窗或前台消息订阅。
  Future<ChatPushToken> readToken({required bool requestPermission}) async {
    if (!Platform.isAndroid && !Platform.isIOS) {
      throw UnsupportedError('Chat 推送只支持 Android 和 iOS');
    }
    await ensureChatFirebaseReady();
    final messaging = FirebaseMessaging.instance;
    if (requestPermission) {
      await messaging.requestPermission(alert: true, badge: true, sound: true);
    }

    if (Platform.isIOS) {
      final token = await messaging.getAPNSToken();
      if (token == null || token.isEmpty) {
        throw StateError('APNs Token 尚未生成');
      }
      return ChatPushToken(provider: 'apns', token: token);
    }
    final token = await messaging.getToken();
    if (token == null || token.isEmpty) {
      throw StateError('FCM Token 尚未生成');
    }
    return ChatPushToken(provider: 'fcm', token: token);
  }

  static Future<void> storeWakeSender(String sender) async {
    final prefs = await SharedPreferences.getInstance();
    final senders = prefs.getStringList(_wakeSendersKey) ?? <String>[];
    if (!senders.contains(sender)) senders.add(sender);
    await prefs.setStringList(_wakeSendersKey, senders);
  }

  Future<List<String>> takePendingWakeSenders() async {
    final prefs = await SharedPreferences.getInstance();
    final senders = prefs.getStringList(_wakeSendersKey) ?? const <String>[];
    await prefs.remove(_wakeSendersKey);
    return List<String>.unmodifiable(senders);
  }

  void _handleMessage(RemoteMessage message) {
    final sender = wakeSenderFromData(message.data);
    if (sender != null) _wakeController.add(sender);
  }

  /// 推送正文只能识别无内容唤醒，不接受消息或附件字段。
  static String? wakeSenderFromData(Map<String, dynamic> data) {
    if (data['kind'] != 'chat_wake' || data.length != 2) return null;
    final sender = data['sender_account'];
    return sender is String && sender.isNotEmpty ? sender : null;
  }

  Future<void> dispose() async {
    await _messageSubscription?.cancel();
    await _openedSubscription?.cancel();
    await _tokenSubscription?.cancel();
    await _wakeController.close();
    await _tokenController.close();
  }
}
