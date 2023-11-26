import 'package:flutter/material.dart';

class QuotedText extends StatelessWidget {
  final String text;
  final TextStyle quoteStyle;
  final TextStyle textStyle;
  final EdgeInsets padding;

  QuotedText({
    required this.text,
    this.textStyle = const TextStyle(
        color: Colors.black, fontWeight: FontWeight.normal, fontSize: 16),
    this.quoteStyle = const TextStyle(
        color: Colors.black, fontWeight: FontWeight.bold, fontSize: 16),
    this.padding = const EdgeInsets.fromLTRB(8.0, 8.0, 8.0, 8.0),
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisAlignment: MainAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [Text('\u275D', style: quoteStyle), const Spacer()],
        ),
        Padding(
          padding: padding,
          child: Text(text, textAlign: TextAlign.center, style: textStyle),
        ),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Spacer(),
            Text('\u275E', style: quoteStyle),
          ],
        ),
      ],
    );
  }
}