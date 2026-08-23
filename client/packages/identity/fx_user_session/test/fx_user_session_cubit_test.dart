import 'package:fx_user_session/fx_user_session.dart';
import 'package:fx_user_core/fx_user_core.dart';
import 'package:flutter_test/flutter_test.dart';

final class _Identity implements FxIdentity {
  @override
  final String id;

  @override
  final String displayName;

  @override
  final Uri? avatar;
  final Map<Object, Object?> fields;

  const _Identity({
    required this.id,
    required this.displayName,
    required this.avatar,
    this.fields = const {},
  });

  @override
  T? read<T>(FxIdentityField<T> field) => fields[field] as T?;
}

final class _UserCodec implements FxIdentityCodec<FxUser> {
  @override
  FxIdentity decode(FxUser user) => _Identity(
        id: user.id,
        displayName: user.displayName,
        avatar: user.avatar,
      );
}

final class _CredentialStore implements AuthCredentialStore {
  UserCredential? value;

  _CredentialStore({this.value});

  @override
  Future<void> clear() async => value = null;
  @override
  Future<UserCredential?> read() async => value;
  @override
  Future<void> write(UserCredential credential) async => value = credential;
}

final class _SnapshotStore implements UserSnapshotStore {
  FxUser? value;

  _SnapshotStore({this.value});

  @override
  Future<void> clearSnapshot() async => value = null;

  @override
  Future<FxUser?> readSnapshot() async => value;

  @override
  Future<void> writeSnapshot(FxUser user) async => value = user;
}

class _Repository implements FxUserRepository {
  @override
  Future<AuthResult> authenticate(AuthRequest request) =>
      throw UnimplementedError();
  @override
  Future<void> cancelScan(String token) => throw UnimplementedError();
  @override
  Future<bool> changePassword(
          {required String oldPassword, required String newPassword}) =>
      throw UnimplementedError();
  @override
  Future<void> confirmScan(String token, String action) =>
      throw UnimplementedError();
  @override
  Future<FxScanSession> createScanSession() => throw UnimplementedError();
  @override
  Future<FxUser> currentUser() => throw UnimplementedError();
  @override
  Future<void> deleteAccount(String password) => throw UnimplementedError();
  @override
  Future<bool> logout() => throw UnimplementedError();
  @override
  Future<String?> requestCode(
          {required String channel, required String identifier}) =>
      throw UnimplementedError();
  @override
  Future<FxScanStatus> scanStatus(String token) => throw UnimplementedError();
  @override
  Future<bool> setPassword(String newPassword) => throw UnimplementedError();
  @override
  Future<FxUser> updateProfile(UserProfilePatch patch) =>
      throw UnimplementedError();
}

final class _RestoreRepository extends _Repository {
  /// 服务端恢复时返回的用户资料。
  final FxUser user;

  _RestoreRepository(this.user);

  @override
  Future<FxUser> currentUser() async => user;
}

final class _RefreshRepository extends _Repository {
  /// 当前模拟的服务端用户资料。
  FxUser user;

  /// 用户资料接口调用次数。
  int callCount = 0;

  _RefreshRepository(this.user);

  @override
  Future<FxUser> currentUser() async {
    callCount++;
    return user;
  }
}

FxUserSessionCubit _cubit() => FxUserSessionCubit(
      repository: _Repository(),
      credentialStore: _CredentialStore(),
      identityCodec: _UserCodec(),
    );

void main() {
  test('starts while authentication is being established', () async {
    final cubit = _cubit();

    expect(cubit.state, isA<FxAuthing>());

    await cubit.close();
  });

  test('publishes guest and authenticated sessions', () async {
    final cubit = _cubit();
    final states = <FxUserSession>[];
    final subscription = cubit.stream.listen(states.add);

    cubit.guest();
    final identity = _Identity(
      id: '7',
      displayName: 'Toly',
      avatar: Uri.parse('https://example.com/avatar.png'),
    );
    cubit.authed(identity);

    await Future<void>.delayed(Duration.zero);

    expect(states[0], isA<FxGuest>());
    expect(states[1], isA<FxAuthed>());
    expect((states[1] as FxAuthed).user.id, '7');

    await subscription.cancel();
    await cubit.close();
  });

  test('exposes typed public extension fields', () {
    const identity = _Identity(
      id: '7',
      displayName: 'Toly',
      avatar: null,
      fields: {FxIdentityFields.signature: '保持好奇'},
    );

    expect(identity.read(FxIdentityFields.signature), '保持好奇');
  });

  test('restore merges missing remote profile fields from snapshot', () async {
    final _CredentialStore credentialStore = _CredentialStore(
      value: const BearerCredential(accessToken: 'token'),
    );
    final _SnapshotStore snapshotStore = _SnapshotStore(
      value: const FxUser(
        id: '7',
        displayName: 'Toly',
        profile: {'email': 'user@example.com'},
      ),
    );
    final FxUserSessionCubit cubit = FxUserSessionCubit(
      repository: _RestoreRepository(
        const FxUser(
          id: '7',
          displayName: 'Toly',
          profile: {'signature': '保持好奇'},
        ),
      ),
      credentialStore: credentialStore,
      snapshotStore: snapshotStore,
      identityCodec: _UserCodec(),
    );

    expect(await cubit.restore(), isTrue);
    await Future<void>.delayed(Duration.zero);
    expect(snapshotStore.value?.profile, {
      'email': 'user@example.com',
      'signature': '保持好奇',
    });

    await cubit.close();
  });

  test('restore replaces a snapshot that belongs to another account', () async {
    final _CredentialStore credentialStore = _CredentialStore(
      value: const BearerCredential(accessToken: 'account-20-token'),
    );
    final _SnapshotStore snapshotStore = _SnapshotStore(
      value: const FxUser(id: '17', displayName: 'Account 17'),
    );
    final FxUserSessionCubit cubit = FxUserSessionCubit(
      repository: _RestoreRepository(
        const FxUser(id: '20', displayName: 'Account 20'),
      ),
      credentialStore: credentialStore,
      snapshotStore: snapshotStore,
      identityCodec: _UserCodec(),
    );

    expect(await cubit.restore(), isTrue);
    await Future<void>.delayed(Duration.zero);

    expect((cubit.state as FxAuthed).user.id, '20');
    expect(snapshotStore.value?.id, '20');

    await cubit.close();
  });

  test('refreshCurrentUser publishes the latest remote profile', () async {
    final _CredentialStore credentialStore = _CredentialStore(
      value: const BearerCredential(accessToken: 'account-20-token'),
    );
    final _RefreshRepository repository = _RefreshRepository(
      const FxUser(id: '20', displayName: 'Old name'),
    );
    final FxUserSessionCubit cubit = FxUserSessionCubit(
      repository: repository,
      credentialStore: credentialStore,
      identityCodec: _UserCodec(),
    );
    await cubit.restore();
    repository.user = const FxUser(id: '20', displayName: 'New name');

    expect(await cubit.refreshCurrentUser(), isTrue);
    expect((cubit.state as FxAuthed).user.displayName, 'New name');
    expect(repository.callCount, 2);

    await cubit.close();
  });
}
