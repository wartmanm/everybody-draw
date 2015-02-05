precision mediump float;
varying float time;
varying float size;
varying vec3 color;
uniform sampler2D texture;
uniform sampler2D backbuffer;
void main() {
  float ctime = clamp(time, 0.0, 1.0);
  float csize = clamp(size, 0.0, 1.0);
  float alpha = texture2D(texture, gl_PointCoord).a;
  gl_FragColor = vec4(color * alpha, alpha);
}
