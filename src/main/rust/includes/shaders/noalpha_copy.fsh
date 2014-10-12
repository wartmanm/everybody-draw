precision lowp float;
uniform sampler2D texture;
varying vec2 uv;
void main() {
    gl_FragColor = vec4(vec3(texture2D(texture, uv)), 1.0);
}
