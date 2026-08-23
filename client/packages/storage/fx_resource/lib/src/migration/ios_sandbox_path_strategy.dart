import 'legacy_path_strategy.dart';

/// 识别旧版 iOS 模拟器或真机沙盒中的 Documents 绝对路径。
final class FxIosSandboxPathStrategy extends FxLegacyPathStrategy {
  /// 创建 iOS 沙盒路径识别策略。
  const FxIosSandboxPathStrategy();

  @override
  String? relativePathOf(String path) {
    const String applicationMarker = '/Containers/Data/Application/';
    const String documentsMarker = '/Documents/';
    final int applicationIndex = path.indexOf(applicationMarker);
    final int documentsIndex = path.indexOf(documentsMarker);
    if (applicationIndex < 0 || documentsIndex < applicationIndex) {
      return null;
    }
    final String relativePath =
        path.substring(documentsIndex + documentsMarker.length);
    return relativePath.isEmpty ? null : relativePath;
  }
}
