precision lowp float;
varying float time;
varying float size;
varying vec3 color;
varying vec2 position;
uniform sampler2D texture;
uniform sampler2D backbuffer;
uniform vec2 texturesize;
uniform mat4 textureMatrix;
void main() {
  vec4 orig = texture2D(texture, gl_PointCoord);
  vec3 core = vec3(orig.g);
  vec3 corona = color * orig.r;
  vec3 color2 = (core + corona);
  gl_FragColor = vec4(core, orig.g);
  /*gl_FragColor = vec4(color2, orig.r + orig.g);*/
}
