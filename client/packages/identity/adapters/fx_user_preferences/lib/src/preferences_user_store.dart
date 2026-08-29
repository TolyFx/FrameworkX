import 'dart:convert';

import 'package:fx_user_core/fx_user_core.dart';
import 'package:fx_user_session/fx_user_session.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// 通过命名空间隔离不同产品的用户凭据与资料快照。
final class FxPreferencesUserStore
    implements AuthCredentialStore, UserSnapshotStore {
  /// 产品级存储命名空间。
  final String namespace;

  /// 异步配置存储入口。
  final SharedPreferencesAsync preferences;

  FxPreferencesUserStore({
    required this.namespace,
    SharedPreferencesAsync? preferences,
  }) : preferences = preferences ?? SharedPreferencesAsync();

  String get _credentialKey => '$namespace.user.session.v1';

  String get _snapshotKey => '$namespace.user.snapshot.v1';

  @override
  Future<UserCredential?> read() async {
    final String? source = await preferences.getString(_credentialKey);
    if (source == null || source.isEmpty) return null;
    try {
      return UserCredential.fromJson(_decode(source));
    } catch (_) {
      await clear();
      return null;
    }
  }

  @override
  Future<void> write(UserCredential credential) {
    return preferences.setString(
      _credentialKey,
      jsonEncode(credential.toJson()),
    );
  }

  @override
  Future<void> clear() => preferences.remove(_credentialKey);

  @override
  Future<FxUser?> readSnapshot() async {
    final String? source = await preferences.getString(_snapshotKey);
    if (source == null || source.isEmpty) return null;
    try {
      return FxUser.fromJson(_decode(source));
    } catch (_) {
      await clearSnapshot();
      return null;
    }
  }

  @override
  Future<void> writeSnapshot(FxUser user) {
    return preferences.setString(_snapshotKey, jsonEncode(user.toJson()));
  }

  @override
  Future<void> clearSnapshot() => preferences.remove(_snapshotKey);

  /// 将持久化 JSON 解码为类型安全对象。
  Map<String, dynamic> _decode(String source) {
    return Map<String, dynamic>.from(jsonDecode(source) as Map);
  }
}
