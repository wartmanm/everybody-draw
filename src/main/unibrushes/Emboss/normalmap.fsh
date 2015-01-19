precision lowp float;
uniform mat4 textureMatrix;
uniform sampler2D texture;
varying vec2 uv;
precision mediump float;
uniform vec2 texturesize;
void main() {
  vec2 leftpos = uv + vec2(-1.0,  0.0) / texturesize;
  vec2 uppos   = uv + vec2( 0.0, -1.0) / texturesize;
  vec2 herepos = uv;
  vec4 herecolor = texture2D(texture, herepos);
  vec4 leftcolor = texture2D(texture, leftpos);
  vec4 upcolor   = texture2D(texture, uppos);
  float h = length(herecolor);
  float l = length(leftcolor);
  float u = length(upcolor);
  // TODO: average, use alpha
  float x = l - h;
  float y = u - h;
  float yscale = 10.0;
  float xzscale = 1.0;
  vec3 normal = normalize(vec3(-x * yscale, 2.0 * xzscale, y * yscale));

  vec3 lightDir = normalize(vec3(1.0, 1.0, 1.0));
  vec3 color = vec3(texture2D(texture, uv));
  float diffuse = max(dot(lightDir, normal), 0.0);
  
  vec3 outcolor;
  outcolor = color * diffuse;
  gl_FragColor = vec4(color * diffuse, 1.0);
}

