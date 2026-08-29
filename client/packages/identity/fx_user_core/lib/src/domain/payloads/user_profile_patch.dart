final class UserProfilePatch {
  final String? displayName;
  final Uri? avatar;
  final Map<String, dynamic> fields;

  const UserProfilePatch({
    this.displayName,
    this.avatar,
    this.fields = const {},
  });

  Map<String, dynamic> toJson() => {
    if (displayName != null) 'display_name': displayName,
    if (avatar != null) 'avatar': avatar.toString(),
    ...fields,
  };
}
