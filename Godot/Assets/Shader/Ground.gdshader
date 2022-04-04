shader_type spatial;
render_mode blend_mix,depth_draw_opaque,cull_back,diffuse_burley,specular_schlick_ggx;

uniform sampler2D Splatmap;
uniform sampler2D Grass_A;
uniform sampler2D Blight_A;

uniform sampler2D Grass_N;
uniform sampler2D Blight_N;

uniform sampler2D Grass_R;
uniform sampler2D Blight_R;

uniform sampler2D Grass_H;
uniform sampler2D Blight_H;

uniform float Grass_Scale = 10.0;
uniform float Blight_Scale = 10.0;

uniform vec4 Blight_Tint : hint_color;
uniform vec4 Grass_Tint : hint_color;

uniform float Height_Blend_Factor = 1.0;

// From the excellent tutorial: http://untitledgam.es/2017/01/height-blending-shader/
vec3 heightblend(vec3 input1, float height1, vec3 input2, float height2) {
    float height_start = max(height1, height2) - Height_Blend_Factor;
    float level1 = max(height1 - height_start, 0);
    float level2 = max(height2 - height_start, 0);
    return ((input1 * level1) + (input2 * level2)) / (level1 + level2);
}

vec3 heightlerp(vec3 input1, float height1, vec3 input2, float height2, float t) {
    t = clamp(t, 0, 1);
    return heightblend(input1, height1 * (1.0 - t), input2, height2 * t);
}

varying vec3 untransformed_vertex;

void vertex() {
	untransformed_vertex = VERTEX;
}

void fragment() {
    vec4 splat = texture(Splatmap, UV.xy);
    
    vec4 grass_a = texture(Grass_A, UV.xy * Grass_Scale) * Grass_Tint;
    vec4 blight_a = texture(Blight_A, UV.xy * Blight_Scale) * Blight_Tint;
    
    vec4 grass_n = texture(Grass_N, UV.xy * Grass_Scale);
    vec4 blight_n = texture(Blight_N, UV.xy * Blight_Scale);
    
    vec4 grass_r = texture(Grass_R, UV.xy * Grass_Scale);
    vec4 blight_r = texture(Blight_R, UV.xy * Blight_Scale);
    
    vec4 grass_h = texture(Grass_H, UV.xy * Grass_Scale);
    vec4 blight_h = texture(Blight_H, UV.xy * Blight_Scale);
    
	ALBEDO = heightlerp(grass_a.xyz, grass_h.r, blight_a.xyz, blight_h.r, splat.r);
    NORMALMAP = heightlerp(grass_n.xyz, grass_h.r, blight_n.xyz, blight_h.r, splat.r);
    NORMALMAP_DEPTH = 3.0;
    ROUGHNESS = heightlerp(grass_r.xyz, grass_h.r, blight_r.xyz, blight_h.r, splat.r).r;
	
	
	float dist = (distance(untransformed_vertex, vec3(0.0, 0.0, 0.0))) / 100.0;
	float t = smoothstep(0.45, 0.5, dist);
	ALPHA = 1.0 - t;
	//ALBEDO.r = 1.0 - t;
}

void light() {
// Output:0

}
