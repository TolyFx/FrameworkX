import 'dart:typed_data';

import 'package:fx_user_core/fx_user_core.dart';

/// 宿主提供的头像上传任务。
///
/// 会话层只编排认证凭据、资料更新与状态刷新；文件存储平台、上传协议及
/// CDN 地址生成均由宿主实现。
abstract interface class FxAvatarUploadTask {
  Future<Uri> upload({
    required Uint8List bytes,
    required UserCredential credential,
  });
}
