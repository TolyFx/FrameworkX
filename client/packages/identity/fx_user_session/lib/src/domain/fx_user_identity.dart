import 'package:fx_user_core/fx_user_core.dart';

import '../contract/fx_identity_codec.dart';
import 'fx_identity.dart';

/// FrameworkX 用户模型的标准公开身份投影。
final class FxUserIdentity implements FxIdentity {
  @override
  final String id;

  @override
  final String? displayName;

  @override
  final Uri? avatar;

  /// 可供跨模块读取的标准公开资料字段。
  final Map<Object, Object?> fields;

  const FxUserIdentity({
    required this.id,
    required this.displayName,
    required this.avatar,
    required this.fields,
  });

  @override
  T? read<T>(FxIdentityField<T> field) => fields[field] as T?;
}

/// 将 FrameworkX 用户模型转换为标准公开身份。
final class FxUserIdentityCodec implements FxIdentityCodec<FxUser> {
  const FxUserIdentityCodec();

  @override
  FxIdentity decode(FxUser user) {
    return FxUserIdentity(
      id: user.id,
      displayName: user.displayName,
      avatar: user.avatar,
      fields: <Object, Object?>{
        FxIdentityFields.signature: user.profile['signature'] as String?,
        FxIdentityFields.email: user.profile['email'] as String?,
        FxIdentityFields.phone: user.profile['phone'] as String?,
        FxIdentityFields.hasPassword:
            user.profile['has_password'] as bool? ?? false,
      },
    );
  }
}
