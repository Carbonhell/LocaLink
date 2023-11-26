import 'package:flutter/material.dart';
import 'package:localink_client/src/models/auth.dart';
import 'package:localink_client/src/widgets/quoted_text.dart';
import 'package:localink_client/src/widgets/user_card.dart';

import '../helpers/API.dart';
import '../helpers/utils.dart';

class QueryPage extends StatefulWidget {
  final AuthResponse authResponse;

  const QueryPage({super.key, required this.authResponse});

  @override
  State<QueryPage> createState() => QueryPageState();
}

class QueryPageState extends State<QueryPage> {
  bool usersLoaded = false;

  Future<List<UserInfo>> getData(String token) async {
    var position = null;
    try {
      position = await determinePosition();
    } catch (e) {
      return Future.error(e);
    }
    return await API().query(token, position);
  }

  // Create a match object linking the two users on the backend
  Future<void> addMatch(
      String token, UserInfo item, BuildContext context) async {
    widget.authResponse.addMatch(item.id, item.name, item.description);
    return await API().match(token, MatchOp.Add, item.id, item.name, item.description);
  }

  Future<void> onCardTap(
      String token, UserInfo item, BuildContext context) async {
    var alertResult = await showDialog<bool>(
      context: context,
      builder: (BuildContext context) => AlertDialog(
        title: Text('Match with ${item.name}?'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Text('Do you really want to match with this user?'),
            QuotedText(text: item.description,),
          ],
        ),
        actions: <Widget>[
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () async {
              await addMatch(token, item, context);
              Navigator.pop(context, true); // From the alert
              Navigator.pop(context, true); // And from the query page
            },
            child: const Text('OK'),
          ),
        ],
      ),
    );
    if (alertResult != null && alertResult) {
      print("match with ${item.id}");
    }
  }

  @override
  Widget build(BuildContext context) {
    final token = widget.authResponse.accessToken;

    return Scaffold(
        appBar: AppBar(
          title: const Text('Match with people'),
        ),
        body: FutureBuilder<List<UserInfo>>(
          future: getData(token),
          builder: (context, snapshot) {
            if (snapshot.hasData &&
                snapshot.connectionState == ConnectionState.done) {
              return ListView.builder(
                itemCount: snapshot.data!.length,
                itemBuilder: (context, index) {
                  final item = snapshot.data?[index];

                  return UserCard(
                      name: item!.name,
                      description: item.description,
                      score: item.searchScore,
                      onTap: () => onCardTap(token, item, context));
                },
              );
            } else if (snapshot.hasError) {
              return AlertDialog(
                title: const Text('Location problem'),
                content: const SingleChildScrollView(
                  child: ListBody(
                    children: <Widget>[
                      Text(
                          'Please ensure the device location sensors are turned on and that the app has been given the required permission to fetch location data.'),
                    ],
                  ),
                ),
                actions: <Widget>[
                  TextButton(
                    child: const Text('OK'),
                    onPressed: () {
                      Navigator.of(context).pop();
                    },
                  ),
                ],
              );
            } else if (snapshot.connectionState == ConnectionState.done && !snapshot.hasData) {
              return AlertDialog(
                title: const Text('No matches available'),
                content: const SingleChildScrollView(
                  child: ListBody(
                    children: <Widget>[
                      Text(
                          'We were not able to find a suitable match based on your current description and position. Try moving in a more public area, or edit your description to be more generic.'),
                    ],
                  ),
                ),
                actions: <Widget>[
                  TextButton(
                    child: const Text('OK'),
                    onPressed: () {
                      Navigator.of(context).pop();
                    },
                  ),
                ],
              );
            } else {
              return const Center(child: CircularProgressIndicator());
            }
          },
        ));
  }
}
