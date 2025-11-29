use macroquad::prelude::*;
use macroquad::miniquad;
use std::sync::OnceLock;

static MODEL_ADDITIVE_MATERIAL: OnceLock<Material> = OnceLock::new();
static SIMPLE_RECT_LIT_MATERIAL: OnceLock<Material> = OnceLock::new();
static OVERLAY_LIGHTING_MATERIAL: OnceLock<Material> = OnceLock::new();
static WASM_COMBINED_LIGHTING_MATERIAL: OnceLock<Material> = OnceLock::new();
static WASM_ADDITIVE_LIGHTING_MATERIAL: OnceLock<Material> = OnceLock::new();
static QUAD_DAMAGE_OUTLINE_MATERIAL: OnceLock<Material> = OnceLock::new();

pub fn create_model_additive_material() -> &'static Material {
    MODEL_ADDITIVE_MATERIAL.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;
        attribute vec4 normal;

        varying lowp vec2 uv;
        varying lowp vec4 color;
        varying lowp vec3 vNormal;
        varying mediump vec3 vWorldPos;

        uniform mat4 Model;
        uniform mat4 Projection;
        uniform vec2 cameraPos;

        void main() {
            vec4 pos = Projection * Model * vec4(position, 1.0);
            pos.z -= 0.0001 * pos.w;
            gl_Position = pos;
            color = color0 / 255.0;
            uv = texcoord;
            vNormal = normal.xyz;
            vWorldPos = vec3(position.xy + cameraPos, 0.0);
        }"#;

        let fragment_shader = r#"#version 100
        precision lowp float;

        varying lowp vec4 color;
        varying lowp vec2 uv;
        varying lowp vec3 vNormal;
        varying mediump vec3 vWorldPos;

        uniform sampler2D Texture;
        uniform vec3 lightPos0;
        uniform vec3 lightColor0;
        uniform float lightRadius0;
        uniform vec3 lightPos1;
        uniform vec3 lightColor1;
        uniform float lightRadius1;
        uniform int numLights;
        uniform float ambientLight;

        void main(){
            vec4 texColor = texture2D(Texture, uv) * color;
            vec3 lighting = vec3(1.0);
            
            gl_FragColor = vec4(texColor.rgb * lighting, 1.0);
        }"#;

        load_material(
            ShaderSource::Glsl { vertex: vertex_shader, fragment: fragment_shader },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("cameraPos", UniformType::Float2),
                    UniformDesc::new("lightPos0", UniformType::Float3),
                    UniformDesc::new("lightColor0", UniformType::Float3),
                    UniformDesc::new("lightRadius0", UniformType::Float1),
                    UniformDesc::new("lightPos1", UniformType::Float3),
                    UniformDesc::new("lightColor1", UniformType::Float3),
                    UniformDesc::new("lightRadius1", UniformType::Float1),
                    UniformDesc::new("numLights", UniformType::Int1),
                    UniformDesc::new("ambientLight", UniformType::Float1),
                ],
                pipeline_params: PipelineParams {
                    color_blend: Some(miniquad::BlendState::new(
                        miniquad::Equation::Add,
                        miniquad::BlendFactor::One,
                        miniquad::BlendFactor::One,
                    )),
                    depth_test: miniquad::Comparison::LessOrEqual,
                    depth_write: false,
                    cull_face: miniquad::CullFace::Back,
                    ..Default::default()
                },
                ..Default::default()
            },
        ).unwrap()
    })
}

pub fn create_muzzle_flash_material() -> Material {
    let vertex_shader = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;

    varying lowp vec2 uv;
    varying lowp vec4 color;

    uniform mat4 Model;
    uniform mat4 Projection;

    void main() {
        gl_Position = Projection * Model * vec4(position, 1);
        color = color0 / 255.0;
        uv = texcoord;
    }"#;

    let fragment_shader = r#"#version 100
    precision mediump float;
    
    varying lowp vec4 color;
    varying lowp vec2 uv;
    
    uniform sampler2D Texture;
    
    float random(vec2 st) {
        return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453123);
    }
    
    float noise(vec2 st) {
        vec2 i = floor(st);
        vec2 f = fract(st);
        float a = random(i);
        float b = random(i + vec2(1.0, 0.0));
        float c = random(i + vec2(0.0, 1.0));
        float d = random(i + vec2(1.0, 1.0));
        vec2 u = f * f * (3.0 - 2.0 * f);
        return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
    }
    
    void main() {
        vec2 pos = uv - vec2(0.5);
        float dist = length(pos);
        float angle = atan(pos.y, pos.x);
        
        float time = color.a * 100.0;
        float t = 1.0 - color.a;
        float pulse = 0.9 + 0.1 * sin(time * 20.0);
        
        float n1 = noise(vec2(angle * 3.0 + time * 4.0, dist * 8.0));
        float n2 = noise(vec2(angle * 5.0 - time * 6.0, dist * 12.0));
        float turbulence = (n1 * 0.6 + n2 * 0.4) * 0.3;
        
        float distorted = dist + turbulence * (1.0 - t);
        
        float core = 1.0 - smoothstep(0.0, 0.15, distorted);
        float mid = 1.0 - smoothstep(0.15, 0.45, distorted);
        float outer = 1.0 - smoothstep(0.45, 1.0, distorted);
        
        float rays = max(0.0, sin(angle * 6.0 + time * 10.0) * 0.5 + 0.5);
        outer *= mix(1.0, rays, 0.4);
        
        float brightness = core * 1.5 + mid * 0.8 + outer * 0.3;
        brightness *= (1.0 - t * 0.7);
        brightness *= pulse;
        
        vec3 innerColor = vec3(1.0, 1.0, 0.95);
        vec3 midColor = color.rgb;
        vec3 outerColor = color.rgb * 0.7;
        
        vec3 finalColor = mix(outerColor, midColor, mid);
        finalColor = mix(finalColor, innerColor, core);
        
        float alpha = brightness * (1.0 - t);
        
        gl_FragColor = vec4(finalColor, alpha);
    }"#;

    load_material(
        ShaderSource::Glsl {
            vertex: vertex_shader,
            fragment: fragment_shader,
        },
        MaterialParams::default(),
    ).unwrap()
}

pub fn create_deferred_lighting_material() -> Material {
    let vertex_shader = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;

    varying lowp vec2 uv;
    varying lowp vec4 color;

    uniform mat4 Model;
    uniform mat4 Projection;

    void main() {
        gl_Position = Projection * Model * vec4(position, 1);
        color = color0 / 255.0;
        uv = texcoord;
    }"#;

    let fragment_shader = r#"#version 100
    precision mediump float;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    
    uniform sampler2D sceneTexture;
    uniform sampler2D lightData;
    uniform sampler2D obstacleTex;
    uniform vec2 screenToWorld;
    uniform vec2 cameraPos;
    uniform vec2 mapSize;
    uniform float time;
    uniform float ambientLight;

    float hash(float n) {
        return fract(sin(n) * 43758.5453);
    }
    
    float sampleDistance(vec2 worldPos, vec2 mapSize) {
        vec2 uv = worldPos / mapSize;
        if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
            return 1000.0;
        }
        float occ = texture2D(obstacleTex, uv).r;
        if (occ > 0.5) {
            return 0.0;
        }
        
        vec2 px = 1.0 / mapSize;
        float minDist = 1000.0;
        
        for (int dy = -4; dy <= 4; dy++) {
            for (int dx = -4; dx <= 4; dx++) {
                vec2 sampleUV = uv + vec2(float(dx), float(dy)) * px;
                float sampleOcc = texture2D(obstacleTex, sampleUV).r;
                if (sampleOcc > 0.5) {
                    vec2 sampleWorld = sampleUV * mapSize;
                    float dist = distance(worldPos, sampleWorld);
                    minDist = min(minDist, dist);
                }
            }
        }
        
        return minDist;
    }
    
    float shadow(vec2 worldPos, vec2 lightPos, float radius, vec2 mapSize) {
        float maxt = distance(worldPos, lightPos);
        if (maxt < 2.0) return 1.0;
        
        vec2 ld = normalize(lightPos - worldPos);
        float t = 0.5;
        float nd = 1000.0;
        
        const float ds = 0.6;
        const float smul = 1.5;
        const float soff = 0.05;
        const float tolerance = 0.5;
        
        for (int i = 0; i < 48; i++) {
            if (t >= maxt) break;
            
            vec2 p = worldPos + ld * t;
            float d = sampleDistance(p, mapSize);
            
            if (d < tolerance) {
                float sd = 1.0 - exp(-smul * max(t / maxt - soff, 0.0));
                return sd;
            }
            
            nd = min(nd, d);
            t += ds * d;
        }
        
        float sd = 1.0 - exp(-smul * max(t / maxt - soff, 0.0));
        return mix(sd, 1.0, smoothstep(0.0, 2.0, nd));
    }

    void main() {
        vec2 worldPos = uv * screenToWorld + cameraPos;
        
        vec4 sceneColor = texture2D(sceneTexture, uv);
        
        vec3 lighting = vec3(ambientLight * 3.0);
        
        for (int i = 0; i < 8; i++) {
            vec4 data1 = texture2D(lightData, vec2((float(i) + 0.5) / 8.0, 0.25));
            vec4 data2 = texture2D(lightData, vec2((float(i) + 0.5) / 8.0, 0.75));
            
            vec2 lightUV = data1.rg;
            float radiusNorm = data1.b;
            vec3 lightCol = data2.rgb;
            float intensity = data1.a;
            float flicker_enabled = data2.a > 0.5 ? 1.0 : 0.0;
            
            vec2 lightPos = lightUV * mapSize;
            float radius = radiusNorm * max(mapSize.x, mapSize.y) * 2.5;
            
            if (radius < 10.0) continue;
            
            float dist = distance(worldPos, lightPos);
            
            if (dist < radius) {
                float flicker = 1.0;
                if (flicker_enabled > 0.5) {
                    float t = time * 8.0 + float(i) * 2.3;
                    flicker = 0.85 + hash(floor(t)) * 0.15;
                    flicker += (sin(t * 17.0) * 0.5 + 0.5) * 0.05;
                }
                
                float attenuation = pow(1.0 - dist / radius, 1.6);
                float sh = shadow(worldPos, normalize(lightPos - worldPos), 0.005, distance(worldPos, lightPos), mapSize, vec2(32.0, 16.0));
                lighting += lightCol * attenuation * sh * intensity * flicker * 2.0;
                lighting += vec3(1.0 - sh, 0.0, 0.0) * 1.0;
            }
        }
        
        vec3 finalColor = sceneColor.rgb * lighting;
        finalColor = finalColor / (finalColor + vec3(1.0));
        finalColor = pow(finalColor, vec3(1.0 / 2.2));
        
        gl_FragColor = vec4(finalColor, sceneColor.a);
    }"#;

    load_material(
        ShaderSource::Glsl { vertex: vertex_shader, fragment: fragment_shader },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("screenToWorld", UniformType::Float2),
                UniformDesc::new("cameraPos", UniformType::Float2),
                UniformDesc::new("mapSize", UniformType::Float2),
                UniformDesc::new("time", UniformType::Float1),
                UniformDesc::new("ambientLight", UniformType::Float1),
            ],
            textures: vec!["sceneTexture".to_string(), "lightData".to_string(), "obstacleTex".to_string()],
            ..Default::default()
        },
    ).unwrap()
}

pub fn create_hybrid_lighting_material() -> Material {
    let vertex_shader = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;

    varying lowp vec2 uv;
    varying lowp vec4 color;

    uniform mat4 Model;
    uniform mat4 Projection;

    void main() {
        gl_Position = Projection * Model * vec4(position, 1);
        color = color0 / 255.0;
        uv = texcoord;
    }"#;

    let fragment_shader = r#"#version 100
    precision mediump float;

    varying lowp vec4 color;
    varying lowp vec2 uv;

    uniform sampler2D sceneTexture;
    uniform sampler2D lightmapTexture;
    uniform sampler2D dynamicLightData;
    uniform sampler2D linearLightData;
    uniform sampler2D obstacleTex;
    uniform vec2 screenToWorld;
    uniform vec2 cameraPos;
    uniform vec2 mapSize;
    uniform vec2 tileSize;
    uniform float time;
    uniform int numDynamicLights;
    uniform int numLinearLights;
    uniform int disableShadows;

    float hash(float n) {
        return fract(sin(n) * 43758.5453);
    }
    
    float shadow(vec2 lp, vec2 ld, float mint, float maxt, vec2 mapSize, vec2 tileSize) {
        vec2 start = lp / tileSize;
        vec2 endp = (lp + ld * maxt) / tileSize;
        vec2 dir = endp - start;
        float totalDist = maxt;
        
        if (totalDist < 2.0) return 1.0;
        
        float sx = dir.x > 0.0 ? 1.0 : -1.0;
        float sy = dir.y > 0.0 ? 1.0 : -1.0;
        float invAbsDx = dir.x == 0.0 ? 1e9 : 1.0 / abs(dir.x);
        float invAbsDy = dir.y == 0.0 ? 1e9 : 1.0 / abs(dir.y);
        
        float cellX = floor(start.x);
        float cellY = floor(start.y);
        float fracX = start.x - cellX;
        float fracY = start.y - cellY;
        float tMaxX = (sx > 0.0 ? (1.0 - fracX) : fracX) * invAbsDx;
        float tMaxY = (sy > 0.0 ? (1.0 - fracY) : fracY) * invAbsDy;
        float tDeltaX = invAbsDx;
        float tDeltaY = invAbsDy;
        
        float t = 0.0;
        
        for (int i = 0; i < 64; i++) {
            vec2 cellCenter = (vec2(cellX, cellY) + 0.5) * tileSize;
            vec2 uv = cellCenter / mapSize;
            
            if (uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0) {
                float occ = texture2D(obstacleTex, uv).r;
                if (occ > 0.5) {
                    float traveled = t * totalDist;
                    return mix(0.02, 0.2, smoothstep(0.0, 100.0, traveled));
                }
            }
            
            if (tMaxX < tMaxY) {
                cellX += sx;
                t = tMaxX;
                tMaxX += tDeltaX;
            } else {
                cellY += sy;
                t = tMaxY;
                tMaxY += tDeltaY;
            }
            
            if (t > 1.0) break;
        }
        
        return 1.0;
    }
    
    float shadowWrapper(vec2 worldPos, vec2 lightPos, float radius, vec2 mapSize, vec2 tileSize) {
        if (disableShadows > 0) return 1.0;
        
        float maxt = distance(worldPos, lightPos);
        if (maxt < 2.0) return 1.0;
        
        vec2 ld = normalize(lightPos - worldPos);
        return shadow(worldPos, ld, 0.005, maxt, mapSize, tileSize);
    }
    
    float distanceToLineSegment(vec2 p, vec2 a, vec2 b) {
        vec2 pa = p - a;
        vec2 ba = b - a;
        float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
        return length(pa - ba * h);
    }

    void main() {
        vec2 worldPos = uv * screenToWorld + cameraPos;
        
        vec4 sceneColor = texture2D(sceneTexture, uv);
        
        vec2 lightmapUV = worldPos / mapSize;
        vec3 staticLighting = texture2D(lightmapTexture, lightmapUV).rgb;
        
        vec3 dynamicLighting = vec3(0.0);

        vec2 obsUV = worldPos / mapSize;
        vec2 px = 1.0 / mapSize;
        float occC = texture2D(obstacleTex, obsUV).r;
        float occU = texture2D(obstacleTex, obsUV + vec2(0.0, px.y)).r;
        float occD = texture2D(obstacleTex, obsUV - vec2(0.0, px.y)).r;
        float occL = texture2D(obstacleTex, obsUV - vec2(px.x, 0.0)).r;
        float occR = texture2D(obstacleTex, obsUV + vec2(px.x, 0.0)).r;
        vec2 surfN = vec2(0.0);
        if (occC > 0.5 && occU < 0.5) surfN += vec2(0.0, -1.0);
        if (occC > 0.5 && occD < 0.5) surfN += vec2(0.0,  1.0);
        if (occC > 0.5 && occL < 0.5) surfN += vec2(-1.0, 0.0);
        if (occC > 0.5 && occR < 0.5) surfN += vec2( 1.0, 0.0);
        float hasNormal = length(surfN);
        if (hasNormal > 0.0) surfN = normalize(surfN);
        
        for (int i = 0; i < 8; i++) {
            if (i >= numDynamicLights) break;
            
            vec4 data1 = texture2D(dynamicLightData, vec2((float(i) + 0.5) / 8.0, 0.25));
            vec4 data2 = texture2D(dynamicLightData, vec2((float(i) + 0.5) / 8.0, 0.75));
            
            vec2 lightUV = data1.rg;
            float radiusNorm = data1.b;
            vec3 lightCol = data2.rgb;
            float intensity = data1.a;
            
            vec2 lightPos = lightUV * mapSize;
            float radius = radiusNorm * max(mapSize.x, mapSize.y) * 2.5;
            
            if (radius < 10.0) continue;
            
            float dist = distance(worldPos, lightPos);
            
            if (dist < radius) {
                float attenuation = pow(1.0 - dist / radius, 1.6);
                float sh = shadowWrapper(worldPos, lightPos, radius, mapSize, tileSize);
                vec2 L = normalize(lightPos - worldPos);
                float facing = hasNormal > 0.0 ? max(dot(surfN, L), 0.0) : 1.0;
                float facingTerm = mix(0.6, 1.0, facing);
                dynamicLighting += lightCol * attenuation * sh * intensity * 2.0 * facingTerm;

            }
        }
        
        // Linear lights (railgun beams)
        for (int i = 0; i < 4; i++) {
            if (i >= numLinearLights) break;
            
            vec4 data1 = texture2D(linearLightData, vec2((float(i) + 0.5) / 4.0, 0.25));
            vec4 data2 = texture2D(linearLightData, vec2((float(i) + 0.5) / 4.0, 0.75));
            
            vec2 startPos = data1.rg * mapSize;
            vec2 endPos = data1.ba * mapSize;
            float width = data2.a * max(mapSize.x, mapSize.y);
            vec3 lightCol = data2.rgb;
            
            if (width < 1.0) continue;
            
            float dist = distanceToLineSegment(worldPos, startPos, endPos);
            
            // Экспоненциальное затухание как у настоящего света
            float attenuation = exp(-dist * dist / (width * width * 0.5));
            
            if (attenuation > 0.01) {
                vec2 beamCenter = mix(startPos, endPos, 0.5);
                float sh = shadowWrapper(worldPos, beamCenter, width, mapSize, tileSize);
                
                dynamicLighting += lightCol * attenuation * sh * 8.0;
            }
        }
        
        vec3 totalLighting = staticLighting + dynamicLighting;
        
        vec3 finalColor = sceneColor.rgb * totalLighting;
        finalColor = finalColor / (finalColor + vec3(1.0));
        finalColor = pow(finalColor, vec3(1.0 / 2.2));
        
        gl_FragColor = vec4(finalColor, sceneColor.a);
    }"#;

    load_material(
        ShaderSource::Glsl { vertex: vertex_shader, fragment: fragment_shader },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("screenToWorld", UniformType::Float2),
                UniformDesc::new("cameraPos", UniformType::Float2),
                UniformDesc::new("mapSize", UniformType::Float2),
                UniformDesc::new("tileSize", UniformType::Float2),
                UniformDesc::new("time", UniformType::Float1),
                UniformDesc::new("numDynamicLights", UniformType::Int1),
                UniformDesc::new("numLinearLights", UniformType::Int1),
                UniformDesc::new("disableShadows", UniformType::Int1),
            ],
            textures: vec![
                "sceneTexture".to_string(), 
                "lightmapTexture".to_string(),
                "dynamicLightData".to_string(),
                "linearLightData".to_string(),
                "obstacleTex".to_string()
            ],
            ..Default::default()
        },
    ).unwrap()
}


pub fn create_model_lit_material() -> Material {
    let vertex_shader = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;
    attribute vec4 normal;

    varying lowp vec2 uv;
    varying lowp vec4 color;
    varying lowp vec3 vNormal;
    varying mediump vec3 vWorldPos;

    uniform mat4 Model;
    uniform mat4 Projection;
    uniform vec2 cameraPos;

    void main() {
        vec4 pos = Projection * Model * vec4(position, 1.0);
        pos.z -= 0.0001 * pos.w;
        gl_Position = pos;
        color = color0 / 255.0;
        uv = texcoord;
        vNormal = normalize(normal.xyz);
        vWorldPos = vec3(position.xy + cameraPos, 0.0);
    }"#;

    let fragment_shader = r#"#version 100
    precision lowp float;

    varying lowp vec4 color;
    varying lowp vec2 uv;
    varying lowp vec3 vNormal;
    varying mediump vec3 vWorldPos;

    uniform sampler2D Texture;
    uniform vec3 lightPos0;
    uniform vec3 lightColor0;
    uniform float lightRadius0;
    uniform vec3 lightPos1;
    uniform vec3 lightColor1;
    uniform float lightRadius1;
    uniform int numLights;
    uniform float ambientLight;

    void main(){
        vec4 texColor = texture2D(Texture, uv) * color;
        
        if (texColor.a < 0.01) {
            discard;
        }
        
        vec3 lighting = vec3(ambientLight);

        if(numLights > 0){
            vec3 L = lightPos0 - vWorldPos;
            float distSq = dot(L, L);
            float invRadius = 1.0 / lightRadius0;
            float radiusSq = lightRadius0 * lightRadius0;
            if(distSq < radiusSq){
                float dist = sqrt(distSq);
                L *= 1.0 / dist;
                float NdotL = max(dot(vNormal, L), 0.0);
                float att = 1.0 - dist * invRadius;
                lighting += lightColor0 * (NdotL * att * att);
            }
        }
        
        if(numLights > 1){
            vec3 L = lightPos1 - vWorldPos;
            float distSq = dot(L, L);
            float invRadius = 1.0 / lightRadius1;
            float radiusSq = lightRadius1 * lightRadius1;
            if(distSq < radiusSq){
                float dist = sqrt(distSq);
                L *= 1.0 / dist;
                float NdotL = max(dot(vNormal, L), 0.0);
                float att = 1.0 - dist * invRadius;
                lighting += lightColor1 * (NdotL * att * att);
            }
        }

        gl_FragColor = vec4(texColor.rgb * lighting, texColor.a);
    }"#;

    load_material(
        ShaderSource::Glsl { vertex: vertex_shader, fragment: fragment_shader },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("cameraPos", UniformType::Float2),
                UniformDesc::new("lightPos0", UniformType::Float3),
                UniformDesc::new("lightColor0", UniformType::Float3),
                UniformDesc::new("lightRadius0", UniformType::Float1),
                UniformDesc::new("lightPos1", UniformType::Float3),
                UniformDesc::new("lightColor1", UniformType::Float3),
                UniformDesc::new("lightRadius1", UniformType::Float1),
                UniformDesc::new("numLights", UniformType::Int1),
                UniformDesc::new("ambientLight", UniformType::Float1),
            ],
            pipeline_params: PipelineParams {
                depth_test: miniquad::Comparison::LessOrEqual,
                depth_write: true,
                cull_face: miniquad::CullFace::Back,
                ..Default::default()
            },
            ..Default::default()
        },
    ).unwrap()
}

pub fn create_railgun_beam_material() -> Material {
    let vertex_shader = r#"#version 100
    attribute vec3 position;
    attribute vec2 texcoord;
    attribute vec4 color0;

    varying lowp vec2 uv;
    varying lowp vec4 color;

    uniform mat4 Model;
    uniform mat4 Projection;

    void main() {
        gl_Position = Projection * Model * vec4(position, 1);
        color = color0 / 255.0;
        uv = texcoord;
    }"#;

    let fragment_shader = r#"#version 100
    precision mediump float;
    
    varying lowp vec4 color;
    varying lowp vec2 uv;
    
    uniform float time;
    uniform vec2 screenSize;
    uniform vec2 beamStart;
    uniform vec2 beamEnd;
    uniform float beamWidth;
    uniform float fade;
    uniform vec2 cameraPos;
    
    float random(vec2 st) {
        return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453123);
    }
    
    float noise(vec2 st) {
        vec2 i = floor(st);
        vec2 f = fract(st);
        float a = random(i);
        float b = random(i + vec2(1.0, 0.0));
        float c = random(i + vec2(0.0, 1.0));
        float d = random(i + vec2(1.0, 1.0));
        vec2 u = f * f * (3.0 - 2.0 * f);
        return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
    }
    
    float distanceToLine(vec2 p, vec2 a, vec2 b) {
        vec2 pa = p - a;
        vec2 ba = b - a;
        float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
        return length(pa - ba * h);
    }
    
    void main() {
        vec2 screenPos = gl_FragCoord.xy;
        vec2 worldPos = screenPos + cameraPos;
        
        vec2 worldBeamStart = beamStart;
        vec2 worldBeamEnd = beamEnd;
        
        float dist = distanceToLine(worldPos, worldBeamStart, worldBeamEnd);
        
        vec2 beamDir = worldBeamEnd - worldBeamStart;
        float beamLength = length(beamDir);
        beamDir = normalize(beamDir);
        
        vec2 toPoint = worldPos - worldBeamStart;
        float alongBeam = dot(toPoint, beamDir);
        float t = clamp(alongBeam / beamLength, 0.0, 1.0);
        
        if (t < 0.0 || t > 1.0) {
            gl_FragColor = vec4(0.0);
            return;
        }
        
        float coreRadius = beamWidth * 0.4;
        float midRadius = beamWidth * 0.7;
        float outerRadius = beamWidth;
        
        float timeOffset = time * 12.0;
        float energyPulse = 0.85 + 0.15 * sin(timeOffset * 2.0 + t * 15.0);
        
        float n1 = noise(vec2(t * 30.0 + timeOffset, dist * 0.1));
        float n2 = noise(vec2(t * 20.0 - timeOffset * 0.8, dist * 0.15));
        float energy = (n1 * 0.7 + n2 * 0.3) * 0.4 + 0.6;
        
        float core = 1.0 - smoothstep(0.0, coreRadius, dist);
        float mid = (1.0 - smoothstep(coreRadius, midRadius, dist)) * (1.0 - core);
        float outer = (1.0 - smoothstep(midRadius, outerRadius, dist)) * (1.0 - core - mid);
        
        float brightness = core * 3.0 + mid * 1.5 + outer * 0.8;
        brightness *= energy * energyPulse * fade;
        
        vec3 coreColor = vec3(1.0, 1.0, 1.0);
        vec3 midColor = color.rgb * 1.3;
        vec3 outerColor = color.rgb * 0.6;
        
        vec3 finalColor = outerColor * outer + midColor * mid + coreColor * core;
        
        float alpha = brightness;
        
        gl_FragColor = vec4(finalColor, alpha);
    }"#;

    load_material(
        ShaderSource::Glsl {
            vertex: vertex_shader,
            fragment: fragment_shader,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("time", UniformType::Float1),
                UniformDesc::new("screenSize", UniformType::Float2),
                UniformDesc::new("beamStart", UniformType::Float2),
                UniformDesc::new("beamEnd", UniformType::Float2),
                UniformDesc::new("beamWidth", UniformType::Float1),
                UniformDesc::new("fade", UniformType::Float1),
                UniformDesc::new("cameraPos", UniformType::Float2),
            ],
            ..Default::default()
        },
    ).unwrap()
}

pub fn create_overlay_lighting_material() -> &'static Material {
    OVERLAY_LIGHTING_MATERIAL.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;
        
        varying lowp vec2 uv;
        varying lowp vec4 color;
        
        uniform mat4 Model;
        uniform mat4 Projection;
        
        void main() {
            gl_Position = Projection * Model * vec4(position, 1.0);
            color = color0 / 255.0;
            uv = texcoord;
        }"#;
        
        let fragment_shader = r#"#version 100
        precision mediump float;
        
        varying lowp vec2 uv;
        varying lowp vec4 color;
        
        uniform sampler2D lightmapTexture;
        uniform sampler2D lightData;
        uniform sampler2D linearLightData;
        uniform sampler2D obstacleTex;
        uniform vec2 screenToWorld;
        uniform vec2 cameraPos;
        uniform vec2 screenSize;
        uniform vec2 rectOrigin;
        uniform vec2 rectSize;
        uniform vec2 mapSize;
        uniform vec2 tileSize;
        uniform float time;
        uniform int numLights;
        uniform int numLinearLights;
        uniform int outputMode;
        uniform float lightingGamma;
        uniform float lightingGain;
        
        float hash(float n) {
            return fract(sin(n) * 43758.5453);
        }
        
        float sampleDistance(vec2 worldPos, vec2 mapSize) {
            vec2 uv = worldPos / mapSize;
            if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
                return 1000.0;
            }
            float occ = texture2D(obstacleTex, uv).r;
            if (occ > 0.5) {
                return 0.0;
            }
            
            vec2 px = 1.0 / mapSize;
            float minDist = 1000.0;
            
            for (int dy = -4; dy <= 4; dy++) {
                for (int dx = -4; dx <= 4; dx++) {
                    vec2 sampleUV = uv + vec2(float(dx), float(dy)) * px;
                    float sampleOcc = texture2D(obstacleTex, sampleUV).r;
                    if (sampleOcc > 0.5) {
                        vec2 sampleWorld = sampleUV * mapSize;
                        float dist = distance(worldPos, sampleWorld);
                        minDist = min(minDist, dist);
                    }
                }
            }
            
            return minDist;
        }
        
        float shadow(vec2 worldPos, vec2 lightPos, float radius, vec2 mapSize, vec2 tileSize) {
            float maxt = distance(worldPos, lightPos);
            if (maxt < 2.0) return 1.0;
            
            vec2 ld = normalize(lightPos - worldPos);
            float t = 0.5;
            float nd = 1000.0;
            
            const float ds = 0.6;
            const float smul = 1.5;
            const float soff = 0.05;
            const float tolerance = 0.5;
            
            for (int i = 0; i < 40; i++) {
                if (t >= maxt) break;
                
                vec2 p = worldPos + ld * t;
                float d = sampleDistance(p, mapSize);
                
                if (d < tolerance) {
                    float sd = 1.0 - exp(-smul * max(t / maxt - soff, 0.0));
                    return sd;
                }
                
                nd = min(nd, d);
                t += ds * d;
            }
            
            float sd = 1.0 - exp(-smul * max(t / maxt - soff, 0.0));
            return mix(sd, 1.0, smoothstep(0.0, 2.0, nd));
        }
        
        float distanceToLineSegment(vec2 p, vec2 a, vec2 b) {
            vec2 pa = p - a;
            vec2 ba = b - a;
            float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
            return length(pa - ba * h);
        }
        
        void main() {
            vec2 uvAbs = (rectOrigin + uv * rectSize) / screenSize;
            vec2 worldPos = uvAbs * screenToWorld + cameraPos;
            vec2 lightmapUV = worldPos / mapSize;
            
            vec3 staticLighting = texture2D(lightmapTexture, lightmapUV).rgb;
            vec3 dynamicLighting = vec3(0.0);

            if (outputMode != 1) {
            
            vec2 obsUV = worldPos / mapSize;
            vec2 px = 1.0 / mapSize;
            float occC = texture2D(obstacleTex, obsUV).r;
            float occU = texture2D(obstacleTex, obsUV + vec2(0.0, px.y)).r;
            float occD = texture2D(obstacleTex, obsUV - vec2(0.0, px.y)).r;
            float occL = texture2D(obstacleTex, obsUV - vec2(px.x, 0.0)).r;
            float occR = texture2D(obstacleTex, obsUV + vec2(px.x, 0.0)).r;
            vec2 surfN = vec2(0.0);
            if (occC > 0.5 && occU < 0.5) surfN += vec2(0.0, -1.0);
            if (occC > 0.5 && occD < 0.5) surfN += vec2(0.0,  1.0);
            if (occC > 0.5 && occL < 0.5) surfN += vec2(-1.0, 0.0);
            if (occC > 0.5 && occR < 0.5) surfN += vec2( 1.0, 0.0);
            float hasNormal = length(surfN);
            if (hasNormal > 0.0) surfN = normalize(surfN);
            
            for (int i = 0; i < 8; i++) {
                if (i >= numLights) break;
                
                vec4 data1 = texture2D(lightData, vec2((float(i) + 0.5) / 8.0, 0.25));
                vec4 data2 = texture2D(lightData, vec2((float(i) + 0.5) / 8.0, 0.75));
                
                vec2 lightUV = data1.rg;
                float radiusNorm = data1.b;
                vec3 lightCol = data2.rgb;
                float intensity = data1.a;
                float flicker_enabled = data2.a > 0.5 ? 1.0 : 0.0;
                
                vec2 lightPos = lightUV * mapSize;
                float radius = radiusNorm * max(mapSize.x, mapSize.y) * 2.5;
                
                if (radius < 10.0) continue;
                
                float dist = distance(worldPos, lightPos);
                
                if (dist < radius) {
                    float flicker = 1.0;
                    if (flicker_enabled > 0.5) {
                        float t = time * 8.0 + float(i) * 2.3;
                        flicker = 0.85 + hash(floor(t)) * 0.15;
                        flicker += (sin(t * 17.0) * 0.5 + 0.5) * 0.05;
                    }
                    
                    float attenuation = pow(1.0 - dist / radius, 1.6);
                    float sh = shadowWrapper(worldPos, lightPos, radius, mapSize, tileSize);
                    vec2 L = normalize(lightPos - worldPos);
                    float facing = hasNormal > 0.0 ? max(dot(surfN, L), 0.0) : 1.0;
                    float facingTerm = mix(0.6, 1.0, facing);
                    dynamicLighting += lightCol * attenuation * sh * intensity * flicker * 2.0 * facingTerm;
                    dynamicLighting += vec3(0.0, 1.0 - sh, 0.0) * 1.0;
                }
            }
            
            for (int i = 0; i < 4; i++) {
                if (i >= numLinearLights) break;
                
                vec4 data1 = texture2D(linearLightData, vec2((float(i) + 0.5) / 4.0, 0.25));
                vec4 data2 = texture2D(linearLightData, vec2((float(i) + 0.5) / 4.0, 0.75));
                
                vec2 startPos = data1.rg * mapSize;
                vec2 endPos = data1.ba * mapSize;
                float width = data2.a * max(mapSize.x, mapSize.y);
                vec3 lightCol = data2.rgb;
                
                if (width < 1.0) continue;
                
                float dist = distanceToLineSegment(worldPos, startPos, endPos);
                float attenuation = exp(-dist * dist / (width * width * 0.5));
                
                if (attenuation > 0.01) {
                    float sh = shadow(worldPos, mix(startPos, endPos, 0.5), width, mapSize, tileSize);
                    dynamicLighting += lightCol * attenuation * sh * 8.0;
                }
            }
            
            }

            vec3 totalLighting = staticLighting + dynamicLighting;
            
            vec3 outColor;
            if (outputMode == 1) {
                outColor = staticLighting;
            } else if (outputMode == 2) {
                vec3 baseLighting = max(staticLighting, vec3(0.001));
                outColor = totalLighting / baseLighting;
            } else if (outputMode == 3) {
                outColor = dynamicLighting;
            } else {
                outColor = totalLighting;
            }
            outColor = pow(max(outColor * lightingGain, vec3(0.0)), vec3(1.0 / max(lightingGamma, 0.0001)));
            gl_FragColor = vec4(outColor, 1.0);
        }"#;
        
        load_material(
            ShaderSource::Glsl {
                vertex: vertex_shader,
                fragment: fragment_shader,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("screenToWorld", UniformType::Float2),
                    UniformDesc::new("cameraPos", UniformType::Float2),
                    UniformDesc::new("screenSize", UniformType::Float2),
                    UniformDesc::new("rectOrigin", UniformType::Float2),
                    UniformDesc::new("rectSize", UniformType::Float2),
                    UniformDesc::new("mapSize", UniformType::Float2),
                    UniformDesc::new("tileSize", UniformType::Float2),
                    UniformDesc::new("time", UniformType::Float1),
                    UniformDesc::new("numLights", UniformType::Int1),
                    UniformDesc::new("numLinearLights", UniformType::Int1),
                    UniformDesc::new("outputMode", UniformType::Int1),
            ],
            textures: vec!["lightmapTexture".to_string(), "lightData".to_string(), "linearLightData".to_string(), "obstacleTex".to_string()],
                pipeline_params: PipelineParams {
                    color_blend: Some(miniquad::BlendState::new(
                        miniquad::Equation::Add,
                        miniquad::BlendFactor::Zero,
                        miniquad::BlendFactor::Value(miniquad::BlendValue::SourceColor),
                    )),
                ..Default::default()
                },
                ..Default::default()
            },
        ).unwrap()
    })
}

pub fn create_wasm_combined_lighting_material() -> &'static Material {
    WASM_COMBINED_LIGHTING_MATERIAL.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;

        varying lowp vec2 uv;
        varying lowp vec4 color;

        uniform mat4 Model;
        uniform mat4 Projection;

        void main() {
            gl_Position = Projection * Model * vec4(position, 1);
            color = color0 / 255.0;
            uv = texcoord;
        }"#;

        let fragment_shader = r#"#version 100
        precision mediump float;

        varying lowp vec4 color;
        varying lowp vec2 uv;

        uniform sampler2D lightmapTexture;
        uniform sampler2D lightData;
        uniform sampler2D linearLightData;
        uniform sampler2D obstacleTex;
        uniform vec2 screenToWorld;
        uniform vec2 screenSize;
        uniform vec2 cameraPos;
        uniform vec2 rectOrigin;
        uniform vec2 rectSize;
        uniform vec2 mapSize;
        uniform vec2 tileSize;
        uniform float time;
        uniform int numLights;
        uniform int numLinearLights;
        uniform int outputMode;
        uniform float lightingGamma;
        uniform float lightingGain;

        float hash(float n) {
            return fract(sin(n) * 43758.5453);
        }
        
        float sampleDistance(vec2 worldPos, vec2 mapSize) {
            vec2 uv = worldPos / mapSize;
            if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
                return 1000.0;
            }
            float occ = texture2D(obstacleTex, uv).r;
            if (occ > 0.5) {
                return 0.0;
            }
            
            vec2 px = 1.0 / mapSize;
            float minDist = 1000.0;
            
            for (int dy = -4; dy <= 4; dy++) {
                for (int dx = -4; dx <= 4; dx++) {
                    vec2 sampleUV = uv + vec2(float(dx), float(dy)) * px;
                    float sampleOcc = texture2D(obstacleTex, sampleUV).r;
                    if (sampleOcc > 0.5) {
                        vec2 sampleWorld = sampleUV * mapSize;
                        float dist = distance(worldPos, sampleWorld);
                        minDist = min(minDist, dist);
                    }
                }
            }
            
            return minDist;
        }
        
        float shadow(vec2 worldPos, vec2 lightPos, float radius, vec2 mapSize, vec2 tileSize) {
            float maxt = distance(worldPos, lightPos);
            if (maxt < 2.0) return 1.0;
            
            vec2 ld = normalize(lightPos - worldPos);
            float t = 0.5;
            float nd = 1000.0;
            
            const float ds = 0.6;
            const float smul = 1.5;
            const float soff = 0.05;
            const float tolerance = 0.5;
            
            for (int i = 0; i < 40; i++) {
                if (t >= maxt) break;
                
                vec2 p = worldPos + ld * t;
                float d = sampleDistance(p, mapSize);
                
                if (d < tolerance) {
                    float sd = 1.0 - exp(-smul * max(t / maxt - soff, 0.0));
                    return sd;
                }
                
                nd = min(nd, d);
                t += ds * d;
            }
            
            float sd = 1.0 - exp(-smul * max(t / maxt - soff, 0.0));
            return mix(sd, 1.0, smoothstep(0.0, 2.0, nd));
        }
        
        float distanceToLineSegment(vec2 p, vec2 a, vec2 b) {
            vec2 pa = p - a;
            vec2 ba = b - a;
            float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
            return length(pa - ba * h);
        }

        void main() {
            vec2 uvAbs = (rectOrigin + uv * rectSize) / screenSize;
            vec2 worldPos = uvAbs * screenToWorld + cameraPos;
            vec2 lightmapUV = worldPos / mapSize;
            vec3 staticLighting = texture2D(lightmapTexture, lightmapUV).rgb;
            
            vec3 dynamicLighting = vec3(0.0);
            
            if (outputMode != 1) {

            vec2 obsUV = worldPos / mapSize;
            vec2 px = 1.0 / mapSize;
            float occC = texture2D(obstacleTex, obsUV).r;
            float occU = texture2D(obstacleTex, obsUV + vec2(0.0, px.y)).r;
            float occD = texture2D(obstacleTex, obsUV - vec2(0.0, px.y)).r;
            float occL = texture2D(obstacleTex, obsUV - vec2(px.x, 0.0)).r;
            float occR = texture2D(obstacleTex, obsUV + vec2(px.x, 0.0)).r;
            vec2 surfN = vec2(0.0);
            if (occC > 0.5 && occU < 0.5) surfN += vec2(0.0, -1.0);
            if (occC > 0.5 && occD < 0.5) surfN += vec2(0.0,  1.0);
            if (occC > 0.5 && occL < 0.5) surfN += vec2(-1.0, 0.0);
            if (occC > 0.5 && occR < 0.5) surfN += vec2( 1.0, 0.0);
            float hasNormal = length(surfN);
            if (hasNormal > 0.0) surfN = normalize(surfN);
            
            for (int i = 0; i < 8; i++) {
                if (i >= numLights) break;
                
                vec4 data1 = texture2D(lightData, vec2((float(i) + 0.5) / 8.0, 0.25));
                vec4 data2 = texture2D(lightData, vec2((float(i) + 0.5) / 8.0, 0.75));
                
                vec2 lightUV = data1.rg;
                float radiusNorm = data1.b;
                vec3 lightCol = data2.rgb;
                float intensity = data1.a;
                float flicker_enabled = data2.a > 0.5 ? 1.0 : 0.0;
                
                vec2 lightPos = lightUV * mapSize;
                float radius = radiusNorm * max(mapSize.x, mapSize.y) * 2.5;
                
                if (radius < 10.0) continue;
                
                float dist = distance(worldPos, lightPos);
                
                if (dist < radius) {
                    float flicker = 1.0;
                    if (flicker_enabled > 0.5) {
                        float t = time * 8.0 + float(i) * 2.3;
                        flicker = 0.85 + hash(floor(t)) * 0.15;
                        flicker += (sin(t * 17.0) * 0.5 + 0.5) * 0.05;
                    }
                    
                    float attenuation = pow(1.0 - dist / radius, 1.6);
                    float sh = shadowWrapper(worldPos, lightPos, radius, mapSize, tileSize);
                    vec2 L = normalize(lightPos - worldPos);
                    float facing = hasNormal > 0.0 ? max(dot(surfN, L), 0.0) : 1.0;
                    float facingTerm = mix(0.6, 1.0, facing);
                    dynamicLighting += lightCol * attenuation * sh * intensity * flicker * 2.0 * facingTerm;
                    dynamicLighting += vec3(0.0, 1.0 - sh, 0.0) * 1.0;
                }
            }
            
            for (int i = 0; i < 4; i++) {
                if (i >= numLinearLights) break;
                
                vec4 data1 = texture2D(linearLightData, vec2((float(i) + 0.5) / 4.0, 0.25));
                vec4 data2 = texture2D(linearLightData, vec2((float(i) + 0.5) / 4.0, 0.75));
                
                vec2 startPos = data1.rg * mapSize;
                vec2 endPos = data1.ba * mapSize;
                float width = data2.a * max(mapSize.x, mapSize.y);
                vec3 lightCol = data2.rgb;
                
                if (width < 1.0) continue;
                
                float dist = distanceToLineSegment(worldPos, startPos, endPos);
                float attenuation = exp(-dist * dist / (width * width * 0.5));
                
                if (attenuation > 0.01) {
                    float sh = shadow(worldPos, mix(startPos, endPos, 0.5), width, mapSize, tileSize);
                    dynamicLighting += lightCol * attenuation * sh * 8.0;
                }
            }
            
            }
            
            vec3 totalLighting = staticLighting + dynamicLighting;
            
            vec3 outColor;
            if (outputMode == 1) {
                outColor = staticLighting;
            } else if (outputMode == 2) {
                vec3 baseLighting = max(staticLighting, vec3(0.001));
                outColor = totalLighting / baseLighting;
            } else if (outputMode == 3) {
                outColor = dynamicLighting;
            } else {
                outColor = totalLighting;
            }
            outColor = pow(max(outColor * lightingGain, vec3(0.0)), vec3(1.0 / max(lightingGamma, 0.0001)));
            
            gl_FragColor = vec4(outColor, 1.0);
        }"#;

        load_material(
            ShaderSource::Glsl { vertex: vertex_shader, fragment: fragment_shader },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("screenToWorld", UniformType::Float2),
                    UniformDesc::new("screenSize", UniformType::Float2),
                    UniformDesc::new("cameraPos", UniformType::Float2),
                    UniformDesc::new("rectOrigin", UniformType::Float2),
                    UniformDesc::new("rectSize", UniformType::Float2),
                    UniformDesc::new("mapSize", UniformType::Float2),
                    UniformDesc::new("tileSize", UniformType::Float2),
                    UniformDesc::new("time", UniformType::Float1),
                    UniformDesc::new("numLights", UniformType::Int1),
                    UniformDesc::new("numLinearLights", UniformType::Int1),
                    UniformDesc::new("outputMode", UniformType::Int1),
                    UniformDesc::new("lightingGamma", UniformType::Float1),
                    UniformDesc::new("lightingGain", UniformType::Float1),
                ],
                textures: vec![
                    "lightmapTexture".to_string(),
                    "lightData".to_string(),
                    "linearLightData".to_string(),
                    "obstacleTex".to_string(),
                ],
                pipeline_params: PipelineParams {
                    color_blend: Some(miniquad::BlendState::new(
                        miniquad::Equation::Add,
                        miniquad::BlendFactor::Zero,
                        miniquad::BlendFactor::Value(miniquad::BlendValue::SourceColor),
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        ).unwrap()
    })
}

static CARTOON_SHADER_MATERIAL: OnceLock<Material> = OnceLock::new();

pub fn create_cartoon_shader_material() -> &'static Material {
    CARTOON_SHADER_MATERIAL.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;
        
        varying lowp vec2 uv;
        varying lowp vec4 color;
        
        uniform mat4 Model;
        uniform mat4 Projection;
        
        void main() {
            gl_Position = Projection * Model * vec4(position, 1);
            color = color0 / 255.0;
            uv = texcoord;
        }"#;
        
        let fragment_shader = r#"#version 100
        precision mediump float;
        
        varying lowp vec4 color;
        varying lowp vec2 uv;
        
        uniform sampler2D sceneTexture;
        uniform vec2 screenSize;
        
        void main() {
            vec2 pixelSize = 1.0 / screenSize;
            vec3 color = texture2D(sceneTexture, uv).rgb;
            
            float lum = dot(color, vec3(0.299, 0.587, 0.114));
            
            float gx = 0.0;
            float gy = 0.0;
            
            gx += -1.0 * dot(texture2D(sceneTexture, uv + vec2(-pixelSize.x, -pixelSize.y)).rgb, vec3(0.299, 0.587, 0.114));
            gx += -2.0 * dot(texture2D(sceneTexture, uv + vec2(-pixelSize.x, 0.0)).rgb, vec3(0.299, 0.587, 0.114));
            gx += -1.0 * dot(texture2D(sceneTexture, uv + vec2(-pixelSize.x, pixelSize.y)).rgb, vec3(0.299, 0.587, 0.114));
            gx +=  1.0 * dot(texture2D(sceneTexture, uv + vec2(pixelSize.x, -pixelSize.y)).rgb, vec3(0.299, 0.587, 0.114));
            gx +=  2.0 * dot(texture2D(sceneTexture, uv + vec2(pixelSize.x, 0.0)).rgb, vec3(0.299, 0.587, 0.114));
            gx +=  1.0 * dot(texture2D(sceneTexture, uv + vec2(pixelSize.x, pixelSize.y)).rgb, vec3(0.299, 0.587, 0.114));
            
            gy += -1.0 * dot(texture2D(sceneTexture, uv + vec2(-pixelSize.x, -pixelSize.y)).rgb, vec3(0.299, 0.587, 0.114));
            gy += -2.0 * dot(texture2D(sceneTexture, uv + vec2(0.0, -pixelSize.y)).rgb, vec3(0.299, 0.587, 0.114));
            gy += -1.0 * dot(texture2D(sceneTexture, uv + vec2(pixelSize.x, -pixelSize.y)).rgb, vec3(0.299, 0.587, 0.114));
            gy +=  1.0 * dot(texture2D(sceneTexture, uv + vec2(-pixelSize.x, pixelSize.y)).rgb, vec3(0.299, 0.587, 0.114));
            gy +=  2.0 * dot(texture2D(sceneTexture, uv + vec2(0.0, pixelSize.y)).rgb, vec3(0.299, 0.587, 0.114));
            gy +=  1.0 * dot(texture2D(sceneTexture, uv + vec2(pixelSize.x, pixelSize.y)).rgb, vec3(0.299, 0.587, 0.114));
            
            float edge = sqrt(gx * gx + gy * gy);
            edge = smoothstep(0.15, 0.4, edge);
            
            vec3 toonColor;
            if (lum < 0.25) {
                toonColor = color * 0.4;
            } else if (lum < 0.5) {
                toonColor = color * 0.85;
            } else if (lum < 0.75) {
                toonColor = color * 1.3;
            } else {
                toonColor = color * 1.7;
            }
            
            toonColor = clamp(toonColor, 0.0, 1.0);
            
            float satBoost = 1.8;
            vec3 gray = vec3(dot(toonColor, vec3(0.299, 0.587, 0.114)));
            toonColor = mix(gray, toonColor, satBoost);
            toonColor = clamp(toonColor, 0.0, 1.0);
            
            vec3 edgeColor = vec3(0.02, 0.02, 0.05);
            vec3 finalColor = mix(toonColor, edgeColor, edge * 0.9);
            
            gl_FragColor = vec4(finalColor, 1.0);
        }"#;
        
        load_material(
            ShaderSource::Glsl { vertex: vertex_shader, fragment: fragment_shader },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("screenSize", UniformType::Float2),
                ],
                textures: vec![
                    "sceneTexture".to_string(),
                ],
                ..Default::default()
            },
        ).unwrap()
    })
}

pub fn create_quad_damage_outline_material() -> &'static Material {
    QUAD_DAMAGE_OUTLINE_MATERIAL.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;
        attribute vec4 normal;

        varying lowp vec3 vNormal;
        varying mediump vec3 vPos;

        uniform mat4 Model;
        uniform mat4 Projection;
        uniform mediump float time;
        uniform mediump float outlineWidth;

        void main() {
            vec3 n = normalize(normal.xyz);
            float pulse = 1.0 + 0.15 * sin(time * 4.0 + position.x * 0.5 + position.y * 0.3);
            vec3 expandedPos = position + n * outlineWidth * pulse;
            
            gl_Position = Projection * Model * vec4(expandedPos, 1.0);
            vNormal = n;
            vPos = position;
        }"#;

        let fragment_shader = r#"#version 100
        precision lowp float;

        varying lowp vec3 vNormal;
        varying mediump vec3 vPos;

        uniform mediump float time;

        float hash(vec2 p) {
            return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
        }

        float noise(vec2 p) {
            vec2 i = floor(p);
            vec2 f = fract(p);
            f = f * f * (3.0 - 2.0 * f);
            float a = hash(i);
            float b = hash(i + vec2(1.0, 0.0));
            float c = hash(i + vec2(0.0, 1.0));
            float d = hash(i + vec2(1.0, 1.0));
            return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
        }

        void main() {
            float fresnel = abs(dot(normalize(vNormal), vec3(0.0, 0.0, 1.0)));
            fresnel = 1.0 - fresnel;
            fresnel = pow(fresnel, 1.5);
            
            float energyFlow = noise(vec2(vPos.y * 0.05 + time * 2.0, vPos.x * 0.05));
            energyFlow = energyFlow * 0.5 + 0.5;
            
            float electricArc = noise(vec2(time * 8.0 + vPos.x * 0.2, vPos.y * 0.2));
            electricArc = smoothstep(0.7, 0.9, electricArc) * 1.2;
            
            float pulse = sin(time * 6.0) * 0.4 + 0.8;
            
            vec3 baseColor = vec3(0.4, 0.7, 1.0);
            vec3 glowColor = vec3(0.6, 0.9, 1.0);
            vec3 arcColor = vec3(0.7, 0.95, 1.0);
            
            vec3 finalColor = mix(baseColor, glowColor, energyFlow);
            finalColor = mix(finalColor, arcColor, electricArc);
            
            float brightness = (fresnel * 2.0 + electricArc * 3.0) * pulse * energyFlow;
            
            gl_FragColor = vec4(finalColor * brightness * 1.5, brightness * 0.9);
        }"#;

        load_material(
            ShaderSource::Glsl { vertex: vertex_shader, fragment: fragment_shader },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("time", UniformType::Float1),
                    UniformDesc::new("outlineWidth", UniformType::Float1),
                ],
                pipeline_params: PipelineParams {
                    color_blend: Some(miniquad::BlendState::new(
                        miniquad::Equation::Add,
                        miniquad::BlendFactor::Value(miniquad::BlendValue::SourceAlpha),
                        miniquad::BlendFactor::One,
                    )),
                    cull_face: miniquad::CullFace::Front,
                    ..Default::default()
                },
                ..Default::default()
            },
        ).unwrap()
    })
}

static LIQUID_BLOOD_MATERIAL: OnceLock<Material> = OnceLock::new();

pub fn create_liquid_blood_material() -> &'static Material {
    LIQUID_BLOOD_MATERIAL.get_or_init(|| {
        let vertex_shader = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;

        varying lowp vec2 uv;
        varying lowp vec4 color;

        uniform mat4 Model;
        uniform mat4 Projection;

        void main() {
            gl_Position = Projection * Model * vec4(position, 1);
            color = color0 / 255.0;
            uv = texcoord;
        }"#;

        let fragment_shader = r#"#version 100
        precision mediump float;

        varying lowp vec4 color;
        varying lowp vec2 uv;

        uniform sampler2D bloodBuffer;
        uniform sampler2D obstacleTex;
        uniform vec2 screenSize;
        uniform vec2 mapSize;
        uniform vec2 tileSize;
        uniform float time;
        uniform float deltaTime;
        uniform vec2 cameraPos;

        // shortcut to sample texture
        #define TEX(uv) texture2D(bloodBuffer, uv).r
        #define TEX1(uv) texture2D(obstacleTex, uv).r

        // shorcut for smoothstep uses
        #define trace(edge, thin) smoothstep(thin,.0,edge)
        #define ss(a,b,t) smoothstep(a,b,t)

        void main() {
            vec2 fragCoord = gl_FragCoord.xy;
            vec2 uv = fragCoord / screenSize;
            
            // coordinates
            vec2 worldUV = (fragCoord + cameraPos) / mapSize;
            
            // value from buffer
            float data = texture2D(bloodBuffer, uv).r;
            float gray = data;
            
            // gradient normal from gray value
            float range = 2.0;
            vec2 aspect = vec2(screenSize.x/screenSize.y, 1);
            vec3 unit = vec3(range/screenSize.y,0);
            vec3 normal = normalize(vec3(
                TEX(uv + unit.xz)-TEX(uv - unit.xz),
                TEX(uv - unit.zy)-TEX(uv + unit.zy),
                gray*gray*gray));
                
            // backlight
            vec3 color = vec3(.2)*(1.-abs(dot(normal, vec3(0,0,1))));
            
            // specular light
            vec3 dir = normalize(vec3(0,1,2));
            float specular = pow(dot(normal, dir)*.5+.5,20.);
            color += vec3(.3)*ss(.2,1.,specular);
            
            // rainbow tint for liquid effect
            vec3 tint = .5+.5*cos(vec3(1,2,3)*1.+dot(normal, dir)*4.-uv.y*3.-3.);
            color += tint * smoothstep(.15,.0,gray);

            // obstacle check - make blood stick to obstacles
            float obstacle = texture2D(obstacleTex, worldUV).r;
            if (obstacle > 0.5) {
                // On obstacle, enhance the color and make it more opaque
                color = mix(color, vec3(0.8, 0.1, 0.0), 0.3);
            }
            
            // dither for texture
            float dither = fract(sin(dot(fragCoord.xy, vec2(12.9898, 78.233))) * 43758.5453123);
            color -= dither*.05;
            
            // background blend
            vec3 background = vec3(0);
            color = mix(background, clamp(color, 0., 1.), ss(.01,.1,gray));
            
            gl_FragColor = vec4(color, gray > 0.01 ? min(gray * 2.0, 1.0) : 0.0);
        }"#;

        load_material(
            ShaderSource::Glsl { vertex: vertex_shader, fragment: fragment_shader },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("screenSize", UniformType::Float2),
                    UniformDesc::new("mapSize", UniformType::Float2),
                    UniformDesc::new("tileSize", UniformType::Float2),
                    UniformDesc::new("time", UniformType::Float1),
                    UniformDesc::new("deltaTime", UniformType::Float1),
                    UniformDesc::new("cameraPos", UniformType::Float2),
                ],
                textures: vec!["bloodBuffer".to_string(), "obstacleTex".to_string()],
                ..Default::default()
            },
        ).unwrap()
    })
}
