precision lowp float;
varying vec2 texcoord;
uniform sampler2D texture;
uniform sampler2D backbuffer;
uniform vec2 texturesize;
uniform mat4 textureMatrix;

void main() {
  vec2 uv = texcoord + gl_PointCoord / vec2(4.0, 9.0);
  vec4 color = texture2D(texture, uv);
  gl_FragColor = vec4(vec3(color) * color.a, color.a);
}
