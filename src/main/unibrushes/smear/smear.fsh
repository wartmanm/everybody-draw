precision lowp float;
varying vec2 speed;
uniform sampler2D texture;
uniform sampler2D backbuffer;
uniform vec2 texturesize;
uniform mat4 textureMatrix;

void main() {
  /*vec2 dist = normalize(speed);*/
  vec2 dist = speed;
  float alpha = texture2D(texture, gl_PointCoord).a;
  vec2 uv = vec2((gl_FragCoord - vec4(-dist.x, dist.y, 0.0, 0.0)) * textureMatrix) * vec2(0.5, -0.5);
  vec3 oldcolor = vec3(texture2D(backbuffer, uv));
  float ir = oldcolor.r;
  float ig = oldcolor.g;
  float ib = oldcolor.b;
  vec3 newcolor = vec3(ir, ig, ib) * alpha;
  gl_FragColor = vec4(newcolor, alpha);
}
