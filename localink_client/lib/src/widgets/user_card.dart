import 'package:flutter/material.dart';

class UserCard extends StatelessWidget {
  final String name;
  final String description;
  final double score;
  final GestureTapCallback? onTap;

  const UserCard(
      {super.key,
      required this.name,
      required this.description,
      required this.score,
      this.onTap});

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
        trailing: Text("${(score * 100).round()}%"),
      )
    );
  }
}
