precision lowp float;
uniform mat4 textureMatrix;
attribute float vSize;
attribute float vTime;
attribute vec4 vPosition;
attribute float vPointer;
attribute vec2 vSpeed;
attribute float vDistance;
uniform vec3 vColor;
varying vec2 speed;

void main() {
    speed = vSpeed;
    gl_Position = (textureMatrix * vPosition);
    gl_PointSize = 50.0;
}
