import 'dart:io';

import 'package:device_info_plus/device_info_plus.dart';
import 'package:flutter/foundation.dart';
import 'package:package_info_plus/package_info_plus.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:uuid/uuid.dart';

/// 收集 FrameworkX 登录审计所需的标准设备信息。
final class FxUserDeviceInfoCollector {
  /// 产品级存储命名空间。
  final String namespace;

  /// 设备信息平台入口。
  final DeviceInfoPlugin deviceInfo;

  /// 异步配置存储入口。
  final SharedPreferencesAsync preferences;

  FxUserDeviceInfoCollector({
    required this.namespace,
    DeviceInfoPlugin? deviceInfo,
    SharedPreferencesAsync? preferences,
  }) : deviceInfo = deviceInfo ?? DeviceInfoPlugin(),
       preferences = preferences ?? SharedPreferencesAsync();

  /// 收集平台、设备名、安装标识和应用版本。
  Future<Map<String, dynamic>> collect() async {
    String platform = 'web';
    String? deviceName;
    if (kIsWeb) {
      deviceName = (await deviceInfo.webBrowserInfo).browserName.name;
    } else if (Platform.isAndroid) {
      platform = 'android';
      deviceName = (await deviceInfo.androidInfo).model;
    } else if (Platform.isIOS) {
      platform = 'ios';
      deviceName = (await deviceInfo.iosInfo).name;
    } else if (Platform.isMacOS) {
      platform = 'macos';
      deviceName = (await deviceInfo.macOsInfo).computerName;
    } else if (Platform.isWindows) {
      platform = 'windows';
      deviceName = (await deviceInfo.windowsInfo).computerName;
    } else if (Platform.isLinux) {
      platform = 'linux';
      deviceName = (await deviceInfo.linuxInfo).prettyName;
    }
    final PackageInfo package = await PackageInfo.fromPlatform();
    return <String, dynamic>{
      'platform': platform,
      'device_name': deviceName,
      'device_id': await installationId(),
      'app_version': package.version,
    };
  }

  /// 返回当前产品安装稳定使用的本机标识。
  Future<String> installationId() async {
    final String key = '$namespace.user.device_id';
    String? deviceId = await preferences.getString(key);
    if (deviceId != null && deviceId.isNotEmpty) return deviceId;
    deviceId = const Uuid().v4();
    await preferences.setString(key, deviceId);
    return deviceId;
  }
}
