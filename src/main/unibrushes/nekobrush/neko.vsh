precision lowp float;
uniform mat4 textureMatrix;
attribute float vSize;
attribute float vTime;
attribute vec4 vPosition;
attribute float vPointer;
attribute float vSpeed;
attribute float vDistance;
uniform vec3 vColor;
varying vec2 texcoord;

void main() {
  texcoord = vec2(vSize, vSpeed);
  gl_PointSize = 64.0;
  gl_Position = (textureMatrix * vPosition);
}
