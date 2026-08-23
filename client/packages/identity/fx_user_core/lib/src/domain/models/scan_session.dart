final class FxScanSession {
  final String token;
  final String content;

  const FxScanSession({required this.token, required this.content});
}

final class FxScanStatus {
  final String status;
  final String? credential;

  const FxScanStatus({required this.status, this.credential});
}
