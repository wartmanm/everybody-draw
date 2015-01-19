precision lowp float;
this should fail
uniform mat4 textureMatrix;
attribute float vSize;
attribute float vTime;
attribute vec4 vPosition;
attribute float vPointer;
attribute vec2 vSpeed;
attribute float vDistance;
uniform vec3 vColor;
varying vec2 texcoord;

void main() {
  texcoord = vSpeed;
  gl_PointSize = 64.0;
  gl_Position = (textureMatrix * vPosition);
}
