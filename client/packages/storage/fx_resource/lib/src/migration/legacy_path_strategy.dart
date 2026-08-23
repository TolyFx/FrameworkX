/// 从历史本地路径中提取相对于当前应用资源根目录的路径。
abstract class FxLegacyPathStrategy {
  /// 创建历史路径识别策略。
  const FxLegacyPathStrategy();

  /// 若当前策略匹配，则返回资源相对路径；否则返回 null。
  String? relativePathOf(String path);
}
