import 'package:flutter/material.dart';
import 'package:google_sign_in/google_sign_in.dart';
import 'package:localink_client/src/helpers/API.dart';
import 'package:localink_client/src/widgets/homepage.dart';
import 'package:localink_client/src/widgets/landing_screen.dart';
import 'package:localink_client/src/models/auth.dart';
import 'package:flutter_dotenv/flutter_dotenv.dart';
import 'package:provider/provider.dart';

Future<void> main() async {
  await dotenv.load(fileName: ".env");

  runApp(const MyApp());
}

const List<String> scopes = <String>[
  'openid',
  'https://www.googleapis.com/auth/userinfo.email',
  'https://www.googleapis.com/auth/userinfo.profile',
];

// The logic is really stupid here: we need an android ID client in the Google identity platform to exist to avoid an APIException error,
// but the clientId must refer to a Web ID client to actually work and return the idToken we need.
GoogleSignIn googleSignIn = GoogleSignIn(
  clientId: dotenv.env['GOOGLE_CLIENT_ID']!,
  // web
  scopes: scopes,
);

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // todo request gps permission
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Localink',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const App(),
    );
  }
}

class App extends StatefulWidget {
  const App({super.key});

  @override
  State<App> createState() => _AppState();
}

class _AppState extends State<App> {
  GoogleSignInAccount? _currentGoogleUser;
  AuthResponse? _authResponse;

  @override
  void initState() {
    super.initState();

    googleSignIn.onCurrentUserChanged
        .listen((GoogleSignInAccount? account) async {
      if (account == null) {
        setState(() {
          _currentGoogleUser = null;
          _authResponse = null;
        });
        return; // this is a logout
      }

      final googleSignInAuthentication = await account?.authentication;
      try {
        final authResponse =
            await API().auth(googleSignInAuthentication!.idToken!);
        setState(() {
          _currentGoogleUser = account;
          _authResponse = authResponse;
        });
      } catch (e) {
        throw Exception('Failed to load profile: $e');
      }
    });

    // In the web, _googleSignIn.signInSilently() triggers the One Tap UX.
    //
    // It is recommended by Google Identity Services to render both the One Tap UX
    // and the Google Sign In button together to "reduce friction and improve
    // sign-in rates" ([docs](https://developers.google.com/identity/gsi/web/guides/display-button#html)).
    googleSignIn.signInSilently();
  }

  @override
  Widget build(BuildContext context) {
    final GoogleSignInAccount? user = _currentGoogleUser;
    if (user != null && _authResponse != null) {
      return Scaffold(
          appBar: AppBar(
            backgroundColor: Theme.of(context).colorScheme.background,
            title: Text("Homepage"),
          ),
          body: ConstrainedBox(
            constraints: const BoxConstraints.expand(),
            child: ChangeNotifierProvider(
              create: (context) => _authResponse,
              child: Homepage(user: user, googleSignIn: googleSignIn),
            ),
          ));
    } else {
      return Scaffold(body: LandingPage(googleSignIn: googleSignIn));
    }
  }
}
