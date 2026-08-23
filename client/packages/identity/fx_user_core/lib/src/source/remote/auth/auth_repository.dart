import '../../../domain/domain.dart';

abstract interface class FxAuthRepository {
  Future<String?> requestCode(
      {required String channel, required String identifier});
  Future<AuthResult> authenticate(AuthRequest request);
  Future<FxScanSession> createScanSession();
  Future<FxScanStatus> scanStatus(String token);
  Future<void> confirmScan(String token, String action);
  Future<void> cancelScan(String token);
  Future<bool> logout();
}
