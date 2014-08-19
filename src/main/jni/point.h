
struct Coordinate {
  float x;
  float y;
}

struct PaintPoint {
  struct Coordinate pos;
  float time;
  float size;
}

struct ShaderPaintPoint {
  struct Coordinate pos;
  float time;
  float size;
  float distance;
  float counter;
}

typedef void (*ShaderCallback)(ShaderPaintPoint *points, int count);
