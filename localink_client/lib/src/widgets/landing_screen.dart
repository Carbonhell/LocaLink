import 'package:flutter/material.dart';
import 'package:flutter_signin_button/button_list.dart';
import 'package:flutter_signin_button/button_view.dart';
import 'package:flutter_svg/svg.dart';
import 'package:google_sign_in/google_sign_in.dart';

class LandingPage extends StatelessWidget {
  final GoogleSignIn googleSignIn;

  const LandingPage({super.key, required this.googleSignIn});

  @override
  Widget build(BuildContext context) {

    return Container(
      margin: const EdgeInsets.only(top: 100.0, left: 20.0, bottom: 50.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Container(
                child: const Text(
                  'LocaLink',
                  style: TextStyle(fontWeight: FontWeight.bold, fontSize: 30),
                ),
              ),
              Container(
                child: const Text(
                  'Welcome to a social network where the local community is the focus.',
                  style: TextStyle(color: Color(0xC9C9C9FF), fontSize: 20),
                ),
              ),
            ],
          ),
          Center(
              child: SvgPicture.asset('assets/svg/login-banner.svg',
                  semanticsLabel: 'LocaLink logo')),
          Center(
            child: SignInButton(
              Buttons.Google,
              onPressed: () async {
                try {
                  await googleSignIn.signIn();
                } catch (error) {
                  print(error);
                }
              },
            ),
          )
        ],
      ),
    );
  }
}
