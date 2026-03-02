import 'package:shared_preferences/shared_preferences.dart';

class UserProfileState {
  const UserProfileState({
    required this.nickname,
    this.avatarPath,
  });

  final String nickname;
  final String? avatarPath;
}

class UserProfileService {
  static const _kNickname = 'user.profile.nickname';
  static const _kAvatarPath = 'user.profile.avatar_path';

  Future<UserProfileState> getState() async {
    final prefs = await SharedPreferences.getInstance();
    return UserProfileState(
      nickname: prefs.getString(_kNickname) ?? '公民用户',
      avatarPath: prefs.getString(_kAvatarPath),
    );
  }

  Future<UserProfileState> saveState(UserProfileState state) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kNickname, state.nickname.trim());
    if (state.avatarPath == null || state.avatarPath!.trim().isEmpty) {
      await prefs.remove(_kAvatarPath);
    } else {
      await prefs.setString(_kAvatarPath, state.avatarPath!.trim());
    }
    return getState();
  }
}
