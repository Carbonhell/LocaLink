import 'package:flutter/material.dart';
import 'package:google_maps_flutter/google_maps_flutter.dart';
import 'package:google_sign_in/google_sign_in.dart';
import 'package:google_sign_in/widgets.dart';
import 'package:localink_client/src/models/auth.dart';
import 'package:localink_client/src/widgets/query_page.dart';
import 'package:localink_client/src/widgets/quoted_text.dart';
import 'package:provider/provider.dart';

import '../helpers/API.dart';
import '../helpers/utils.dart';
import 'map_page.dart';
import 'match_card.dart';

class Homepage extends StatefulWidget {
  const Homepage({super.key, required this.googleSignIn, required this.user});

  final String title = "Homepage";
  final GoogleSignIn googleSignIn;
  final GoogleSignInAccount user;

  @override
  State<Homepage> createState() => HomepageState();
}

class HomepageState extends State<Homepage> {
  bool isButtonDisabled = true;
  String? description;
  int lastTimestamp = 0;

  @override
  void initState() {
    super.initState();
  }

  Future<void> _handleSignOut() => widget.googleSignIn.disconnect();

  Future<void> _updateDescription(String token) async {
    if (description != null) {
      await API().generateEmbeddings(token, description!);
    }
  }

  Future<void> _syncPosition(String token) async {
    var currentTime = DateTime.now().millisecondsSinceEpoch;
    if (currentTime > 10000 + lastTimestamp) {
      lastTimestamp = currentTime;
      var position = await determinePosition();
      await API()
          .syncPosition(token, LatLng(position.latitude, position.longitude));
    }
  }

  showDescription(Match item) async {
    return () async {
      await showDialog<bool>(
        context: context,
        builder: (BuildContext context) => AlertDialog(
          title: Text('Match - ${item.userName}?'),
          content: Column(
            children: [
              QuotedText(
                text: item.userDescription,
              ),
            ],
          ),
          actions: <Widget>[
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: const Text('Return'),
            ),
          ],
        ),
      );
    };
  }

  @override
  Widget build(BuildContext context) {
    final ButtonStyle raisedButtonStyle = ElevatedButton.styleFrom(
      foregroundColor: Colors.black87,
      backgroundColor: Colors.grey[300],
      minimumSize: Size(88, 36),
      padding: EdgeInsets.symmetric(horizontal: 16),
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.all(Radius.circular(2)),
      ),
    );

    matchesList(AuthResponse authResponse) {
      if (authResponse.matches.isEmpty) {
        return const Text("No matches found.");
      }

      return ListView.builder(
        scrollDirection: Axis.vertical,
        shrinkWrap: true,
        itemCount: authResponse.matches.length,
        itemBuilder: (context, index) {
          final item = authResponse.matches[index];

          return MatchCard(
              onTap: item.matchStatus == MatchStatus.Accepted
                  ? () {
                      Navigator.push(
                        context,
                        MaterialPageRoute(
                            builder: (context) => MapPage(
                                authResponse: authResponse, match: item)),
                      );
                    }
                  : showDescription(item),
              updateItem: (newStatus) {
                setState(() {
                  item.matchStatus = newStatus;
                });
              },
              token: authResponse.accessToken,
              id: item.userID,
              name: item.userName,
              description: item.userDescription,
              status: item.matchStatus);
        },
      );
    }

    return Consumer<AuthResponse>(builder: (context, authResponse, child) {
      _syncPosition(authResponse.accessToken);
      final originalDescription = authResponse.description;
      return Container(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Column(
              children: [
                Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Expanded(
                      child: ListTile(
                        leading: GoogleUserCircleAvatar(
                          identity: widget.user,
                        ),
                        title: Text(widget.user.displayName ?? ''),
                        subtitle: Text(widget.user.email),
                      ),
                    ),
                    ElevatedButton(
                      onPressed: _handleSignOut,
                      child: const Text('Log out'),
                    )
                  ],
                ),
                Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                  const Text(
                    'Your current description',
                    style: TextStyle(fontWeight: FontWeight.bold, fontSize: 25),
                  ),
                  TextFormField(
                      initialValue: description ?? originalDescription,
                      onChanged: (text) {
                        description = text;
                        if ((originalDescription == null && text == "") ||
                            originalDescription == text) {
                          setState(() {
                            isButtonDisabled = true;
                          });
                        } else {
                          setState(() {
                            isButtonDisabled = false;
                          });
                        }
                      },
                      minLines: 6,
                      keyboardType: TextInputType.multiline,
                      maxLines: null,
                      onTapOutside: ((event) {
                        FocusScope.of(context).unfocus();
                      }),
                      decoration: InputDecoration(
                          border: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(25.0),
                            borderSide: BorderSide(
                              color: Colors.blue,
                            ),
                          ),
                          hintText: "Describe yourself however you want!")),
                  Center(
                    child: ElevatedButton(
                      style: raisedButtonStyle,
                      onPressed: isButtonDisabled
                          ? null
                          : () async {
                              await _updateDescription(
                                  authResponse.accessToken);
                              authResponse.updateDescription(description!);
                              setState(() {
                                isButtonDisabled = true;
                              });
                            },
                      child: Text('Update description'),
                    ),
                  ),
                  matchesList(authResponse)
                ]),
              ],
            ),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                FloatingActionButton.extended(
                  heroTag: "search",
                  icon: const Icon(Icons.search),
                  onPressed: () async {
                    Navigator.push(
                      context,
                      MaterialPageRoute(
                          builder: (context) =>
                              QueryPage(authResponse: authResponse)),
                    );
                  },
                  label: Text("Meet new people"),
                ),
                FloatingActionButton(
                    heroTag: "refresh",
                    child: const Icon(Icons.refresh),
                    onPressed: () async {
                      var newAuthResponse =
                          await API().refreshProfile(authResponse.accessToken);
                      authResponse.update(newAuthResponse);
                    })
              ],
            )
          ],
        ),
      );
    });
  }
}
