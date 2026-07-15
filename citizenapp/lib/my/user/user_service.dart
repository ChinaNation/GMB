import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

class UserProfileState {
  const UserProfileState({
    this.avatarPath,
    this.backgroundPath,
  });

  final String? avatarPath;
  final String? backgroundPath;

  UserProfileState copyWith({
    Object? avatarPath = _sentinel,
    Object? backgroundPath = _sentinel,
  }) {
    return UserProfileState(
      avatarPath: identical(avatarPath, _sentinel)
          ? this.avatarPath
          : avatarPath as String?,
      backgroundPath: identical(backgroundPath, _sentinel)
          ? this.backgroundPath
          : backgroundPath as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'avatar_path': avatarPath,
      'background_path': backgroundPath,
    };
  }

  static UserProfileState fromJson(Map<String, dynamic> json) {
    return UserProfileState(
      avatarPath: _normalizeOptionalString(json['avatar_path']),
      backgroundPath: _normalizeOptionalString(json['background_path']),
    );
  }
}

class UserProfileService {
  static const String _kProfile = 'user.profile.state.v2';

  Future<UserProfileState> getState() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_kProfile);
    if (raw == null || raw.trim().isEmpty) {
      return const UserProfileState();
    }

    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) {
        throw const FormatException('profile payload invalid');
      }
      return UserProfileState.fromJson(decoded);
    } catch (_) {
      return const UserProfileState();
    }
  }

  Future<UserProfileState> saveState(UserProfileState state) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kProfile, jsonEncode(state.toJson()));
    return state;
  }

  Future<UserProfileState> updateAvatarPath(String? avatarPath) async {
    final current = await getState();
    return saveState(current.copyWith(avatarPath: avatarPath));
  }

  Future<UserProfileState> updateBackgroundPath(String? backgroundPath) async {
    final current = await getState();
    return saveState(current.copyWith(backgroundPath: backgroundPath));
  }
}

const Object _sentinel = Object();

String? _normalizeOptionalString(Object? value) {
  final normalized = value?.toString().trim() ?? '';
  if (normalized.isEmpty) {
    return null;
  }
  return normalized;
}
