use crate::surface::GfxError;

const VERT_PREAMBLE: &str = "#version 330 core\n#define attribute in\n#define varying out\n";

const FRAG_PREAMBLE: &str = "#version 330 core\n#define varying in\n\
                             #define texture2D texture\nout vec4 FRAG_COLOR;\n";

const VERT_PREAMBLE_ES: &str = "";
const FRAG_PREAMBLE_ES: &str = "#define FRAG_COLOR gl_FragColor\n";

pub fn program(vert: &str, frag: &str) -> Result<gl::types::GLuint, GfxError> {
    let (vp, fp) = match crate::gl::es() {
        true => (VERT_PREAMBLE_ES, FRAG_PREAMBLE_ES),
        false => (VERT_PREAMBLE, FRAG_PREAMBLE),
    };
    crate::gl::program(&format!("{vp}{vert}"), &format!("{fp}{frag}"))
}

pub const RECT_VERT: &str = r#"
attribute vec2 a_pos;
uniform vec4 u_rect;
uniform vec2 u_target;
varying vec2 v_uv;
void main() {
    v_uv = a_pos;
    vec2 p = (u_rect.xy + a_pos * u_rect.zw) / u_target;
    gl_Position = vec4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0);
}
"#;

pub const SPRITE_VERT: &str = r#"
attribute vec2 a_pos;
uniform vec4 u_rect;
uniform vec2 u_target;
uniform vec2 u_turn;
uniform vec4 u_src;
varying vec2 v_uv;
void main() {
    v_uv = u_src.xy + a_pos * u_src.zw;
    vec2 mid = u_rect.zw * 0.5;
    vec2 local = a_pos * u_rect.zw - mid;
    vec2 turned = vec2(u_turn.x * local.x - u_turn.y * local.y,
                       u_turn.y * local.x + u_turn.x * local.y);
    vec2 p = (u_rect.xy + a_pos * u_rect.zw + (turned - local)) / u_target;
    gl_Position = vec4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0);
}
"#;

pub const GAME_FRAG: &str = r#"
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
uniform sampler2D u_game;
uniform sampler2D u_paper;
uniform vec2 u_src;
uniform vec4 u_uv;
uniform float u_bright;
uniform float u_mode;
uniform float u_paper_size;
uniform vec2 u_origin;
uniform vec4 u_rect;
uniform vec2 u_fbo;
uniform vec4 u_pic;
varying vec2 v_uv;
const vec3 offsets = vec3(3.141592654) * vec3(1.0 / 2.0, 1.0 / 2.0 - 2.0 / 3.0, 1.0 / 2.0 - 4.0 / 3.0);
const vec3 luma_coeff = vec3(0.2126, 0.7152, 0.0722);
vec2 grid_band(vec2 x) {
    return floor(x) / 3.0 + min(fract(x), vec2(1.0 / 3.0));
}
float dot_weight(vec2 f, vec2 o, float bloom) {
    return exp(-4.0 * length(f - (o + vec2(0.5))) * bloom);
}
void main() {
    vec2 uv = u_uv.xy + v_uv * u_uv.zw;
    vec2 f = fract(uv * u_src);
    vec3 c = texture2D(u_game, uv).rgb;
    bool inside = all(greaterThanEqual(uv, u_pic.xy)) && all(lessThan(uv, u_pic.zw));
    if (!inside) {
    } else if (u_mode > 3.5) {
        c = pow(c, vec3(1.05));
        vec2 d = abs(f - 0.5);
        float x2 = max(d.x, d.y);
        x2 = x2 * x2;
        float x4 = x2 * x2;
        float w = 48.0 * (x4 - (8.0 / 3.0) * x4 * x2);
        w = w * (w + (1.0 - w) * 0.65) * 0.65;
        float luma = dot(c, luma_coeff);
        w = w * luma;
        c = mix(c, vec3(1.0), w);
        vec3 bg = vec3(texture2D(u_paper, (floor(gl_FragCoord.xy - u_origin) + 0.5) / u_paper_size).r) * c;
        c = mix(c, bg, dot(c, luma_coeff));
    } else if (u_mode > 2.5) {
        float bloom = mix(1.05, 0.95, dot(c, vec3(0.30, 0.59, 0.11)));
        float mid = dot_weight(f, vec2(0.0), bloom);
        float sum = 0.0;
        for (int j = -1; j <= 1; j++) {
            for (int i = -1; i <= 1; i++) {
                sum += dot_weight(f, vec2(float(i), float(j)), bloom);
            }
        }
        c = mix(c, mix(1.2 * c * mid, c * sum, 0.85), 0.85);
    } else if (u_mode > 1.5) {
        vec2 hp = 0.5 * u_uv.zw * u_src / (u_rect.zw * u_fbo);
        vec2 edge = (grid_band(f + hp) - grid_band(f - hp)) / (2.0 * hp);
        c *= 1.0 - 0.75 * max(edge.x, edge.y);
    } else if (u_mode > 0.5) {
        vec2 angle = f * 6.283185307;
        float yfactor = (16.0 + sin(angle.y)) / 17.0;
        vec3 xfactors = (4.0 + sin(angle.x + offsets)) / 5.0;
        c *= yfactor * xfactors;
    }
    FRAG_COLOR = vec4(c * u_bright, 1.0);
}
"#;

pub const SPRITE_FRAG: &str = r#"
precision mediump float;
uniform sampler2D u_tex;
uniform vec4 u_colour;
uniform float u_lava;
uniform vec3 u_round;
uniform vec4 u_flow0;
uniform vec4 u_flow1;
uniform vec4 u_flow2;
uniform vec4 u_flow3;
uniform vec4 u_flow4;
uniform vec4 u_flow5;
varying vec2 v_uv;
// Smooth metaball field and analytic gradient: blobs join and pinch apart
// without hard outlines or per-pixel trigonometry.
vec3 wax(vec2 p, vec4 flow) {
    vec2 q = (p - flow.xy) / flow.zw;
    float inv = 1.0 / (1.0 + dot(q, q));
    return vec3(inv * inv, -4.0 * q / flow.zw * inv * inv * inv);
}
vec2 eddy(vec2 p, vec4 flow) {
    vec2 q = p - flow.xy;
    return vec2(-q.y, q.x) / (0.06 + dot(q, q));
}
void main() {
    if (u_round.z > 0.0) {
        vec2 q = abs((v_uv - 0.5) * u_round.xy) - u_round.xy * 0.5 + u_round.z;
        float d = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - u_round.z;
        FRAG_COLOR = vec4(u_colour.rgb, u_colour.a * (1.0 - smoothstep(-0.7, 0.7, d)));
    } else if (u_lava < 0.5) {
        FRAG_COLOR = texture2D(u_tex, v_uv) * u_colour;
    } else {
        float calm = 1.0 - smoothstep(0.54, 0.90, v_uv.y);
        // Overlapping counter-rotating eddies bend the wax into flowing necks.
        vec2 p = v_uv + 0.010 * (eddy(v_uv, u_flow0) - eddy(v_uv, u_flow1)
                                      + 0.65 * eddy(v_uv, u_flow2));
        vec3 a = wax(p, u_flow0), b = wax(p, u_flow1), c = wax(p, u_flow2);
        vec3 d = wax(p, u_flow3), e = wax(p, u_flow4), f = wax(p, u_flow5);
        vec3 field = a + b + c + d + e + f;
        vec3 dye = (vec3(0.08, 0.65, 0.66) * (a.x + e.x)
                  + vec3(0.49, 0.20, 0.77) * (b.x + d.x)
                  + vec3(0.12, 0.58, 0.66) * c.x
                  + vec3(0.49, 0.20, 0.77) * f.x) / max(field.x, 0.001);
        float body = smoothstep(0.55, 1.02, field.x);
        vec3 normal = normalize(vec3(-field.yz * 0.045, 1.0));
        float diffuse = max(dot(normal, vec3(-0.45, -0.55, 0.704)), 0.0);
        float sheen = diffuse * diffuse; sheen *= sheen;
        vec3 liquid = dye * (0.22 + 0.48 * diffuse) + vec3(0.08, 0.10, 0.13) * sheen;
        vec3 glow = dye * min(field.x, 1.8) * 0.075;
        vec3 base = vec3(14.0, 21.0, 35.0) / 255.0;
        float grain = fract(dot(floor(v_uv * vec2(720.0, 480.0)), vec2(0.75487766, 0.56984029))) - 0.5;
        FRAG_COLOR = vec4(base + calm * (glow + liquid * body) + grain * (0.8 / 255.0), 1.0);
    }
}
"#;

pub const BLIT_VERT: &str = r#"
attribute vec2 a_pos;
uniform vec4 u_uv;
varying vec2 v_uv;
void main() {
    v_uv = u_uv.xy + a_pos * u_uv.zw;
    gl_Position = vec4(a_pos * 2.0 - 1.0, 0.0, 1.0);
}
"#;

pub const TURN_VERT: &str = r#"
attribute vec2 a_pos;
uniform float u_ccw;
varying vec2 v_uv;
void main() {
    vec2 cw = vec2(1.0 - a_pos.y, a_pos.x);
    vec2 ccw = vec2(a_pos.y, 1.0 - a_pos.x);
    v_uv = mix(cw, ccw, u_ccw);
    gl_Position = vec4(a_pos * 2.0 - 1.0, 0.0, 1.0);
}
"#;

pub const BLIT_FRAG: &str = r#"
precision mediump float;
uniform sampler2D u_tex;
uniform vec3 u_gain;
varying vec2 v_uv;
void main() {
    FRAG_COLOR = vec4(texture2D(u_tex, v_uv).rgb * u_gain, 1.0);
}
"#;
