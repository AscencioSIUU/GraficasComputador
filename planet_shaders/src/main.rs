use raylib::prelude::*;
mod obj;
mod shader;

use obj::Obj;
use raylib::math::Vector3;
use shader::PlanetType;

// Nueva estructura para planeta individual
#[derive(Clone, Copy)]
struct Planet {
    position: Vector3,
    planet_type: PlanetType,
    spin: f32,
    spin_speed: f32,
}

fn rotate_y_vec(v: Vector3, angle: f32) -> Vector3 {
    let c = angle.cos();
    let s = angle.sin();
    Vector3::new(v.x * c - v.z * s, v.y, v.x * s + v.z * c)
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("Planet Renderer - GPU Accelerated")
        .msaa_4x() // Anti-aliasing
        .build();
    
    rl.toggle_fullscreen();
    let width = rl.get_screen_width();
    let height = rl.get_screen_height();
    rl.set_target_fps(60);

    println!("✓ Fullscreen mode: {}x{} (GPU rendering)", width, height);

    // Load sphere OBJ
    let mut tris: Vec<Vector3> = Vec::new();
    let paths = [
        "static_shaders/sphere.obj",
        "./static_shaders/sphere.obj",
        "assets/sphere.obj",
        "../static_shaders/sphere.obj"
    ];
    
    let mut loaded = false;
    for p in paths {
        if let Ok(m) = Obj::load(p) {
            tris = m.get_vertex_array();
            println!("✓ Loaded: {} (triangles: {})", p, tris.len() / 3);
            loaded = true;
            break;
        }
    }
    
    if !loaded {
        eprintln!("❌ ERROR: Could not load sphere OBJ");
        return;
    }

    // Subdivisión - ahora puedes usar nivel 3 sin problemas (GPU lo maneja)
    let mut subdivision_level = 3; // Ultra por defecto
    let base_tris = tris.clone();
    
    fn subdivide(tris: &[Vector3]) -> Vec<Vector3> {
        let mut result = Vec::new();
        for i in (0..tris.len()).step_by(3) {
            if i + 2 >= tris.len() { break; }
            let v0 = tris[i];
            let v1 = tris[i + 1];
            let v2 = tris[i + 2];
            
            let m01 = Vector3::new((v0.x+v1.x)*0.5, (v0.y+v1.y)*0.5, (v0.z+v1.z)*0.5);
            let m12 = Vector3::new((v1.x+v2.x)*0.5, (v1.y+v2.y)*0.5, (v1.z+v2.z)*0.5);
            let m20 = Vector3::new((v2.x+v0.x)*0.5, (v2.y+v0.y)*0.5, (v2.z+v0.z)*0.5);
            
            let normalize = |v: Vector3| {
                let len = (v.x*v.x + v.y*v.y + v.z*v.z).sqrt();
                if len > 1e-6 { Vector3::new(v.x/len, v.y/len, v.z/len) } else { v }
            };
            
            let m01 = normalize(m01);
            let m12 = normalize(m12);
            let m20 = normalize(m20);
            
            result.extend_from_slice(&[m01, m12, m20, v0, m01, m20, v1, m12, m01, v2, m20, m12]);
        }
        result
    }
    
    for _ in 0..subdivision_level {
        tris = subdivide(&tris);
    }
    println!("✓ Triangles: {} (Level {})\n", tris.len()/3, subdivision_level+1);

    // Convertir geometría a Mesh de Raylib (GPU) - ahora como función que toma el Vec
    let build_mesh = |mesh_tris: &Vec<Vector3>| {
        unsafe {
            let mut m = std::mem::zeroed::<raylib::ffi::Mesh>();
            m.vertexCount = mesh_tris.len() as i32;
            m.triangleCount = (mesh_tris.len() / 3) as i32;
            
            let vertices = vec![0f32; mesh_tris.len() * 3].into_boxed_slice();
            let normals = vec![0f32; mesh_tris.len() * 3].into_boxed_slice();
            
            m.vertices = Box::leak(vertices).as_mut_ptr();
            m.normals = Box::leak(normals).as_mut_ptr();
            
            for (i, v) in mesh_tris.iter().enumerate() {
                let idx = i * 3;
                *m.vertices.add(idx) = v.x;
                *m.vertices.add(idx + 1) = v.y;
                *m.vertices.add(idx + 2) = v.z;
                
                let len = (v.x*v.x + v.y*v.y + v.z*v.z).sqrt();
                *m.normals.add(idx) = v.x / len;
                *m.normals.add(idx + 1) = v.y / len;
                *m.normals.add(idx + 2) = v.z / len;
            }
            
            raylib::ffi::UploadMesh(&mut m, false);
            Mesh::from_raw(m)
        }
    };
    
    let mut sphere_mesh = build_mesh(&tris);

    // Camera 3D
    let mut camera = Camera3D::perspective(
        Vector3::new(12.0, 4.0, 12.0),
        Vector3::zero(),
        Vector3::up(),
        60.0,
    );

    // Planetas + Luna
    let mut planets = vec![
        Planet {
            position: Vector3::new(-3.5, 0.0, 0.0),
            planet_type: PlanetType::Lava,
            spin: 0.0,
            spin_speed: 0.08,
        },
        Planet {
            position: Vector3::new(3.5, 0.0, 0.0),
            planet_type: PlanetType::Ice,
            spin: 0.0,
            spin_speed: 0.025,
        },
        Planet {
            position: Vector3::new(0.0, 0.0, 4.0),
            planet_type: PlanetType::Rocky,
            spin: 0.0,
            spin_speed: 0.045,
        },
        Planet {
            position: Vector3::new(0.0, 0.0, -4.0),
            planet_type: PlanetType::BlackHole,
            spin: 0.0,
            spin_speed: 0.15,
        },
    ];

    // Luna que orbita el planeta Rocky
    let mut moon = Planet {
        position: Vector3::new(1.5, 0.3, 4.0), // Cerca del Rocky
        planet_type: PlanetType::Moon,
        spin: 0.0,
        spin_speed: 0.03,
    };

    // Parámetros de órbita lunar
    let moon_orbit_radius = 1.8f32;
    let moon_orbit_speed = 0.5f32;
    let rocky_planet_pos = Vector3::new(0.0, 0.0, 4.0);

    let mut time_s = 0.0f32;
    let mut auto_rotate = true;
    let mut cam_yaw = std::f32::consts::PI * 0.25;
    let mut cam_pitch = 0.3f32;
    let mut cam_distance = 12.0f32;

    println!("\n=== CONTROLS ===");
    println!("Arrow keys: Rotate camera");
    println!("+/- : Zoom");
    println!("SPACE: Toggle auto-rotate");
    println!("1-4: Change quality");
    println!("ESC: Exit");
    println!("================\n");

    // Shader procedimental AVANZADO (GPU) - con ruido procedural y FBM
    let shader_code = r#"
#version 330
in vec3 vertexPosition;
in vec3 vertexNormal;
uniform mat4 mvp;
uniform mat4 matModel;
uniform mat4 matNormal;
out vec3 fragPos;
out vec3 fragNormal;
out vec3 fragWorldPos;
out float fragRadialDist;
out float fragPolarAngle;
out vec3 fragViewDir;
out float fragVerticalDist; // NUEVO: para anillos ecuatoriales
void main() {
    fragWorldPos = vertexPosition;
    fragPos = vec3(matModel * vec4(vertexPosition, 1.0));
    fragNormal = normalize(mat3(matNormal) * vertexNormal);
    
    // Distancia radial desde el eje Y (para anillos ecuatoriales)
    fragRadialDist = length(vertexPosition.xz);
    
    // Distancia vertical (altura sobre plano ecuatorial)
    fragVerticalDist = abs(vertexPosition.y);
    
    // Ángulo azimutal (para patrones espirales)
    fragPolarAngle = atan(vertexPosition.z, vertexPosition.x);
    
    // Dirección de vista
    fragViewDir = normalize(vertexPosition);
    
    gl_Position = mvp * vec4(vertexPosition, 1.0);
}
    "#;
    
    let frag_code = r#"
#version 330
in vec3 fragPos;
in vec3 fragNormal;
in vec3 fragWorldPos;
in float fragRadialDist;
in float fragPolarAngle;
in vec3 fragViewDir;
in float fragVerticalDist;
out vec4 finalColor;
uniform float time;
uniform int planetType;

// ========== FUNCIONES DE RUIDO PROCEDURAL ==========

float hash(vec3 p) {
    p = fract(p * vec3(443.897, 441.423, 437.195));
    p += dot(p, p.yzx + 19.19);
    return fract((p.x + p.y) * p.z);
}

float noise3D(vec3 p) {
    vec3 i = floor(p);
    vec3 f = fract(p);
    f = f * f * (3.0 - 2.0 * f); // smoothstep
    
    float n = 0.0;
    for(int z = 0; z <= 1; z++)
    for(int y = 0; y <= 1; y++)
    for(int x = 0; x <= 1; x++) {
        vec3 g = vec3(float(x), float(y), float(z));
        float h = hash(i + g);
        vec3 w = mix(vec3(1.0) - g, g, f);
        n += w.x * w.y * w.z * h;
    }
    return n;
}

float fbm(vec3 p, int octaves) {
    float value = 0.0;
    float amplitude = 0.5;
    float frequency = 1.0;
    
    for(int i = 0; i < octaves; i++) {
        value += amplitude * noise3D(p * frequency);
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

float turbulence(vec3 p, int octaves) {
    float value = 0.0;
    float amplitude = 0.5;
    float frequency = 1.0;
    
    for(int i = 0; i < octaves; i++) {
        value += amplitude * abs(noise3D(p * frequency) * 2.0 - 1.0);
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

float smoothstep_custom(float edge0, float edge1, float x) {
    float t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

// ========== SHADERS DE PLANETAS ==========

vec3 rockyShader(vec3 pos, vec3 N) {
    float surfaceNoise = fbm(pos * 2.2, 4);
    float craters = turbulence(pos * 4.5, 3);
    float craterMask = smoothstep_custom(0.6, 0.75, craters);
    
    vec3 baseColor1 = vec3(0.6, 0.46, 0.38);
    vec3 baseColor2 = vec3(0.4, 0.35, 0.32);
    vec3 craterColor = vec3(0.2, 0.2, 0.22);
    
    vec3 albedo = mix(baseColor1, baseColor2, surfaceNoise);
    albedo = mix(albedo, craterColor, craterMask);
    
    vec3 L = normalize(vec3(1.0, 1.0, -0.5));
    float diff = max(dot(N, L), 0.0);
    float ao = 1.0 - fbm(pos * 2.0, 2) * 0.15;
    
    vec3 ambient = albedo * 0.15;
    vec3 diffuse = albedo * diff * 0.7;
    
    return (ambient + diffuse) * ao;
}

vec3 iceShader(vec3 pos, vec3 N, float t) {
    float crystals = fbm(pos * 5.5 + vec3(t * 0.08, 0.0, 0.0), 4);
    float cracks = noise3D(pos * 12.0);
    
    vec3 iceBase = vec3(0.7, 0.85, 0.95);
    vec3 iceBright = vec3(0.92, 0.96, 1.0);
    vec3 iceCrystal = vec3(0.98, 0.99, 1.0);
    
    float crystalMix = smoothstep_custom(0.4, 0.7, crystals);
    vec3 albedo = mix(iceBase, iceBright, crystalMix);
    
    float crystalHighlight = smoothstep_custom(0.75, 0.85, cracks);
    albedo = mix(albedo, iceCrystal, crystalHighlight * 0.5);
    
    vec3 L = normalize(vec3(1.0, 1.0, -0.5));
    vec3 V = normalize(-pos);
    vec3 H = normalize(L + V);
    
    float diff = max(dot(N, L), 0.0);
    float spec = pow(max(dot(N, H), 0.0), 150.0) * 0.8;
    float fresnel = pow(1.0 - max(dot(N, V), 0.0), 3.0) * 0.15;
    
    vec3 ambient = albedo * 0.2;
    vec3 diffuse = albedo * diff * 0.6;
    
    return ambient + diffuse + vec3(spec) + vec3(fresnel);
}

vec3 lavaShader(vec3 pos, vec3 N, float t) {
    vec3 flowPos = pos * 2.8 + vec3(t * 0.18, 0.0, t * 0.12);
    float flow = fbm(flowPos, 4);
    
    vec3 crackPos = pos * 5.5 + vec3(sin(t * 0.35) * 0.4, 0.0, cos(t * 0.35) * 0.4);
    float cracks = turbulence(crackPos, 3);
    
    float lavaMask = smoothstep_custom(0.3, 0.75, flow * 0.65 + cracks * 0.35);
    
    vec3 rockCold = vec3(0.12, 0.08, 0.06);
    vec3 lavaRed = vec3(0.95, 0.3, 0.08);
    vec3 lavaOrange = vec3(1.0, 0.5, 0.15);
    vec3 lavaYellow = vec3(1.0, 0.88, 0.35);
    
    vec3 albedo;
    if (lavaMask > 0.7) {
        float t = (lavaMask - 0.7) * 3.33;
        albedo = mix(lavaOrange, lavaYellow, smoothstep_custom(0.0, 1.0, t));
    } else if (lavaMask > 0.3) {
        float t = (lavaMask - 0.3) * 2.5;
        albedo = mix(lavaRed, lavaOrange, smoothstep_custom(0.0, 1.0, t));
    } else {
        albedo = mix(rockCold, lavaRed, lavaMask * 3.33);
    }
    
    float pulse = sin(t * 1.2) * 0.08 + 0.92;
    float emission = lavaMask * pulse * 0.7;
    
    vec3 L = normalize(vec3(1.0, 1.0, -0.5));
    float diff = max(dot(N, L), 0.0);
    
    vec3 ambient = albedo * 0.15;
    vec3 diffuse = albedo * diff * 0.5;
    vec3 emissive = vec3(emission * 1.0, emission * 0.7, emission * 0.3);
    
    return ambient + diffuse + emissive;
}

vec3 moonShader(vec3 pos, vec3 N, float t) {
    // SUPERFICIE LUNAR: gris rocosa con cráteres profundos
    
    // Textura base rocosa
    float baseNoise = fbm(pos * 3.5, 4);
    vec3 regolithLight = vec3(0.55, 0.52, 0.50); // Regolito claro
    vec3 regolithDark = vec3(0.28, 0.26, 0.24);  // Regolito oscuro
    vec3 baseColor = mix(regolithDark, regolithLight, baseNoise);
    
    // CRÁTERES PROCEDURALES (múltiples escalas)
    float largeCraters = 0.0;
    float mediumCraters = 0.0;
    float smallCraters = 0.0;
    
    // Cráteres grandes (Tycho, Copernicus style)
    for(int i = 0; i < 3; i++) {
        vec3 offset = vec3(float(i) * 2.5, float(i) * 1.8, float(i) * 3.2);
        float dist = length(pos - offset * 0.3);
        float crater = smoothstep_custom(0.25, 0.35, dist);
        crater *= 1.0 - smoothstep_custom(0.35, 0.45, dist);
        
        // Borde elevado del cráter
        float rim = smoothstep_custom(0.33, 0.36, dist) * (1.0 - smoothstep_custom(0.36, 0.40, dist));
        largeCraters += crater * 1.5 + rim * 0.8;
    }
    
    // Cráteres medianos (alta densidad)
    mediumCraters = turbulence(pos * 6.0, 4);
    float craterMask = smoothstep_custom(0.65, 0.80, mediumCraters);
    
    // Cráteres pequeños (impactos secundarios)
    smallCraters = turbulence(pos * 15.0, 3);
    float microCraters = smoothstep_custom(0.70, 0.85, smallCraters);
    
    // MARIA LUNARES (mares de lava basáltica)
    float mariaPattern = fbm(pos * 1.8 + vec3(0.5, 0.3, 0.8), 3);
    float mariaMask = smoothstep_custom(0.35, 0.55, mariaPattern);
    vec3 mariaColor = vec3(0.20, 0.19, 0.18); // Basalto oscuro
    
    // Mezclar colores base
    vec3 albedo = mix(baseColor, mariaColor, mariaMask * 0.7);
    
    // Aplicar cráteres (oscurecer interior)
    vec3 craterColor = vec3(0.15, 0.14, 0.13);
    albedo = mix(albedo, craterColor, craterMask * 0.6);
    albedo = mix(albedo, craterColor * 0.8, microCraters * 0.3);
    
    // Bordes de cráteres más claros (eyecta)
    vec3 ejectaColor = vec3(0.65, 0.62, 0.58);
    albedo += ejectaColor * largeCraters * 0.4;
    
    // ILUMINACIÓN (sin atmósfera, sombras duras)
    vec3 L = normalize(vec3(0.8, 0.6, -0.3));
    float diff = max(dot(N, L), 0.0);
    
    // Sombras duras de cráteres (sin suavizado atmosférico)
    float craterShadow = 1.0 - craterMask * 0.7;
    diff *= craterShadow;
    
    // Sin luz ambiente (espacio vacío)
    vec3 ambient = albedo * 0.05;
    vec3 diffuse = albedo * diff * 0.9;
    
    // Subtle Earthshine (luz reflejada de la Tierra - azul tenue)
    float earthshine = max(dot(N, normalize(vec3(0.0, 0.3, 1.0))), 0.0);
    vec3 earthshineColor = vec3(0.1, 0.15, 0.25) * earthshine * 0.15;
    
    return ambient + diffuse + earthshineColor;
}

vec3 blackHoleShader(vec3 pos, vec3 N, float t) {
    float distFromCenter = length(pos);
    float radialDist = fragRadialDist;
    float angle = fragPolarAngle;
    float verticalDist = fragVerticalDist;
    
    // ========== HORIZONTE DE EVENTOS ==========
    float eventHorizonRadius = 0.35;
    float horizonMask = smoothstep_custom(eventHorizonRadius - 0.12, eventHorizonRadius, distFromCenter);
    float photonSphere = smoothstep_custom(eventHorizonRadius, eventHorizonRadius + 0.05, distFromCenter);
    
    // Borde ultra brillante (fotón sphere) - AMARILLO INTENSO
    vec3 photonGlow = vec3(1.0, 0.95, 0.4) * pow(1.0 - photonSphere, 4.0) * 4.0;
    
    // ========== ANILLOS ECUATORIALES AMARILLOS (ABSORCIÓN DE LUZ) ==========
    // Estos anillos simulan la luz siendo absorbida y re-emitida
    vec3 absorptionRingColor = vec3(0.0);
    
    // Anillo de absorción interno (muy brillante, amarillo-blanco)
    float absRing1_inner = 0.36;
    float absRing1_outer = 0.40;
    float absRing1_mask = smoothstep_custom(absRing1_inner, absRing1_inner + 0.01, radialDist);
    absRing1_mask *= 1.0 - smoothstep_custom(absRing1_outer - 0.01, absRing1_outer, radialDist);
    absRing1_mask *= exp(-verticalDist * verticalDist / 0.004); // Muy fino verticalmente
    
    // Patrón ondulatorio (luz oscilando al ser absorbida)
    float wavePattern1 = sin(angle * 20.0 - t * 12.0) * 0.5 + 0.5;
    wavePattern1 = pow(wavePattern1, 3.0); // Hacer picos más pronunciados
    
    vec3 absColor1 = vec3(1.0, 1.0, 0.7) * (0.6 + wavePattern1 * 0.4) * 3.5;
    absorptionRingColor += absColor1 * absRing1_mask;
    
    // Anillo de absorción medio (amarillo brillante)
    float absRing2_inner = 0.42;
    float absRing2_outer = 0.48;
    float absRing2_mask = smoothstep_custom(absRing2_inner, absRing2_inner + 0.015, radialDist);
    absRing2_mask *= 1.0 - smoothstep_custom(absRing2_outer - 0.015, absRing2_outer, radialDist);
    absRing2_mask *= exp(-verticalDist * verticalDist / 0.006);
    
    float wavePattern2 = sin(angle * 16.0 + t * 8.0 + radialDist * 15.0) * 0.5 + 0.5;
    float turbulence2 = fbm(pos * 20.0 + vec3(t * 0.5, 0.0, 0.0), 2);
    
    vec3 absColor2 = vec3(1.0, 0.95, 0.3) * (0.5 + wavePattern2 * 0.5) * (0.8 + turbulence2 * 0.4) * 2.8;
    absorptionRingColor += absColor2 * absRing2_mask;
    
    // Anillo de absorción externo (amarillo-naranja)
    float absRing3_inner = 0.50;
    float absRing3_outer = 0.58;
    float absRing3_mask = smoothstep_custom(absRing3_inner, absRing3_inner + 0.02, radialDist);
    absRing3_mask *= 1.0 - smoothstep_custom(absRing3_outer - 0.02, absRing3_outer, radialDist);
    absRing3_mask *= exp(-verticalDist * verticalDist / 0.01);
    
    float spiralPattern = sin(angle * 12.0 - radialDist * 10.0 - t * 5.0) * 0.5 + 0.5;
    float turbulence3 = fbm(pos * 15.0 + vec3(-t * 0.3, 0.0, 0.0), 3);
    
    vec3 absColor3 = vec3(1.0, 0.85, 0.2) * (0.4 + spiralPattern * 0.6) * (0.7 + turbulence3 * 0.3) * 2.2;
    absorptionRingColor += absColor3 * absRing3_mask;
    
    // ========== ANILLOS DE ACRECIÓN 3D (ORIGINALES) ==========
    struct Ring {
        float innerRadius;
        float outerRadius;
        float thickness;
        vec3 hotColor;
        vec3 coolColor;
        float speed;
        float density;
    };
    
    Ring rings[4];
    rings[0] = Ring(0.60, 0.70, 0.08, vec3(1.0, 0.4, 0.1), vec3(1.0, 0.7, 0.2), -8.0, 1.0);
    rings[1] = Ring(0.72, 0.90, 0.12, vec3(1.0, 0.8, 0.3), vec3(1.0, 0.95, 0.5), -5.0, 0.9);
    rings[2] = Ring(0.92, 1.10, 0.10, vec3(0.9, 0.7, 0.4), vec3(0.7, 0.5, 1.0), 3.5, 0.7);
    rings[3] = Ring(1.12, 1.28, 0.08, vec3(0.5, 0.6, 1.0), vec3(0.7, 0.4, 0.9), 2.8, 0.5);
    
    vec3 ringColor = vec3(0.0);
    float totalRingMask = 0.0;
    
    for(int i = 0; i < 4; i++) {
        float ringMask = smoothstep_custom(rings[i].innerRadius, rings[i].innerRadius + 0.02, radialDist);
        ringMask *= 1.0 - smoothstep_custom(rings[i].outerRadius - 0.02, rings[i].outerRadius, radialDist);
        
        float heightMask = exp(-verticalDist * verticalDist / (rings[i].thickness * rings[i].thickness));
        ringMask *= heightMask;
        
        if(ringMask > 0.01) {
            float spiralFreq = 6.0 + float(i) * 3.0;
            float spiralPattern = sin(angle * spiralFreq + radialDist * 8.0 - t * rings[i].speed);
            spiralPattern = spiralPattern * 0.5 + 0.5;
            
            vec3 turbulencePos = pos * (4.0 + float(i) * 2.0) + vec3(t * rings[i].speed * 0.1, 0.0, 0.0);
            float turbulence = fbm(turbulencePos, 3);
            
            float density = mix(0.3, 1.0, spiralPattern) * (0.7 + turbulence * 0.3) * rings[i].density;
            
            float temp = mix(0.3, 1.0, 1.0 - (radialDist - rings[i].innerRadius) / (rings[i].outerRadius - rings[i].innerRadius));
            temp *= (0.8 + spiralPattern * 0.4);
            
            vec3 tempColor = mix(rings[i].coolColor, rings[i].hotColor, temp);
            
            float dopplerShift = sin(angle) * 0.15;
            tempColor.r *= 1.0 + dopplerShift;
            tempColor.b *= 1.0 - dopplerShift;
            
            ringColor += tempColor * density * ringMask;
            totalRingMask += ringMask * density;
        }
    }
    
    // ========== DISCO DE ACRECIÓN (RELLENO) ==========
    float diskMask = smoothstep_custom(0.58, 0.62, radialDist) * (1.0 - smoothstep_custom(1.25, 1.35, radialDist));
    float diskHeight = exp(-verticalDist * verticalDist / 0.15);
    diskMask *= diskHeight;
    
    float diskNoise = fbm(pos * 5.0 + vec3(t * 0.3, 0.0, -t * 0.2), 4);
    vec3 diskColor = mix(vec3(0.3, 0.25, 0.15), vec3(0.7, 0.5, 0.25), diskNoise);
    diskColor *= (1.0 - totalRingMask * 0.8);
    
    // ========== LENTE GRAVITACIONAL ==========
    float viewAngle = abs(dot(N, fragViewDir));
    float gravitationalLens = pow(1.0 - viewAngle, 2.5) * (1.0 - horizonMask);
    gravitationalLens *= smoothstep_custom(0.3, 1.3, radialDist);
    
    vec3 lensingGlow = vec3(1.0, 0.9, 0.5) * gravitationalLens * 0.5;
    
    // ========== RADIACIÓN DE HAWKING ==========
    float hawkingNoise = noise3D(pos * 12.0 + vec3(t * 4.0, 0.0, 0.0));
    float hawkingMask = (1.0 - horizonMask) * pow(1.0 - photonSphere, 3.0);
    vec3 hawkingRadiation = vec3(0.3, 0.5, 1.0) * hawkingNoise * hawkingMask * 0.25;
    
    // ========== JETS POLARES ==========
    float jetMask = exp(-radialDist * radialDist / 0.08) * smoothstep_custom(0.5, 1.5, verticalDist);
    float jetNoise = fbm(pos * 3.0 + vec3(0.0, t * 2.0, 0.0), 3);
    vec3 jetColor = vec3(0.4, 0.7, 1.0) * jetNoise * jetMask * 0.6;
    
    // ========== COMBINACIÓN FINAL ==========
    vec3 finalColor = vec3(0.0);
    
    // Horizonte negro absoluto
    finalColor = mix(vec3(0.0), finalColor, horizonMask);
    
    // Fotón sphere (amarillo brillante)
    finalColor += photonGlow;
    
    // ANILLOS DE ABSORCIÓN (amarillos, primer plano)
    finalColor += absorptionRingColor;
    
    // Anillos de acreción tradicionales
    finalColor += ringColor * 1.3;
    
    // Disco base
    finalColor += diskColor * diskMask * 0.5;
    
    // Lente gravitacional (amarillo-dorado)
    finalColor += lensingGlow;
    
    // Radiación de Hawking
    finalColor += hawkingRadiation;
    
    // Jets polares
    finalColor += jetColor;
    
    // Pulsación global
    float pulse = sin(t * 1.5) * 0.08 + 0.92;
    finalColor *= pulse;
    
    // Bloom effect más agresivo para anillos amarillos
    float brightness = dot(finalColor, vec3(0.299, 0.587, 0.114));
    if(brightness > 0.7) {
        finalColor += (finalColor - 0.7) * 0.8;
    }
    
    return finalColor;
}

// ========== MAIN ==========

void main() {
    vec3 N = normalize(fragNormal);
    vec3 color;
    
    if (planetType == 0) {
        color = rockyShader(fragWorldPos, N);
    } else if (planetType == 1) {
        color = iceShader(fragWorldPos, N, time);
    } else if (planetType == 2) {
        color = lavaShader(fragWorldPos, N, time);
    } else if (planetType == 3) {
        color = blackHoleShader(fragWorldPos, N, time);
    } else {
        color = moonShader(fragWorldPos, N, time);
    }
    
    finalColor = vec4(color, 1.0);
}
    "#;

    let shader = rl.load_shader_from_memory(&thread, Some(shader_code), Some(frag_code));
    let time_loc = shader.get_shader_location("time");
    let planet_type_loc = shader.get_shader_location("planetType");

    // Crear material Y asignarle el shader
    let mut default_material = rl.load_material_default(&thread);
    (*default_material.as_mut()).shader = *shader.as_ref();

    // Main loop
    while !rl.window_should_close() {
        time_s += rl.get_frame_time();

        // Input
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) { cam_yaw += 0.02; }
        if rl.is_key_down(KeyboardKey::KEY_LEFT) { cam_yaw -= 0.02; }
        if rl.is_key_down(KeyboardKey::KEY_UP) { cam_pitch = (cam_pitch + 0.02).min(1.5); }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) { cam_pitch = (cam_pitch - 0.02).max(-1.5); }
        if rl.is_key_down(KeyboardKey::KEY_EQUAL) { cam_distance = (cam_distance - 0.1).max(2.0); }
        if rl.is_key_down(KeyboardKey::KEY_MINUS) { cam_distance = (cam_distance + 0.1).min(20.0); }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) { auto_rotate = !auto_rotate; }
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) { break; }
        
        // Cambio de calidad
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) && subdivision_level != 0 {
            subdivision_level = 0;
            tris = base_tris.clone();
            for _ in 0..subdivision_level { tris = subdivide(&tris); }
            sphere_mesh = build_mesh(&tris);
            println!("Quality: Low ({} tris)", tris.len()/3);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO) && subdivision_level != 1 {
            subdivision_level = 1;
            tris = base_tris.clone();
            for _ in 0..subdivision_level { tris = subdivide(&tris); }
            sphere_mesh = build_mesh(&tris);
            println!("Quality: Medium ({} tris)", tris.len()/3);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_THREE) && subdivision_level != 2 {
            subdivision_level = 2;
            tris = base_tris.clone();
            for _ in 0..subdivision_level { tris = subdivide(&tris); }
            sphere_mesh = build_mesh(&tris);
            println!("Quality: High ({} tris)", tris.len()/3);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_FOUR) && subdivision_level != 3 {
            subdivision_level = 3;
            tris = base_tris.clone();
            for _ in 0..subdivision_level { tris = subdivide(&tris); }
            sphere_mesh = build_mesh(&tris);
            println!("Quality: Ultra ({} tris)", tris.len()/3);
        }

        if auto_rotate {
            for planet in &mut planets {
                planet.spin += planet.spin_speed;
            }
            moon.spin += moon.spin_speed;
            
            // Órbita lunar alrededor del planeta Rocky
            let orbit_angle = time_s * moon_orbit_speed;
            moon.position = Vector3::new(
                rocky_planet_pos.x + moon_orbit_radius * orbit_angle.cos(),
                rocky_planet_pos.y + 0.3 * (orbit_angle * 2.0).sin(), // Oscilación vertical
                rocky_planet_pos.z + moon_orbit_radius * orbit_angle.sin(),
            );
        }

        // Actualizar cámara
        camera.position = Vector3::new(
            cam_distance * cam_yaw.cos() * cam_pitch.cos(),
            cam_distance * cam_pitch.sin(),
            cam_distance * cam_yaw.sin() * cam_pitch.cos(),
        );

        // Render
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        
        {
            let mut d3 = d.begin_mode3D(&camera);
            
            // Renderizar planetas
            for planet in &planets {
                let planet_type_val = match planet.planet_type {
                    PlanetType::Rocky => 0,
                    PlanetType::Ice => 1,
                    PlanetType::Lava => 2,
                    PlanetType::BlackHole => 3,
                    _ => 0,
                };
                
                unsafe {
                    raylib::ffi::SetShaderValue(
                        *shader.as_ref(),
                        time_loc,
                        &time_s as *const f32 as *const std::ffi::c_void,
                        raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_FLOAT as i32,
                    );
                    raylib::ffi::SetShaderValue(
                        *shader.as_ref(),
                        planet_type_loc,
                        &planet_type_val as *const i32 as *const std::ffi::c_void,
                        raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_INT as i32,
                    );
                }
                
                d3.draw_mesh(
                    &sphere_mesh,
                    default_material.clone(),
                    Matrix::rotate_y(planet.spin)
                        * Matrix::translate(planet.position.x, planet.position.y, planet.position.z)
                        * Matrix::scale(1.0, 1.0, 1.0),
                );
            }
            
            // Renderizar Luna (más pequeña)
            unsafe {
                raylib::ffi::SetShaderValue(
                    *shader.as_ref(),
                    time_loc,
                    &time_s as *const f32 as *const std::ffi::c_void,
                    raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_FLOAT as i32,
                );
                let moon_type = 4; // Moon
                raylib::ffi::SetShaderValue(
                    *shader.as_ref(),
                    planet_type_loc,
                    &moon_type as *const i32 as *const std::ffi::c_void,
                    raylib::ffi::ShaderUniformDataType::SHADER_UNIFORM_INT as i32,
                );
            }
            
            d3.draw_mesh(
                &sphere_mesh,
                default_material.clone(),
                Matrix::rotate_y(moon.spin)
                    * Matrix::translate(moon.position.x, moon.position.y, moon.position.z)
                    * Matrix::scale(0.27, 0.27, 0.27), // 27% del tamaño de la Tierra (realista)
            );
        }

        // UI
        d.draw_text(&format!("FPS: {}", d.get_fps()), 20, 20, 20, Color::GREEN);
        d.draw_text(&format!("Triangles: {}", tris.len()/3), 20, 45, 16, Color::WHITE);
        d.draw_text("GPU Accelerated", 20, 65, 16, Color::LIME);
        d.draw_text("5 Bodies: Lava | Ice | Rocky+Moon | Black Hole", 20, 85, 16, Color::CYAN);
        
        let quality_text = match subdivision_level {
            0 => "Quality: [1] Low",
            1 => "Quality: [2] Medium",
            2 => "Quality: [3] High",
            3 => "Quality: [4] Ultra",
            _ => "Quality: Unknown",
        };
        d.draw_text(quality_text, 20, 105, 18, Color::CYAN);
    }

    println!("\n✓ Application closed successfully");
}