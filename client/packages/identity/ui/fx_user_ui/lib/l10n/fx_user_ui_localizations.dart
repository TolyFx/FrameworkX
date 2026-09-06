import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'fx_user_ui_localizations_en.dart';
import 'fx_user_ui_localizations_zh.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of FxUserUiLocalizations
/// returned by `FxUserUiLocalizations.of(context)`.
///
/// Applications need to include `FxUserUiLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/fx_user_ui_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: FxUserUiLocalizations.localizationsDelegates,
///   supportedLocales: FxUserUiLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the FxUserUiLocalizations.supportedLocales
/// property.
abstract class FxUserUiLocalizations {
  FxUserUiLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static FxUserUiLocalizations? of(BuildContext context) {
    return Localizations.of<FxUserUiLocalizations>(
      context,
      FxUserUiLocalizations,
    );
  }

  static const LocalizationsDelegate<FxUserUiLocalizations> delegate =
      _FxUserUiLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('zh'),
  ];

  /// No description provided for @close.
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get close;

  /// No description provided for @scanUnavailable.
  ///
  /// In en, this message translates to:
  /// **'QR login unavailable'**
  String get scanUnavailable;

  /// No description provided for @acceptAgreements.
  ///
  /// In en, this message translates to:
  /// **'Accept the terms first'**
  String get acceptAgreements;

  /// No description provided for @loginMethodUnavailable.
  ///
  /// In en, this message translates to:
  /// **'Login method unavailable'**
  String get loginMethodUnavailable;

  /// No description provided for @emailLogin.
  ///
  /// In en, this message translates to:
  /// **'Email'**
  String get emailLogin;

  /// No description provided for @phoneLogin.
  ///
  /// In en, this message translates to:
  /// **'Phone'**
  String get phoneLogin;

  /// No description provided for @passwordLogin.
  ///
  /// In en, this message translates to:
  /// **'Password'**
  String get passwordLogin;

  /// No description provided for @scanLogin.
  ///
  /// In en, this message translates to:
  /// **'QR Code'**
  String get scanLogin;

  /// No description provided for @account.
  ///
  /// In en, this message translates to:
  /// **'Account'**
  String get account;

  /// No description provided for @email.
  ///
  /// In en, this message translates to:
  /// **'Email'**
  String get email;

  /// No description provided for @phone.
  ///
  /// In en, this message translates to:
  /// **'Phone'**
  String get phone;

  /// No description provided for @verificationCode.
  ///
  /// In en, this message translates to:
  /// **'Code'**
  String get verificationCode;

  /// No description provided for @password.
  ///
  /// In en, this message translates to:
  /// **'Password'**
  String get password;

  /// No description provided for @emailHint.
  ///
  /// In en, this message translates to:
  /// **'Enter email'**
  String get emailHint;

  /// No description provided for @phoneHint.
  ///
  /// In en, this message translates to:
  /// **'Enter phone'**
  String get phoneHint;

  /// No description provided for @accountHint.
  ///
  /// In en, this message translates to:
  /// **'ID, phone, or email'**
  String get accountHint;

  /// No description provided for @codeHint.
  ///
  /// In en, this message translates to:
  /// **'Enter code'**
  String get codeHint;

  /// No description provided for @passwordHint.
  ///
  /// In en, this message translates to:
  /// **'Enter password'**
  String get passwordHint;

  /// No description provided for @getCode.
  ///
  /// In en, this message translates to:
  /// **'Get Code'**
  String get getCode;

  /// No description provided for @login.
  ///
  /// In en, this message translates to:
  /// **'Log In'**
  String get login;

  /// No description provided for @agreementPrefix.
  ///
  /// In en, this message translates to:
  /// **'By logging in, you agree to '**
  String get agreementPrefix;

  /// No description provided for @userAgreement.
  ///
  /// In en, this message translates to:
  /// **'Terms'**
  String get userAgreement;

  /// No description provided for @agreementAnd.
  ///
  /// In en, this message translates to:
  /// **' and '**
  String get agreementAnd;

  /// No description provided for @privacyPolicy.
  ///
  /// In en, this message translates to:
  /// **'Privacy Policy'**
  String get privacyPolicy;

  /// No description provided for @agreementSuffix.
  ///
  /// In en, this message translates to:
  /// **'. New accounts are created automatically.'**
  String get agreementSuffix;

  /// No description provided for @otherLoginMethods.
  ///
  /// In en, this message translates to:
  /// **'Other ways'**
  String get otherLoginMethods;

  /// No description provided for @codeLogin.
  ///
  /// In en, this message translates to:
  /// **'Code'**
  String get codeLogin;

  /// No description provided for @reloadQrCode.
  ///
  /// In en, this message translates to:
  /// **'Reload QR code'**
  String get reloadQrCode;

  /// No description provided for @qrCodeExpired.
  ///
  /// In en, this message translates to:
  /// **'Expired. Tap to refresh'**
  String get qrCodeExpired;

  /// No description provided for @scanConfirmed.
  ///
  /// In en, this message translates to:
  /// **'Scanned. Confirm on your phone'**
  String get scanConfirmed;

  /// No description provided for @scanQrCodeHint.
  ///
  /// In en, this message translates to:
  /// **'Scan with a signed-in phone'**
  String get scanQrCodeHint;
}

class _FxUserUiLocalizationsDelegate
    extends LocalizationsDelegate<FxUserUiLocalizations> {
  const _FxUserUiLocalizationsDelegate();

  @override
  Future<FxUserUiLocalizations> load(Locale locale) {
    return SynchronousFuture<FxUserUiLocalizations>(
      lookupFxUserUiLocalizations(locale),
    );
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'zh'].contains(locale.languageCode);

  @override
  bool shouldReload(_FxUserUiLocalizationsDelegate old) => false;
}

FxUserUiLocalizations lookupFxUserUiLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return FxUserUiLocalizationsEn();
    case 'zh':
      return FxUserUiLocalizationsZh();
  }

  throw FlutterError(
    'FxUserUiLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
