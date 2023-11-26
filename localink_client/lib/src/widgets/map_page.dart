import 'dart:async';

import 'package:flutter/material.dart';
import 'package:geolocator/geolocator.dart';
import 'package:google_maps_flutter/google_maps_flutter.dart';
import 'package:localink_client/src/models/auth.dart';

import '../helpers/API.dart';

class MapPage extends StatefulWidget {
  final AuthResponse authResponse;
  final Match match;

  const MapPage({super.key, required this.authResponse, required this.match});

  @override
  State<MapPage> createState() => MapPageState();
}

class MapPageState extends State<MapPage> {
  LatLng? _devicePosition;
  LatLng? _targetPosition;

  Future<void> setupDeviceMarker() async {
    bool serviceEnabled;
    LocationPermission permission;

    serviceEnabled = await Geolocator.isLocationServiceEnabled();
    if (!serviceEnabled) {
      // Location services are not enabled don't continue
      // accessing the position and request users of the
      // App to enable the location services.
      return Future.error('Location services are disabled.');
    }

    permission = await Geolocator.checkPermission();
    if (permission == LocationPermission.denied) {
      permission = await Geolocator.requestPermission();
      if (permission == LocationPermission.denied) {
        // Permissions are denied, next time you could try
        // requesting permissions again (this is also where
        // Android's shouldShowRequestPermissionRationale
        // returned true. According to Android guidelines
        // your App should show an explanatory UI now.
        return Future.error('Location permissions are denied');
      }
    }

    if (permission == LocationPermission.deniedForever) {
      // Permissions are denied forever, handle appropriately.
      return Future.error(
          'Location permissions are permanently denied, we cannot request permissions.');
    }

    const LocationSettings locationSettings = LocationSettings(
      accuracy: LocationAccuracy.high,
      distanceFilter: 20,
    );
    StreamSubscription<Position> positionStream =
        Geolocator.getPositionStream(locationSettings: locationSettings)
            .listen((Position? position) {
      if (position != null) {
        setState(() {
          _devicePosition = LatLng(position!.latitude, position!.longitude);
        });
      }
    });
  }

  Future<void> setupMarkers(String token) async {
    var meet = await API().meet(token, widget.match.userID);
    _targetPosition = LatLng(meet.poi.coordinates[0], meet.poi.coordinates[1]);
    await setupDeviceMarker();
  }

  @override
  Widget build(BuildContext context) {
    final token = widget.authResponse.accessToken;

    final Completer<GoogleMapController> _controller =
        Completer<GoogleMapController>();


    return Scaffold(
      appBar: AppBar(
        title: Text('Meet with ${widget.match.userName}'),
      ),
      body: FutureBuilder<void>(
          future: setupMarkers(token),
          builder: (BuildContext context, AsyncSnapshot<void> snapshot) {
            print(snapshot);
            print(_devicePosition);
            if (snapshot.connectionState == ConnectionState.done &&
                _devicePosition != null) {
              var markers = {Marker(
                markerId: MarkerId("source"),
                position: _devicePosition!,
                icon: BitmapDescriptor.defaultMarkerWithHue(BitmapDescriptor.hueViolet),
              )};
              if (_targetPosition != null) {
                markers.add(Marker(
                  markerId: MarkerId("target"),
                  position: _targetPosition!,
                ));
              }
              
              return GoogleMap(
                mapType: MapType.normal,
                initialCameraPosition: CameraPosition(
                  target: _devicePosition!,
                  zoom: 15,
                ),
                markers: markers,
                onMapCreated: (GoogleMapController controller) {
                  _controller.complete(controller);
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
            }
            return const Center(child: CircularProgressIndicator());
          }),
      floatingActionButton: _devicePosition != null ? FloatingActionButton.extended(
        onPressed: () {Navigator.of(context).pop();},
        label: const Text('Found!'),
        icon: const Icon(Icons.lightbulb),
      ) : null,
    );
  }
}
