precision lowp float;
uniform mat4 textureMatrix;
attribute float vSize;
attribute float vTime;
attribute vec4 vPosition;
attribute float vPointer;
attribute float vDistance;
attribute vec2 vSpeed;
uniform vec3 vColor;
varying vec2 texcoord;

void main() {
  texcoord = vSpeed;
  gl_PointSize = 64.0;
  gl_Position = (textureMatrix * vPosition);
}
