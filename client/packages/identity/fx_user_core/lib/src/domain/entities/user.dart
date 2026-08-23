final class FxUser {
  final String id;
  final String displayName;
  final Uri? avatar;
  final Map<String, dynamic> profile;

  const FxUser({
    required this.id,
    required this.displayName,
    this.avatar,
    this.profile = const {},
  });

  factory FxUser.fromJson(Map<String, dynamic> json) {
    final String? avatar = json['avatar'] as String?;
    return FxUser(
      id: '${json['id'] ?? json['user_id']}',
      displayName: (json['display_name'] ?? json['nickname'] ?? '') as String,
      avatar: avatar == null || avatar.isEmpty ? null : Uri.tryParse(avatar),
      profile: Map<String, dynamic>.from(json['profile'] as Map? ?? const {}),
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'display_name': displayName,
        if (avatar != null) 'avatar': avatar.toString(),
        'profile': profile,
      };

  FxUser copyWith({
    String? id,
    String? displayName,
    Uri? avatar,
    Map<String, dynamic>? profile,
  }) {
    return FxUser(
      id: id ?? this.id,
      displayName: displayName ?? this.displayName,
      avatar: avatar ?? this.avatar,
      profile: profile ?? this.profile,
    );
  }
}
