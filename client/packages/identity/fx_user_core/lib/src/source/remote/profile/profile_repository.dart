import '../../../domain/domain.dart';

abstract interface class FxProfileRepository {
  Future<FxUser> currentUser();
  Future<FxUser> updateProfile(UserProfilePatch patch);
  Future<bool> setPassword(String newPassword);
  Future<bool> changePassword(
      {required String oldPassword, required String newPassword});
  Future<void> deleteAccount(String password);
}
