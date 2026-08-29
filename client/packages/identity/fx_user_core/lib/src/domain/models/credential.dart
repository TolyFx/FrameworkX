sealed class UserCredential {
  const UserCredential();
  Map<String, dynamic> toJson();

  factory UserCredential.fromJson(Map<String, dynamic> json) =>
      switch (json['type']) {
        'bearer' => BearerCredential(
          accessToken: json['access_token'] as String,
          refreshToken: json['refresh_token'] as String?,
        ),
        'cookie' => const CookieCredential(),
        _ => throw FormatException(
          'Unsupported credential type: ${json['type']}',
        ),
      };
}

final class BearerCredential extends UserCredential {
  final String accessToken;
  final String? refreshToken;

  const BearerCredential({required this.accessToken, this.refreshToken});

  @override
  Map<String, dynamic> toJson() => {
    'type': 'bearer',
    'access_token': accessToken,
    if (refreshToken != null) 'refresh_token': refreshToken,
  };
}

final class CookieCredential extends UserCredential {
  const CookieCredential();

  @override
  Map<String, dynamic> toJson() => const {'type': 'cookie'};
}
