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
            backgroundColor: const Color(0xFF101010),
            primaryColorDark: Colors.orange,
            cardColor: const Color(0xFF101010)),
      ),
      themeMode: ThemeMode.dark,
      home: const MyHomePage(title: 'Flutter Demo Home Page 5'),
    );
  }
}

class Node extends StatelessWidget {
  final List<Key?> socketKeys;
  const Node(this.socketKeys, {super.key});

  @override
  Widget build(BuildContext context) {
    return CustomPaint(
      painter: NodePainter(context: context),
      child: ConstrainedBox(
        constraints: const BoxConstraints.tightFor(width: 128),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const NodeHeader('ADD'),
            NodeOutput(
              'Output 1',
              key: socketKeys[0],
            ),
            NodeOutput(
              'Output 2',
              key: socketKeys[1],
            ),
            NodeInput(
              'Input 1',
              key: socketKeys[2],
            ),
            NodeInput(
              'Input 2',
              key: socketKeys[3],
            ),
            const SizedBox(height: 8),
          ],
        ),
      ),
    );
  }
}

class NodeHeader extends StatelessWidget {
  final String label;

  const NodeHeader(this.label, {super.key});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: double.infinity,
      child: CustomPaint(
        painter: NodeHeaderPainter(),
        child: Padding(
          padding: const EdgeInsets.all(8),
          child: Text(label,
              textAlign: TextAlign.center,
              style: const TextStyle(fontWeight: FontWeight.bold)),
        ),
      ),
    );
  }
}

class NodeInput extends StatelessWidget {
  final String label;

  const NodeInput(this.label, {super.key});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: double.infinity,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(0, 8, 16, 0),
        child: CustomPaint(
          painter: NodeInputPainter(),
          child: Padding(
            padding: const EdgeInsets.all(4),
            child: Text(label, textAlign: TextAlign.left),
          ),
        ),
      ),
    );
  }
}

class NodeOutput extends StatelessWidget {
  final String label;

  const NodeOutput(this.label, {super.key});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: double.infinity,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(16, 8, 0, 0),
        child: CustomPaint(
          painter: NodeOutputPainter(),
          child: Padding(
            padding: const EdgeInsets.all(4),
            child: Text(label, textAlign: TextAlign.right),
          ),
        ),
      ),
    );
  }
}

class NodePainter extends CustomPainter {
  BuildContext context;

  NodePainter({required this.context});

  @override
  void paint(Canvas canvas, Size size) {
    var paint = Paint();
    paint.color = Theme.of(context).colorScheme.background;
    final r = RRect.fromLTRBR(
        0, 0, size.width, size.height, const Radius.circular(8));
    canvas.drawRRect(r, paint);
    paint.style = PaintingStyle.stroke;
    paint.color = Colors.grey.shade700;
    paint.strokeWidth = 1;
    canvas.drawRRect(r, paint);
  }

  @override
  bool shouldRepaint(covariant NodePainter oldDelegate) {
    return context != oldDelegate.context;
  }
}

class NodeHeaderPainter extends CustomPainter {
  NodeHeaderPainter();

  @override
  void paint(Canvas canvas, Size size) {
    var paint = Paint();
    paint.color = Colors.red;
    var z = const Radius.circular(0);
    var e = const Radius.circular(8);
    final r = RRect.fromLTRBAndCorners(0, 0, size.width, size.height,
        bottomLeft: z, bottomRight: z, topLeft: e, topRight: e);
    canvas.drawRRect(r, paint);
  }

  @override
  bool shouldRepaint(covariant NodeHeaderPainter oldDelegate) {
    return false;
  }
}

class NodeInputPainter extends CustomPainter {
  NodeInputPainter();

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint();
    paint.color = Colors.blue;
    const z = Radius.circular(0);
    const e = Radius.circular(8);
    final r = RRect.fromLTRBAndCorners(0, 0, size.width, size.height,
        bottomLeft: z, bottomRight: e, topLeft: z, topRight: e);
    canvas.drawRRect(r, paint);
  }

  @override
  bool shouldRepaint(covariant NodeInputPainter oldDelegate) {
    return false;
  }
}

class NodeOutputPainter extends CustomPainter {
  NodeOutputPainter();

  @override
  void paint(Canvas canvas, Size size) {
    var paint = Paint();
    paint.color = Colors.blue;
    var z = const Radius.circular(0);
    var e = const Radius.circular(8);
    final r = RRect.fromLTRBAndCorners(0, 0, size.width, size.height,
        bottomLeft: e, bottomRight: z, topLeft: e, topRight: z);
    canvas.drawRRect(r, paint);
  }

  @override
  bool shouldRepaint(covariant NodeOutputPainter oldDelegate) {
    return false;
  }
}

class NodeWirePainter extends CustomPainter {
  final GlobalKey socket0, socket1, node2;
  final GlobalKey self;

  const NodeWirePainter(this.self, this.socket0, this.socket1, this.node2);

  @override
  void paint(Canvas canvas, Size size) {
    final sp = (self.currentContext?.findRenderObject() as RenderBox)
        .localToGlobal(Offset.zero);
    const padding = 8.0;
    const paddingOffset = Offset(0, padding);
    final s0 = socket0.currentContext?.findRenderObject() as RenderBox;
    final s0p = s0.size.centerRight(s0.localToGlobal(Offset.zero)) + paddingOffset / 2;
    final s1 = socket1.currentContext?.findRenderObject() as RenderBox;
    final s1p = s1.size.centerLeft(s1.localToGlobal(Offset.zero)) + paddingOffset / 2;
    var paint = Paint();
    paint.color = Colors.blue;
    paint.strokeWidth = 8;
    canvas.drawLine(s0p - sp, s1p - sp, paint);
    canvas.drawCircle(s0p - sp, (s0.size.height - padding) / 2, paint);
    canvas.drawCircle(s1p - sp, (s1.size.height - padding) / 2, paint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    return false;
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
    return Stack(
      children: [
        CustomPaint(
          key: graph,
          painter: NodeWirePainter(graph, socket0, socket1, socket2),
        ),
        Node([null, socket0, null, null]),
        Positioned(
          left: 200,
          top: 100,
          child: Node([null, null, socket1, null]),
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
          width: double.infinity, height: double.infinity, child: NodeGraph()),
      floatingActionButton: FloatingActionButton(
        onPressed: _incrementCounter,
        tooltip: 'Increment',
        child: const Icon(Icons.add),
      ),
    );
  }
}
