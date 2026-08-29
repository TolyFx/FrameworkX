import '../../../domain/domain.dart';

abstract interface class FxProfileRepository {
  Future<FxUser> currentUser();
  Future<FxUser> updateProfile(UserProfilePatch patch);
  Future<FxAccountCheckResult> checkAccount({
    required String type,
    required String identifier,
  });
  Future<FxUser> bindEmail({required String email, required String code});
  Future<FxUser> bindPhone({required String phone, required String code});
  Future<bool> setPassword(String newPassword);
  Future<bool> changePassword({
    required String oldPassword,
    required String newPassword,
  });
  Future<void> deleteAccount(String password);
}
