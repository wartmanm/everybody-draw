precision lowp float;
uniform mat4 textureMatrix;
attribute float vSize;
attribute float vTime;
attribute vec4 vPosition;
attribute float vPointer;
attribute vec2 vSpeed;
attribute float vDistance;
uniform vec3 vColor;
uniform float vSizeFactor;
varying float time;
varying float size;
varying vec3 color;
varying vec2 position;

void main() {
    time = vTime;
    float tmpSize = vSize * 1500.0 * vSizeFactor;
    size = clamp(tmpSize, 7.5, 60.0);
    color = vColor;
    gl_PointSize = size;
    gl_Position = (textureMatrix * vPosition);
    position = vec2(textureMatrix * vPosition);
}
