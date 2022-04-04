shader_type spatial;

uniform sampler2D Matcap;
uniform sampler2D Grunge;

varying mat4 view_matrix;

uniform vec4 tint : hint_color;
uniform float grunge_scale;
uniform float grunge_amount;

void vertex() {
    view_matrix = WORLD_MATRIX;
}

void fragment() {

    

    vec2 muv = (view_matrix * vec4(normalize(NORMAL), 0)).xy * 0.5 + vec2(0.5,0.5);
    vec4 matcap = texture(Matcap, muv) * tint;
    
    vec4 grunge = texture(Grunge, UV * grunge_scale) * grunge_amount;
    
    ALBEDO = mix(matcap.rgb, matcap.rgb * grunge.rgb, grunge_amount);
    ROUGHNESS = 1.5; // Nonsensical, but has a nice darkening effect
}