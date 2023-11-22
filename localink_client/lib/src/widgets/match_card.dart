import 'package:flutter/material.dart';
import 'package:localink_client/src/models/auth.dart';

import '../helpers/API.dart';

class MatchCard extends StatelessWidget {
  ValueChanged<MatchStatus> updateItem;
  final String token;
  final String id;
  final String name;
  final String description;
  final MatchStatus? status;
  final GestureTapCallback? onTap;

  MatchCard({super.key,
    required this.updateItem,
    required this.token,
    required this.id,
    required this.name,
    required this.description,
    this.onTap,
    this.status});

  Future<void> accept(String token) async {
    // 1) invoke a function to write the accepted status on cosmos db
    await API().match(token, MatchOp.Accept, id, name, description);
    updateItem(MatchStatus.Accepted);
    // 2) update local status so that the widget refreshes
  }

  Future<void> reject(String token) async {
    // 1) invoke a function to write the accepted status on cosmos db
    await API().match(token, MatchOp.Reject, id, name, description);
    // 2) update local status so that the widget refreshes
    updateItem(MatchStatus.Denied);
  }

  Widget statusToWidget(MatchStatus status) {
    switch (status) {
      case MatchStatus.Pending:
        return Container(
            padding: EdgeInsets.all(10),
            decoration: BoxDecoration(
              color: Colors.orange,
              border: Border.all(color: Colors.black, width: 0.0),
              borderRadius: new BorderRadius.all(Radius.elliptical(100, 25)),
            ),
            child: Icon(Icons.hourglass_bottom));
      case MatchStatus.AwaitingUserAction:
        return Row(mainAxisSize: MainAxisSize.min, children: [
          InkWell(
            onTap: () async {print("ok");accept(token);},
            child: Container(
                padding: EdgeInsets.all(10),
                decoration: BoxDecoration(
                  color: Colors.lightGreen,
                  border: Border.all(color: Colors.black, width: 0.0),
                  borderRadius: new BorderRadius.all(
                      Radius.elliptical(100, 25)),
                ),
                child: Icon(Icons.check)),
          ),
          SizedBox(width: 10,),
          InkWell(
            onTap: () async {reject(token);},
            child: Container(
                padding: EdgeInsets.all(10),
                decoration: BoxDecoration(
                  color: Colors.redAccent,
                  border: Border.all(color: Colors.black, width: 0.0),
                  borderRadius: new BorderRadius.all(
                      Radius.elliptical(100, 25)),
                ),
                child: Icon(Icons.close)),
          )
        ]);
      case MatchStatus.Accepted:
        return Container(
            padding: EdgeInsets.all(10),
            decoration: BoxDecoration(
              color: Colors.green,
              border: Border.all(color: Colors.black, width: 0.0),
              borderRadius: new BorderRadius.all(Radius.elliptical(100, 25)),
            ),
            child: Icon(Icons.check));
      case MatchStatus.Denied:
        return Container(
            padding: EdgeInsets.all(10),
            decoration: BoxDecoration(
              color: Colors.red,
              border: Border.all(color: Colors.black, width: 0.0),
              borderRadius: new BorderRadius.all(Radius.elliptical(100, 25)),
            ),
            child: Icon(Icons.close));
    }
  }

  @override
  Widget build(BuildContext context) {
    return Card(
      child: ListTile(
          onTap: onTap,
          leading: CircleAvatar(
            backgroundColor: Colors.white,
            child: Text(name.characters.first),
          ),
          title: Text(name),
          subtitle: Text(
            description,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
          trailing: status != null ? statusToWidget(status!) : null),
    );
  }
}
