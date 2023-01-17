import 'dart:math';

import 'package:flutter/material.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo 2',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      darkTheme: ThemeData.from(
        colorScheme: ColorScheme.fromSwatch(
          brightness: Brightness.dark,
          accentColor: Colors.orange,
          backgroundColor: const Color(0xFF000000),
          primaryColorDark: Colors.orange,
          cardColor: const Color(0xFF101010),
        ),
      ),
      themeMode: ThemeMode.dark,
      home: const MyHomePage(title: 'Flutter Demo Home Page 5'),
    );
  }
}

const nodeWidth = 128.0;
const nodeHeaderHeight = 32.0;
const nodeHeaderFontSize = 16.0;
const nodeCornerRadius = Radius.circular(16);
const nodeSocketHeight = 32.0;
const nodeSocketPadding = 8.0;
const nodeBackgroundColor = Color(0xFF101010);

class VisualWire {
  final VisualNode start, end;
  final int startIndex, endIndex;

  const VisualWire(this.start, this.startIndex, this.end, this.endIndex);
}

class VisualSocket {
  final String label;
  final Color color;

  const VisualSocket(this.label, {this.color = Colors.blue});
}

class VisualNode {
  final String label;
  final Color color;
  final List<VisualSocket> inputSockets, outputSockets;
  final Offset position;

  const VisualNode(
    this.position,
    this.label,
    this.inputSockets,
    this.outputSockets, {
    this.color = Colors.red,
  });

  Offset outputSocketConnectionPoint(int index) {
    return position +
        Offset(
          nodeWidth,
          nodeHeaderHeight +
              nodeSocketPadding +
              index * (nodeSocketPadding + nodeSocketHeight) +
              nodeSocketHeight / 2,
        );
  }

  Offset inputSocketConnectionPoint(int index) {
    return position +
        Offset(
          0,
          nodeHeaderHeight +
              nodeSocketPadding +
              (outputSockets.length + index) *
                  (nodeSocketPadding + nodeSocketHeight) +
              nodeSocketHeight / 2,
        );
  }

  double height() {
    return nodeHeaderHeight +
        (inputSockets.length + outputSockets.length) *
            (nodeSocketPadding + nodeSocketHeight) +
        max(nodeCornerRadius.y, nodeSocketPadding);
  }
}

class NodeGraphPainter extends CustomPainter {
  BuildContext context;
  List<VisualNode> nodes;
  List<VisualWire> wires;

  NodeGraphPainter(this.context, this.nodes, this.wires);

  @override
  void paint(Canvas canvas, Size size) {
    for (var wire in wires) {
      paintWire(canvas, wire);
    }
    for (var node in nodes) {
      paintNode(canvas, node);
    }
  }

  @override
  bool shouldRepaint(covariant NodeGraphPainter oldDelegate) {
    return context != oldDelegate.context;
  }

  void paintWire(Canvas canvas, VisualWire wire) {
    final paint = Paint();
    paint.color = wire.start.outputSockets[wire.startIndex].color;
    final start = wire.start.outputSocketConnectionPoint(wire.startIndex);
    final end = wire.end.inputSocketConnectionPoint(wire.endIndex);
    canvas.drawCircle(
      start,
      nodeSocketHeight / 2,
      paint,
    );
    canvas.drawCircle(
      end,
      nodeSocketHeight / 2,
      paint,
    );
    paint.strokeWidth = nodeSocketHeight / 4;
    canvas.drawLine(start, end, paint);
  }

  void paintNode(Canvas canvas, VisualNode node) {
    canvas.save();
    canvas.translate(node.position.dx, node.position.dy);
    paintNodeBackground(
      canvas,
      node.height(),
    );
    paintNodeHeader(canvas, node);
    canvas.translate(0, nodeHeaderHeight + nodeSocketPadding);
    for (var socket in node.outputSockets) {
      paintNodeOutput(canvas, socket);
      canvas.translate(0, nodeSocketHeight + nodeSocketPadding);
    }
    for (var socket in node.inputSockets) {
      paintNodeInput(canvas, socket);
      canvas.translate(0, nodeSocketHeight + nodeSocketPadding);
    }
    canvas.restore();
  }

  void paintNodeBackground(Canvas canvas, double height) {
    var paint = Paint();
    paint.color = nodeBackgroundColor;
    final r = RRect.fromLTRBR(0, 0, nodeWidth, height, nodeCornerRadius);
    canvas.drawRRect(r, paint);
    paint.style = PaintingStyle.stroke;
    paint.color = Colors.grey.shade700;
    paint.strokeWidth = 1;
    canvas.drawRRect(r, paint);
  }

  void drawText(Canvas canvas, String text, Rect boundingBox,
      {FontWeight? weight, Alignment alignment = Alignment.center}) {
    final textStyle =
        TextStyle(fontSize: nodeHeaderFontSize, fontWeight: weight);
    final textSpan = TextSpan(text: text, style: textStyle);
    final textPainter =
        TextPainter(text: textSpan, textDirection: TextDirection.ltr);
    textPainter.layout(minWidth: 0, maxWidth: nodeWidth);
    final dx = boundingBox.size.width - textPainter.width;
    final dy = boundingBox.size.height - textPainter.height;
    final alignBox =
        Rect.fromLTWH(boundingBox.topLeft.dx, boundingBox.topLeft.dy, dx, dy);
    textPainter.paint(canvas, alignment.withinRect(alignBox));
  }

  void paintNodeHeader(Canvas canvas, VisualNode node) {
    var paint = Paint();
    paint.color = node.color;
    final r = RRect.fromLTRBAndCorners(0, 0, nodeWidth, nodeHeaderHeight,
        topLeft: nodeCornerRadius, topRight: nodeCornerRadius);
    canvas.drawRRect(r, paint);
    drawText(
      canvas,
      node.label,
      const Rect.fromLTWH(0, 0, nodeWidth, nodeHeaderHeight),
      weight: FontWeight.bold,
    );
  }

  void paintNodeOutput(Canvas canvas, VisualSocket socket) {
    var paint = Paint();
    paint.color = socket.color;
    final r = RRect.fromLTRBAndCorners(
        2 * nodeSocketPadding, 0, nodeWidth, nodeSocketHeight,
        topLeft: nodeCornerRadius, bottomLeft: nodeCornerRadius);
    canvas.drawRRect(r, paint);
    drawText(
      canvas,
      socket.label,
      const Rect.fromLTRB(
          3 * nodeSocketPadding, 0, nodeWidth, nodeSocketHeight),
      alignment: Alignment.centerLeft,
    );
  }

  void paintNodeInput(Canvas canvas, VisualSocket socket) {
    var paint = Paint();
    paint.color = socket.color;
    final r = RRect.fromLTRBAndCorners(
        0, 0, nodeWidth - 2 * nodeSocketPadding, nodeSocketHeight,
        topRight: nodeCornerRadius, bottomRight: nodeCornerRadius);
    canvas.drawRRect(r, paint);
    drawText(
      canvas,
      socket.label,
      const Rect.fromLTRB(nodeSocketPadding, 0,
          nodeWidth - 3 * nodeSocketPadding, nodeSocketHeight),
      alignment: Alignment.centerLeft,
    );
  }
}

class NodeGraph extends StatelessWidget {
  final GlobalKey graph = GlobalKey(),
      socket0 = GlobalKey(),
      socket1 = GlobalKey(),
      socket2 = GlobalKey();

  NodeGraph({super.key});

  @override
  Widget build(BuildContext context) {
    const nodes = [
      VisualNode(
        Offset(0, 0),
        'ADD',
        [VisualSocket('a'), VisualSocket('b')],
        [VisualSocket('result')],
      ),
      VisualNode(
        Offset(2 * nodeWidth, 0),
        'ADD',
        [VisualSocket('a'), VisualSocket('b')],
        [VisualSocket('result')],
      ),
    ];
    final wires = [
      VisualWire(nodes[0], 0, nodes[1], 0),
      VisualWire(nodes[0], 0, nodes[1], 1),
    ];
    return Stack(
      children: [
        CustomPaint(
          key: graph,
          painter: NodeGraphPainter(context, nodes, wires),
        ),
      ],
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  int _counter = 0;

  void _incrementCounter() {
    setState(() {
      _counter++;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.title),
      ),
      body: SizedBox(
        width: double.infinity,
        height: double.infinity,
        child: NodeGraph(),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _incrementCounter,
        tooltip: 'Increment',
        child: const Icon(Icons.add),
      ),
    );
  }
}
