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
    @location(2) world_pos: vec2<f32>,
}

struct Uniforms {
    view_proj: mat4x4<f32>,
    screen_to_world: vec2<f32>,
    camera_pos: vec2<f32>,
    map_size: vec2<f32>,
    tile_size: vec2<f32>,
    time: f32,
    num_dynamic_lights: i32,
    num_linear_lights: i32,
    disable_shadows: i32,
    ambient_light: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var scene_texture: texture_2d<f32>;

@group(0) @binding(2)
var scene_sampler: sampler;

@group(0) @binding(3)
var lightmap_texture: texture_2d<f32>;

@group(0) @binding(4)
var lightmap_sampler: sampler;

@group(0) @binding(5)
var dynamic_light_data: texture_2d<f32>;

@group(0) @binding(6)
var dynamic_light_sampler: sampler;

@group(0) @binding(7)
var linear_light_data: texture_2d<f32>;

@group(0) @binding(8)
var linear_light_sampler: sampler;

@group(0) @binding(9)
var obstacle_texture: texture_2d<f32>;

@group(0) @binding(10)
var obstacle_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.view_proj * vec4<f32>(input.position, 1.0);
    output.uv = input.uv;
    output.color = input.color;
    output.world_pos = input.uv * uniforms.screen_to_world + uniforms.camera_pos;
    return output;
}

fn hash(n: f32) -> f32 {
    return fract(sin(n) * 43758.5453);
}

fn sample_distance(world_pos: vec2<f32>, map_size: vec2<f32>) -> f32 {
    let uv = world_pos / map_size;
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return 1000.0;
    }
    
    let occ = textureSample(obstacle_texture, obstacle_sampler, uv).r;
    if (occ > 0.5) {
        return 0.0;
    }
    
    let px = 1.0 / map_size;
    var min_dist = 1000.0;
    
    for (var dy = -4; dy <= 4; dy++) {
        for (var dx = -4; dx <= 4; dx++) {
            let sample_uv = uv + vec2<f32>(f32(dx), f32(dy)) * px;
            let sample_occ = textureSample(obstacle_texture, obstacle_sampler, sample_uv).r;
            if (sample_occ > 0.5) {
                let sample_world = sample_uv * map_size;
                let dist = distance(world_pos, sample_world);
                min_dist = min(min_dist, dist);
            }
        }
    }
    
    return min_dist;
}

fn shadow(world_pos: vec2<f32>, light_pos: vec2<f32>, radius: f32, map_size: vec2<f32>) -> f32 {
    let maxt = distance(world_pos, light_pos);
    if (maxt < 2.0) {
        return 1.0;
    }
    
    let ld = normalize(light_pos - world_pos);
    var t = 0.5;
    var nd = 1000.0;
    
    let ds = 0.6;
    let smul = 1.5;
    let soff = 0.05;
    let tolerance = 0.5;
    
    for (var i = 0; i < 48; i++) {
        if (t >= maxt) {
            break;
        }
        
        let p = world_pos + ld * t;
        let d = sample_distance(p, map_size);
        
        if (d < tolerance) {
            let sd = 1.0 - exp(-smul * max(t / maxt - soff, 0.0));
            return sd;
        }
        
        nd = min(nd, d);
        t += ds * d;
    }
    
    let sd = 1.0 - exp(-smul * max(t / maxt - soff, 0.0));
    return mix(sd, 1.0, smoothstep(0.0, 2.0, nd));
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = input.world_pos;
    let scene_color = textureSample(scene_texture, scene_sampler, input.uv);
    
    var lighting = vec3<f32>(uniforms.ambient_light * 3.0);
    
    let lightmap_uv = world_pos / uniforms.map_size;
    let lightmap_color = textureSample(lightmap_texture, lightmap_sampler, lightmap_uv);
    lighting += lightmap_color.rgb;
    
    for (var i = 0; i < uniforms.num_dynamic_lights; i++) {
        let data1 = textureSample(dynamic_light_data, dynamic_light_sampler, vec2<f32>((f32(i) + 0.5) / 8.0, 0.25));
        let data2 = textureSample(dynamic_light_data, dynamic_light_sampler, vec2<f32>((f32(i) + 0.5) / 8.0, 0.75));
        
        let light_uv = data1.rg;
        let radius_norm = data1.b;
        let light_col = data2.rgb;
        let intensity = data1.a;
        let flicker_enabled = data2.a > 0.5;
        
        let light_pos = light_uv * uniforms.map_size;
        let radius = radius_norm * max(uniforms.map_size.x, uniforms.map_size.y) * 2.5;
        
        if (radius < 10.0) {
            continue;
        }
        
        let dist = distance(world_pos, light_pos);
        
        if (dist < radius) {
            var flicker = 1.0;
            if (flicker_enabled) {
                let t = uniforms.time * 8.0 + f32(i) * 2.3;
                flicker = 0.85 + hash(floor(t)) * 0.15;
                flicker += (sin(t * 17.0) * 0.5 + 0.5) * 0.05;
            }
            
            let attenuation = pow(1.0 - dist / radius, 1.6);
            var sh = 1.0;
            if (uniforms.disable_shadows == 0) {
                sh = shadow(world_pos, light_pos, radius, uniforms.map_size);
            }
            lighting += light_col * attenuation * sh * intensity * flicker * 2.0;
        }
    }
    
    var final_color = scene_color.rgb * lighting;
    final_color = final_color / (final_color + vec3<f32>(1.0));
    final_color = pow(final_color, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(final_color, scene_color.a);
}

