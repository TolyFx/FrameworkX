import 'dart:async';
import 'dart:typed_data';

import 'package:bloc/bloc.dart';
import 'package:fx_user_core/fx_user_core.dart';

import '../contract/auth_credential_store.dart';
import '../contract/fx_identity_codec.dart';
import '../contract/fx_avatar_upload_task.dart';
import '../contract/fx_user_session_source.dart';
import '../contract/user_snapshot_store.dart';
import '../domain/fx_identity.dart';
import '../domain/fx_user_session.dart';

/// 用户认证与应用会话的唯一状态拥有者。
class FxUserSessionCubit extends Cubit<FxUserSession>
    implements FxUserSessionSource {
  final FxUserRepository repository;
  final AuthCredentialStore credentialStore;
  final FxIdentityCodec<FxUser> identityCodec;

  /// 可选的用户资料快照存储。
  final UserSnapshotStore? snapshotStore;
  final FxAvatarUploadTask? avatarUploadTask;
  final void Function(UserCredential? credential)? onCredentialChanged;

  UserCredential? _credential;

  /// 会话存储操作队列，保证凭据与用户快照按状态产生顺序写入。
  Future<void> _storageQueue = Future<void>.value();

  /// 当前会话存储版本，用于跳过已经被新状态取代的待执行写入。
  int _storageRevision = 0;

  FxUserSessionCubit({
    required this.repository,
    required this.credentialStore,
    required this.identityCodec,
    this.snapshotStore,
    this.avatarUploadTask,
    this.onCredentialChanged,
  }) : super(const FxAuthing());

  UserCredential? get credential => _credential;

  Future<bool> restore() async {
    authing();
    final credential = await credentialStore.read();
    if (credential == null) {
      guest();
      return false;
    }
    _setCredential(credential);
    try {
      final FxUser? snapshot = await snapshotStore?.readSnapshot();
      final FxUser remoteUser = await repository.currentUser();
      _activate(credential, _mergeUser(snapshot, remoteUser));
      return true;
    } catch (_) {
      await _clear();
      guest();
      return false;
    }
  }

  Future<void> authenticate(AuthRequest request) async {
    authing();
    try {
      final result = await repository.authenticate(request);
      _setCredential(result.credential);
      _activate(result.credential, result.user);
    } catch (_) {
      await _clear();
      guest();
      rethrow;
    }
  }

  Future<String?> requestCode({
    required String channel,
    required String identifier,
  }) =>
      repository.requestCode(channel: channel, identifier: identifier);

  Future<void> updateProfile(UserProfilePatch patch) async {
    final credential = _credential;
    if (credential == null) throw StateError('Authentication is required.');
    final user = await repository.updateProfile(patch);
    _activate(credential, user);
  }

  /// 使用当前凭据刷新用户资料；请求期间若账号已经切换，则丢弃迟到响应。
  Future<bool> refreshCurrentUser() async {
    final UserCredential? credential = _credential;
    if (credential == null) return false;
    try {
      final FxUser user = await repository.currentUser();
      if (!identical(_credential, credential)) return false;
      _activate(credential, user);
      return true;
    } catch (_) {
      // 临时网络错误不应主动清除当前会话；鉴权失效由宿主 401 拦截器处理。
      return false;
    }
  }

  /// 上传头像并更新资料，完成后发出新的已认证会话状态。
  Future<void> updateAvatar(Uint8List bytes) async {
    final credential = _credential;
    final task = avatarUploadTask;
    if (credential == null) {
      throw StateError('Authentication is required.');
    }
    if (task == null) {
      throw UnsupportedError('Avatar upload is not configured.');
    }
    final avatar = await task.upload(bytes: bytes, credential: credential);
    await updateProfile(UserProfilePatch(avatar: avatar));
  }

  Future<bool> setPassword(String newPassword) async {
    final ok = await repository.setPassword(newPassword);
    if (ok) await _refreshUser();
    return ok;
  }

  Future<bool> changePassword({
    required String oldPassword,
    required String newPassword,
  }) async {
    final ok = await repository.changePassword(
      oldPassword: oldPassword,
      newPassword: newPassword,
    );
    if (ok) await _refreshUser();
    return ok;
  }

  Future<void> logout() async {
    try {
      if (_credential != null) await repository.logout();
    } finally {
      await _clear();
      guest();
    }
  }

  Future<void> deleteAccount(String password) async {
    await repository.deleteAccount(password);
    await _clear();
    guest();
  }

  Future<void> handleUnauthorized() async {
    await _clear();
    guest();
  }

  void authing() => emit(const FxAuthing());
  void guest() => emit(const FxGuest());
  void authed(FxIdentity user) => emit(FxAuthed(user));
  void update(FxUserSession session) => emit(session);

  @override
  FxUserSession get session => state;

  @override
  Stream<FxUserSession> get sessions => stream;

  void _activate(UserCredential credential, FxUser user) {
    _credential = credential;
    _setCredential(credential);
    authed(identityCodec.decode(user));
    final int revision = ++_storageRevision;
    unawaited(_enqueuePersist(revision, credential, user));
  }

  void _setCredential(UserCredential? credential) {
    _credential = credential;
    onCredentialChanged?.call(credential);
  }

  /// 串行写入同一会话的凭据与资料，避免不同账号的数据交叉落盘。
  Future<void> _enqueuePersist(
    int revision,
    UserCredential credential,
    FxUser user,
  ) {
    final Future<void> operation = _storageQueue.then((_) async {
      if (revision != _storageRevision) return;
      try {
        await credentialStore.write(credential);
        if (revision != _storageRevision) return;
        await snapshotStore?.writeSnapshot(user);
      } catch (_) {
        // 持久化失败不影响当前运行时认证状态。
      }
    });
    _storageQueue = operation.catchError((Object _) {});
    return operation;
  }

  Future<void> _refreshUser() async {
    final credential = _credential;
    if (credential == null) return;
    _activate(credential, await repository.currentUser());
  }

  Future<void> _clear() async {
    _setCredential(null);
    final int revision = ++_storageRevision;
    final UserSnapshotStore? store = snapshotStore;
    final Future<void> operation = _storageQueue.then((_) async {
      if (revision != _storageRevision) return;
      await Future.wait([
        credentialStore.clear(),
        if (store != null) store.clearSnapshot(),
      ]);
    });
    _storageQueue = operation.catchError((Object _) {});
    await operation;
  }

  /// 远端资料优先，本地快照仅补齐服务端响应中缺失的扩展字段。
  FxUser _mergeUser(FxUser? snapshot, FxUser remote) {
    if (snapshot == null || snapshot.id != remote.id) return remote;
    return remote.copyWith(
      profile: {...snapshot.profile, ...remote.profile},
    );
  }
}
