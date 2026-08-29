import 'package:fx_dio/fx_dio.dart';
import 'package:fx_user_core/fx_user_core.dart';

/// 宿主可在解码用户资料时补充产品级字段转换。
typedef FxUserDecoder = FxUser Function(Map<String, dynamic> json);

/// 基于 [RequestHost] 的 Fx 用户远端仓储实现。
class HttpFxUserRepository implements FxUserRepository {
  /// 远端请求宿主。
  final RequestHost host;

  /// 登录请求携带的设备信息。
  final Map<String, dynamic> deviceInfo;

  /// 用户资料解码器。
  final FxUserDecoder userDecoder;

  HttpFxUserRepository({
    required this.host,
    this.deviceInfo = const {},
    FxUserDecoder? userDecoder,
  }) : userDecoder = userDecoder ?? FxUser.fromJson;

  @override
  Future<String?> requestCode({
    required String channel,
    required String identifier,
    FxVerificationCodeScene scene = FxVerificationCodeScene.login,
  }) async {
    final ApiRet<String?> result = await host.post<String?>(
      '/auth/code',
      data: {
        'channel': channel,
        'identifier': identifier,
        'scene': scene.value,
      },
      convertor: _decodeCode,
    );
    return _unwrap(result);
  }

  @override
  Future<void> resetPassword({
    required String email,
    required String code,
    required String newPassword,
  }) async {
    final ApiRet<void> result = await host.post<void>(
      '/auth/password/reset',
      data: {'email': email, 'code': code, 'new_password': newPassword},
      convertor: _decodeVoid,
    );
    _unwrap(result);
  }

  @override
  Future<AuthResult> authenticate(AuthRequest request) async {
    final ApiRet<AuthResult> result = await host.post<AuthResult>(
      '/auth/login',
      data: {
        ...request.toJson(),
        if (deviceInfo.isNotEmpty) 'device_info': deviceInfo,
      },
      convertor: _decodeAuthEnvelope,
    );
    return _unwrap(result);
  }

  @override
  Future<bool> logout() async {
    final ApiRet<void> result = await host.post<void>(
      '/auth/logout',
      convertor: _decodeVoid,
    );
    return result.success;
  }

  @override
  Future<FxUser> currentUser() async {
    final ApiRet<dynamic> result = await host.get<dynamic>(
      '/user/profile',
      convertor: _responseData,
    );
    return _userFrom(_unwrap(result));
  }

  @override
  Future<FxUser> updateProfile(UserProfilePatch patch) async {
    final ApiRet<dynamic> result = await host.put<dynamic>(
      '/user/profile',
      data: patch.toJson(),
      convertor: _responseData,
    );
    return _userFrom(_unwrap(result));
  }

  @override
  Future<FxAccountCheckResult> checkAccount({
    required String type,
    required String identifier,
  }) async {
    final ApiRet<dynamic> result = await host.post<dynamic>(
      '/user/account/check',
      data: {'type': type, 'identifier': identifier},
      convertor: _responseData,
    );
    return FxAccountCheckResult.fromJson(_asMap(_unwrap(result)));
  }

  @override
  Future<FxUser> bindEmail({
    required String email,
    required String code,
  }) async {
    final ApiRet<dynamic> result = await host.put<dynamic>(
      '/user/email',
      data: {'email': email, 'code': code},
      convertor: _responseData,
    );
    return _userFrom(_unwrap(result));
  }

  @override
  Future<FxUser> bindPhone({
    required String phone,
    required String code,
  }) async {
    final ApiRet<dynamic> result = await host.put<dynamic>(
      '/user/phone',
      data: {'phone': phone, 'code': code},
      convertor: _responseData,
    );
    return _userFrom(_unwrap(result));
  }

  @override
  Future<bool> setPassword(String newPassword) async {
    final ApiRet<void> result = await host.post<void>(
      '/user/password',
      data: {'new_password': newPassword},
      convertor: _decodeVoid,
    );
    return result.success;
  }

  @override
  Future<bool> changePassword({
    required String oldPassword,
    required String newPassword,
  }) async {
    final ApiRet<void> result = await host.put<void>(
      '/user/password',
      data: {'old_password': oldPassword, 'new_password': newPassword},
      convertor: _decodeVoid,
    );
    return result.success;
  }

  @override
  Future<void> deleteAccount(String password) async {
    final ApiRet<void> result = await host.delete<void>(
      '/user/account',
      data: {'password': password},
      convertor: _decodeVoid,
    );
    _unwrap(result);
  }

  @override
  Future<FxScanSession> createScanSession() async {
    final ApiRet<dynamic> result = await host.post<dynamic>(
      '/auth/scan/create',
      convertor: _responseData,
    );
    final Map<String, dynamic> json = _asMap(_unwrap(result));
    return FxScanSession(
      token: json['token'] as String,
      content: json['qr_content'] as String,
    );
  }

  @override
  Future<FxScanStatus> scanStatus(String token) async {
    final ApiRet<dynamic> result = await host.get<dynamic>(
      '/auth/scan/status',
      queryParameters: {'token': token},
      convertor: _responseData,
    );
    final Map<String, dynamic> json = _asMap(_unwrap(result));
    return FxScanStatus(
      status: json['status'] as String,
      credential: json['token'] as String?,
    );
  }

  @override
  Future<void> confirmScan(String token, String action) async {
    final ApiRet<void> result = await host.post<void>(
      '/auth/scan/confirm',
      data: {'scan_token': token, 'action': action},
      convertor: _decodeVoid,
    );
    _unwrap(result);
  }

  @override
  Future<void> cancelScan(String token) async {
    final ApiRet<void> result = await host.post<void>(
      '/auth/scan/cancel',
      data: {'scan_token': token},
      convertor: _decodeVoid,
    );
    _unwrap(result);
  }

  String? _decodeCode(dynamic response) {
    final Map<String, dynamic> data = _asMap(_responseData(response));
    return data['code'] as String?;
  }

  AuthResult _decodeAuthEnvelope(dynamic response) {
    return _decodeAuth(_responseData(response));
  }

  AuthResult _decodeAuth(dynamic data) {
    final Map<String, dynamic> json = _asMap(data);
    final dynamic token = json['token'] ?? json['access_token'];
    final dynamic userJson = json['user'];
    if (userJson == null) {
      throw const FormatException('Login response is missing user data.');
    }
    return AuthResult(
      credential: token == null
          ? const CookieCredential()
          : BearerCredential(
              accessToken: token as String,
              refreshToken: json['refresh_token'] as String?,
            ),
      user: _userFrom(userJson),
    );
  }

  FxUser _userFrom(dynamic data) => userDecoder(_asMap(data));
}

dynamic _responseData(dynamic response) {
  final Map<String, dynamic> envelope = _asMap(
    response,
    message: 'Invalid API response.',
  );
  if (envelope.containsKey('data')) {
    return envelope['data'];
  }
  return response;
}

Map<String, dynamic> _asMap(Object? data, {String? message}) {
  if (data is Map<String, dynamic>) return data;
  if (data is Map) return Map<String, dynamic>.from(data);
  throw FormatException(
    message ?? 'Expected a Map, but got ${data.runtimeType}.',
  );
}

void _decodeVoid(dynamic _) {}

T _unwrap<T>(ApiRet<T> result) {
  return switch (result) {
    ApiOK<T>(:final T t) => t,
    ApiFail<T>(:final Trace trace) => throw trace,
  };
}
