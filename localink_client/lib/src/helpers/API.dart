import 'dart:convert';

import 'package:geolocator/geolocator.dart';
import 'package:google_maps_flutter/google_maps_flutter.dart';
import 'package:http/http.dart' as http;
import 'package:localink_client/src/models/auth.dart';
import 'package:flutter_dotenv/flutter_dotenv.dart';

class API {
  final String endpoint;
  final String authUrl;
  final String generateEmbeddingsUrl;
  final String queryUrl;
  final String syncPositionUrl;
  final String addMatchUrl;
  final String meetUrl;

  static API? _singleton;

  API._internal(this.endpoint, this.authUrl, this.generateEmbeddingsUrl,
      this.queryUrl, this.syncPositionUrl, this.addMatchUrl, this.meetUrl);

  factory API() {
    if (_singleton != null) {
      return _singleton!;
    }
    _singleton = API._internal(
      dotenv.env['API_ENDPOINT']!,
      dotenv.env['API_AUTH_URL']!,
      dotenv.env['API_EMBEDDINGS_URL']!,
      dotenv.env['API_QUERY_URL']!,
      dotenv.env['API_SYNC_POSITION_URL']!,
      dotenv.env['API_MATCH_URL']!,
      dotenv.env['API_MEET_URL']!,
    );
    return _singleton!;
  }

  Future<AuthResponse> auth(String idToken) async {
    print("HTTPing $authUrl");

    Map data = {'id_token': idToken};
    //encode Map to JSON
    var body = json.encode(data);

    final response = await http.post(Uri.parse(authUrl),
        headers: {"Content-Type": "application/json"}, body: body);

    if (response.statusCode == 200) {
      return AuthResponse.fromJson(
          jsonDecode(response.body) as Map<String, dynamic>);
    } else {
      final err = response.body;
      throw Exception('Auth call error: $err - ${response.statusCode}');
    }
  }

  Future<AuthResponse> refreshProfile(String token) async {
    print("HTTPing refreshProfile $authUrl");

    final response = await http.get(Uri.parse(authUrl), headers: {
      "Content-Type": "application/json",
      "Authorization": "Bearer $token"
    });

    if (response.statusCode == 200) {
      return AuthResponse.fromJson(
          jsonDecode(response.body) as Map<String, dynamic>);
    } else {
      final err = response.body;
      throw Exception('Auth call error: $err');
    }
  }

  Future<void> generateEmbeddings(String token, String description) async {
    print("HTTPing $generateEmbeddingsUrl");

    Map data = {'description': description};
    //encode Map to JSON
    var body = json.encode(data);

    final response = await http.post(Uri.parse(generateEmbeddingsUrl),
        headers: {
          "Content-Type": "application/json",
          "Authorization": "Bearer $token"
        },
        body: body);

    if (response.statusCode == 200) {
      return;
    } else {
      final err = response.body;
      throw Exception('GenerateEmbeddings call error: $err');
    }
  }

  Future<List<UserInfo>> query(String token, Position position) async {
    print("HTTPing $queryUrl");

    Map data = {
      'position': {
        'latitude': position.latitude,
        'longitude': position.longitude
      }
    };
    //encode Map to JSON
    var body = json.encode(data);

    final response = await http.post(Uri.parse(queryUrl),
        headers: {
          "Content-Type": "application/json",
          "Authorization": "Bearer $token"
        },
        body: body);

    if (response.statusCode == 200) {
      var resBody = jsonDecode(response.body) as Map<String, dynamic>;
      var users = resBody['data']['value']
          .map((user) => UserInfo.fromJson(user))
          .toList()
          .cast<UserInfo>();

      return users;
    } else {
      final err = response.body;
      throw Exception('Query call error: $err');
    }
  }

  Future<void> syncPosition(String token, LatLng location) async {
    print("HTTPing $syncPositionUrl with token $token");

    Map data = {
      'latitude': location.latitude,
      'longitude': location.longitude
    };
    //encode Map to JSON
    var body = json.encode(data);

    final response = await http.post(Uri.parse(syncPositionUrl),
        headers: {
          "Content-Type": "application/json",
          "Authorization": "Bearer $token"
        },
        body: body);

    if (response.statusCode == 200) {
      return;
    } else {
      final err = response.body;
      throw Exception(
          'SyncPosition call error: $err - ${response.headers} - ${response.statusCode}');
    }
  }

  Future<void> match(String token, MatchOp op, String userID, String userName,
      String userDescription) async {
    print("HTTPing $addMatchUrl with token $token");

    Map data = {
      'operation': op.name,
      'target_user_id': userID,
      'target_user_name': userName,
      'target_description': userDescription
    };
    //encode Map to JSON
    var body = json.encode(data);

    final response = await http.post(Uri.parse(addMatchUrl),
        headers: {
          "Content-Type": "application/json",
          "Authorization": "Bearer $token"
        },
        body: body);

    if (response.statusCode == 200) {
      return;
    } else {
      final err = response.body;
      throw Exception(
          'Match call error: $err - ${response.headers} - ${response.statusCode}');
    }
  }

  Future<MeetResponse> meet(String token, String targetId) async {
    print("HTTPing $meetUrl with token $token");

    Map data = {'target_id': targetId};
    //encode Map to JSON
    var body = json.encode(data);

    final response = await http.post(Uri.parse(meetUrl),
        headers: {
          "Content-Type": "application/json",
          "Authorization": "Bearer $token"
        },
        body: body);

    if (response.statusCode == 200) {
      return MeetResponse.fromJson(
          jsonDecode(response.body) as Map<String, dynamic>);;
    } else {
      final err = response.body;
      throw Exception(
          'Meet call error: $err - ${response.headers} - ${response.statusCode}');
    }
  }
}

class UserInfo {
  final String id;
  final String name;
  final String description;

  UserInfo({
    required this.id,
    required this.name,
    required this.description,
  });

  factory UserInfo.fromJson(Map<String, dynamic> json) {
    return UserInfo(
      id: json['id'],
      name: utf8.decode(json['name'].codeUnits),
      description: utf8.decode(json['description'].codeUnits),
    );
  }
}

enum MatchOp { Add, Accept, Reject }

class MeetResponse {
  final Point poi;

  MeetResponse(this.poi);

  factory MeetResponse.fromJson(Map<String, dynamic> json) {
    return MeetResponse(Point.fromJson(json['poi']));
  }
}

class Point {
  final String type;
  final List<double> coordinates;

  Point(this.type, this.coordinates);

  factory Point.fromJson(Map<String, dynamic> json) {
    return Point(json['type'], json['coordinates'].cast<double>());
  }
}
