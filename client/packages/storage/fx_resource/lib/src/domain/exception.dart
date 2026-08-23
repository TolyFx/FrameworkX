import 'package:fx_exception/fx_exception.dart';

/// 资源模块统一错误码；数值仅在资源领域内表达稳定语义。
enum FxResourceCode with Code {
  /// 无法取得或创建资源根目录。
  rootUnavailable(1001),

  /// 引用格式不符合资源协议。
  invalidReference(1002),

  /// 受管理资源路径包含不安全的上级目录。
  unsafeRelativePath(1003),

  /// 平台文件选择器调用失败。
  pickFailed(1004),

  /// 组件树中没有注入资源依赖。
  scopeUnavailable(1005),

  /// 旧资源路径迁移过程中发生未知错误。
  migrationFailed(1006),

  /// 路径解析或文件存在性检查失败。
  storageFailed(1007),
  ;

  /// 错误码数值。
  @override
  final int code;

  /// 创建资源错误码。
  const FxResourceCode(this.code);
}

/// 资源模块对外抛出的结构化异常。
final class FxResourceException extends FxException<FxResourceCode> {
  /// 与错误直接相关的资源引用；没有具体引用时为 null。
  final String? reference;

  /// 创建资源模块异常。
  const FxResourceException(
    super.code,
    super.message, [
    super.error,
    super.stack,
    this.reference,
  ]);
}
