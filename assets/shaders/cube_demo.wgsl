#import bevy_pbr::mesh_types
// The time since startup data is in the globals binding which is part of the mesh_view_bindings import
#import bevy_pbr::mesh_view_bindings

struct CustomMaterial {
    time: f32
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;

fn oklab_to_linear_srgb(c: vec3<f32>) -> vec3<f32> {
    let L = c.x;
    let a = c.y;
    let b = c.z;

    let l_ = L + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = L - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = L - 0.0894841775 * a - 1.2914855480 * b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    return vec3<f32>(
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    );
}

@fragment
fn fragment(
    @builtin(position) coord: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) normals: vec3<f32>,
    @location(2) uv: vec2<f32>
    ) -> @location(0) vec4<f32> {
    // var input1: vec2<f32> = vec2<f32>(world_position.x  , material.time);
    // var input2: vec2<f32> = vec2<f32>(world_position.y  , material.time);
    // var input3: vec2<f32> = vec2<f32>(world_position.z  , material.time);

    // var noise1 = 1.0 + (material.time / 10.0);
    // var noise2 = 0.5 + (material.time / 10.0);
    // var noise3 = 0.7 + (material.time / 10.0);

    // var value1 = (noise1 + 1.0) / 2.0;
    // var value2 = (noise2 + 1.0) / 2.0;
    // var value3 = (noise3 + 1.0) / 2.0;
    
    // return vec4<f32>(uv);
    let speed = 2.0;
    // The globals binding contains various global values like time
    // which is the time since startup in seconds
    let t_1 = sin(globals.time * speed) * 0.5 + 0.5;
    let t_2 = cos(globals.time * speed);

    let distance_to_center = distance(uv, vec2<f32>(0.5)) * 1.4;

    // blending is done in a perceptual color space: https://bottosson.github.io/posts/oklab/
    let red = vec3<f32>(0.627955, 0.224863, 0.125846);
    let green = vec3<f32>(0.86644, -0.233887, 0.179498);
    let blue = vec3<f32>(0.701674, 0.274566, -0.169156);
    let white = vec3<f32>(1.0, 0.0, 0.0);
    let mixed = mix(mix(red, blue, t_1), mix(green, white, t_2), distance_to_center);

    return vec4<f32>(oklab_to_linear_srgb(mixed), 1.0);
}
