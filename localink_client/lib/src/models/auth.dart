import 'dart:convert';

import 'package:flutter/material.dart';

class AuthResponse extends ChangeNotifier {
  String id;String accessToken;
  String? description;
  List<Match> matches;

  AuthResponse({
    required this.id,
    required this.accessToken,
    this.description,
    required this.matches,
  });

  factory AuthResponse.fromJson(Map<String, dynamic> json) {
    var matches = (json['matches'] as List<dynamic>);
    var typedMatches = matches.isEmpty
        ? List<Match>.empty(growable: true)
        : matches
            .map((matchJson) => Match(
                matchJson['id'],
                utf8.decode(matchJson['name'].codeUnits),
                utf8.decode(matchJson['description'].codeUnits),
                MatchStatus.values.asNameMap()[matchJson['match_status']]!))
            .toList();

    return AuthResponse(
      id: json['id'],
      accessToken: json['access_token'],
      description: json['description'] != null ? utf8.decode(json['description'].codeUnits) : "",
      matches: typedMatches,
    );
  }

  void addMatch(String id, String name, String description) {
    matches.add(Match(id, name, description, MatchStatus.Pending));
    notifyListeners();
  }

  void updateDescription(String description) {
    this.description = description;
    notifyListeners();
  }

  void update(AuthResponse newAuthResponse) {
    id = newAuthResponse.id;
    description = newAuthResponse.description;
    matches = newAuthResponse.matches;
    notifyListeners();
  }
}

class Match {
  final String userID;
  final String userName;
  final String userDescription;
  MatchStatus matchStatus;

  Match(this.userID, this.userName, this.userDescription, this.matchStatus);
}

enum MatchStatus { Pending, AwaitingUserAction, Accepted, Denied }
