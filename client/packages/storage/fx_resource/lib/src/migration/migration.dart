import '../domain/exception.dart';
import '../domain/reference.dart';
import '../storage/store.dart';
import 'ios_sandbox_path_strategy.dart';
import 'legacy_path_strategy.dart';

/// 将旧版不稳定沙盒绝对路径迁移为稳定的 `resources:` 引用。
final class FxResourceMigration {
  /// 用于验证目标资源是否仍在当前设备存在的存储。
  final FxResourceStore _store;

  /// 按顺序尝试识别历史路径的策略集合。
  final List<FxLegacyPathStrategy> _strategies;

  /// 创建本地资源迁移器。
  const FxResourceMigration(
    this._store, {
    List<FxLegacyPathStrategy> strategies = const <FxLegacyPathStrategy>[
      FxIosSandboxPathStrategy(),
    ],
  }) : _strategies = strategies;

  /// 迁移单个引用；无法可靠映射时保留原值，绝不修改同步内容。
  String migrate(String reference) {
    if (reference.trim().isEmpty) {
      return reference;
    }
    try {
      final FxResourceRef parsedReference = FxResourceRef.parse(reference);
      if (parsedReference is! FxLocalResourceRef) {
        return reference;
      }
      final String? relativePath = _resolveLegacyRelativePath(reference);
      if (relativePath == null) {
        return reference;
      }
      final String managedReference =
          FxManagedResourceRef(relativePath).rawValue;
      return _store.exists(managedReference) ? managedReference : reference;
    } on FxResourceException {
      // 已有资源引用即使格式不完整，也不能阻断宿主对同步文件的物化。
      // 迁移阶段只负责尽力转换旧路径，无法转换时应保留原始同步数据。
      return reference;
    } catch (error, stackTrace) {
      throw FxResourceException(
        FxResourceCode.migrationFailed,
        'Failed to migrate legacy resource reference',
        error,
        stackTrace,
        reference,
      );
    }
  }

  /// 依次交给策略识别历史路径，优先使用最先匹配的结果。
  String? _resolveLegacyRelativePath(String path) {
    for (final FxLegacyPathStrategy strategy in _strategies) {
      final String? relativePath = strategy.relativePathOf(path);
      if (relativePath != null) {
        return relativePath;
      }
    }
    return null;
  }
}
