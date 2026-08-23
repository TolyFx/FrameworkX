sealed class AuthRequest {
  const AuthRequest();
  Map<String, dynamic> toJson();
}

final class VerificationCodeAuth extends AuthRequest {
  final String channel;
  final String identifier;
  final String code;

  const VerificationCodeAuth({
    required this.channel,
    required this.identifier,
    required this.code,
  });

  @override
  Map<String, dynamic> toJson() => {
        'type': channel,
        'identifier': identifier,
        'credential': code,
      };
}

final class PasswordAuth extends AuthRequest {
  final String identifier;
  final String password;

  const PasswordAuth({required this.identifier, required this.password});

  @override
  Map<String, dynamic> toJson() => {
        'type': 'password',
        'identifier': identifier,
        'credential': password,
      };
}

final class OAuthAuth extends AuthRequest {
  final String provider;
  final String code;

  const OAuthAuth({required this.provider, required this.code});

  @override
  Map<String, dynamic> toJson() => {
        'type': provider,
        'identifier': '',
        'credential': code,
      };
}
