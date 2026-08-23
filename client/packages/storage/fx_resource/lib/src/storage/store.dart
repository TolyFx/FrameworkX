import '../domain/reference.dart';

/// 本地资源根目录与引用解析的抽象；宿主可替换其存储策略。
abstract interface class FxResourceStore {
  /// 应用受管理资源所在的根目录。
  String get rootPath;

  /// 将当前机器上的路径转换成稳定资源引用。
  FxResourceRef referenceForPath(String path);

  /// 解析引用为当前设备可访问的路径或网络地址。
  String resolvePath(String reference);

  /// 判断本地引用当前是否可访问；网络资源始终返回 false。
  bool exists(String reference);
}

/// 将用户选中的外部文件纳入应用资源目录的可选能力。
///
/// 选择器检测到该能力时，会在授权仍有效的当次选择中完成复制，避免
/// macOS 沙盒在下次启动后失去外部文件的读取权限。
abstract interface class FxResourceImportStore {
  /// 导入一个外部本地文件，并返回应用托管的稳定资源引用。
  Future<FxManagedResourceRef> importFile(String sourcePath);
}
