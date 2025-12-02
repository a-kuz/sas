struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) world_pos: vec3<f32>,
}

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    camera_pos: vec3<f32>,
    light_pos0: vec3<f32>,
    light_color0: vec3<f32>,
    light_radius0: f32,
    light_pos1: vec3<f32>,
    light_color1: vec3<f32>,
    light_radius1: f32,
    num_lights: i32,
    ambient_light: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var model_texture: texture_2d<f32>;

@group(0) @binding(2)
var model_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    output.clip_position = uniforms.view_proj * world_pos;
    output.uv = input.uv;
    output.color = input.color;
    output.normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    output.world_pos = world_pos.xyz;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(model_texture, model_sampler, input.uv);
    
    var lighting = vec3<f32>(uniforms.ambient_light);
    
    if (uniforms.num_lights > 0) {
        let light_dir0 = normalize(uniforms.light_pos0 - input.world_pos);
        let dist0 = distance(input.world_pos, uniforms.light_pos0);
        let attenuation0 = pow(1.0 - min(dist0 / uniforms.light_radius0, 1.0), 1.6);
        let ndotl0 = max(dot(input.normal, light_dir0), 0.0);
        lighting += uniforms.light_color0 * ndotl0 * attenuation0;
    }
    
    if (uniforms.num_lights > 1) {
        let light_dir1 = normalize(uniforms.light_pos1 - input.world_pos);
        let dist1 = distance(input.world_pos, uniforms.light_pos1);
        let attenuation1 = pow(1.0 - min(dist1 / uniforms.light_radius1, 1.0), 1.6);
        let ndotl1 = max(dot(input.normal, light_dir1), 0.0);
        lighting += uniforms.light_color1 * ndotl1 * attenuation1;
    }
    
    let final_color = tex_color.rgb * lighting;
    return vec4<f32>(final_color, tex_color.a * input.color.a);
}

