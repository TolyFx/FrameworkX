import '../auth/auth_repository.dart';
import '../profile/profile_repository.dart';

abstract interface class FxUserRepository
    implements FxAuthRepository, FxProfileRepository {}
