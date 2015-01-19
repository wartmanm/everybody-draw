precision lowp float;
uniform mat4 textureMatrix;
attribute float vSize;
attribute float vTime;
attribute vec4 vPosition;
attribute float vPointer;
attribute vec2 vSpeed;
attribute float vDistance;
uniform vec3 vColor;
varying float time;
varying float size;
varying vec3 color;
varying vec2 position;

void main() {
    time = vTime;
    float tmpSize = vSize * 1500.0;
    size = clamp(tmpSize, 7.5, 60.0);
    color = vec3(1.0, 1.0, 0.0);
    gl_PointSize = 30.0;
    gl_Position = (textureMatrix * vPosition);
    position = vec2(textureMatrix * vPosition);
}
