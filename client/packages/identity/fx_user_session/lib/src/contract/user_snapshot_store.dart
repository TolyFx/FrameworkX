import 'package:fx_user_core/fx_user_core.dart';

/// 可恢复用户资料快照的宿主存储。
///
/// 快照只用于启动恢复和远端字段缺失时的降级，服务端资料仍是最终事实。
abstract interface class UserSnapshotStore {
  /// 读取最近一次完整用户资料。
  Future<FxUser?> readSnapshot();

  /// 保存最近一次完整用户资料。
  Future<void> writeSnapshot(FxUser user);

  /// 清除用户资料快照。
  Future<void> clearSnapshot();
}
