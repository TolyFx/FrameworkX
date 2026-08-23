/// 可跨模块安全使用的用户公开摘要。
///
/// 固定字段承载高频的公开资料；业务需要共享、但不适合固化为字段的公开信息
/// 可通过 [FxIdentityField] 按类型读取。完整资料、认证凭据和私有业务字段不属于
/// 此协议。
abstract interface class FxIdentity {
  String get id;

  String? get displayName;

  Uri? get avatar;

  /// 读取宿主已声明的公开扩展字段；未声明或不存在时返回 `null`。
  T? read<T>(FxIdentityField<T> field);
}

/// 用户公开扩展字段的类型化标识。
///
/// 跨模块使用的字段标识应由共享模块统一声明，宿主只负责在 identity codec 中
/// 映射一次，业务模块不应依赖宿主的具体用户模型。
final class FxIdentityField<T> {
  final String name;

  const FxIdentityField(this.name);
}

/// Fx 应用通用的公开用户资料字段。
abstract final class FxIdentityFields {
  static const signature = FxIdentityField<String>('profile.signature');
}
