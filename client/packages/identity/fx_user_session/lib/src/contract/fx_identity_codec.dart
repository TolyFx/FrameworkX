import '../domain/fx_identity.dart';

/// 将宿主自己的用户对象投影为跨模块可见的 [FxIdentity]。
///
/// 宿主只需实现一次；功能模块始终只读取 [FxIdentity]，无需依赖宿主用户模型。
abstract interface class FxIdentityCodec<TUser> {
  FxIdentity decode(TUser user);
}
