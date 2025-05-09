const INF: f32 = 3.4028235e+38;
const PI: f32 = 3.1415926535;
const E: f32 = 2.71828;
const DEG_TO_RAD: f32 = PI / 180;











struct FrameUniform {
    frame_info: vec4<u32>,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct CameraUniform {
    position: vec3<f32>,
    _pad0: f32,
    lower_left_pixel: vec3<f32>,
    _pad1: f32,
    pixel_delta_x: vec3<f32>,
    _pad2: f32,
    pixel_delta_y: vec3<f32>,
    _pad3: f32
};

struct Material {
    albedo_and_mat: vec4<f32>,
    emission_and_roughness: vec4<f32>,
    ior: vec4<f32>,
}

struct GlassRefract {
    direction: vec3<f32>,
    attenuation: vec3<f32>,
}

struct HitInfo {
    hit: bool,
    front: bool,
    t: f32,
    material: Material,
    normal: vec3<f32>,
}









@group(0) @binding(0)
var traced_image: texture_storage_2d<rgba16float, write>;
@group(0) @binding(1)
var<uniform> frame: FrameUniform;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<storage, read> material_buffer: array<Material>;
@group(2) @binding(1)
var<storage, read> vertex_buffer: array<vec4<f32>>;
@group(2) @binding(2)
var<storage, read> tri_buffer: array<vec4<u32>>;

@group(3) @binding(0)
var<storage, read> read_frame_buffer: array<vec4<f32>>;
@group(3) @ binding(1)
var<storage, read_write> write_frame_buffer: array<vec4<f32>>;









@compute @workgroup_size(8, 8)
fn raytrace(
    @builtin(global_invocation_id) id: vec3<u32>
) {
    let frame_count = frame.frame_info.x;
    let pixel_center = camera.lower_left_pixel + (f32(id.x) * camera.pixel_delta_x) + (f32(id.y) * camera.pixel_delta_y);
    var pixel_color = vec3<f32>(0.0);
    let top_color = vec3<f32>(0.529, 0.808, 0.922);
    let bottom_color = vec3<f32>(0.0, 0.4, 0.8);
    var seed = id.x * 1973u ^ id.y * 9277u ^ frame_count * 26699u;
    let texture_size = textureDimensions(traced_image);
    let buffer_pixel = id.y * texture_size.x + id.x;

    for (var samples: u32 = 0; samples < 1; samples++) {
        var sample_color = vec3<f32>(0.0);
        seed = seed ^ samples * 5892379u;

        for (var jitter: u32 = 0; jitter < 4; jitter++) {
            seed = seed ^ jitter * 374761393u;
            let quad_x = f32(jitter & 1u);
            let quad_y = f32(jitter >> 1u);
            let rand_x = pcg_randf32(seed);
            let rand_y = pcg_randf32(seed ^ 0x85ebca6bu);

            var new_pixel_center = pixel_center + (rand_x - quad_x) * 0.5 * camera.pixel_delta_x;
            new_pixel_center = new_pixel_center + (rand_y - quad_y) * 0.5 * camera.pixel_delta_y;
            var ray = trace_ray(camera.position, new_pixel_center);
            var bounce_color = vec3<f32>(1.0);

            for (var bounces: u32 = 0; bounces < 10; bounces++) {
                let hit = intersect(ray);

                if hit.hit == true {
                    seed = seed ^ bounces * 374761393u;

                    if length(hit.material.emission_and_roughness.xyz) > 0.0001 {
                        sample_color += bounce_color * hit.material.emission_and_roughness.xyz;
                    }

                    var intersection = ray.origin + ray.direction * hit.t;
                    ray.origin = intersection;

                    if hit.material.albedo_and_mat.w == 0 {
                        ray.direction = normalize(diffuse_bounce(hit.material, ray, hit.normal, seed));
                        bounce_color *= hit.material.albedo_and_mat.xyz;
                    } else if hit.material.albedo_and_mat.w == 1 {
                        ray.direction = normalize(metallic_bounce(hit.material, ray, hit.normal, seed));
                        bounce_color *= hit.material.albedo_and_mat.xyz;
                    } else if hit.material.albedo_and_mat.w == 2 {
                        let refraction = transparent_material(hit.material, ray, hit.normal, seed, hit.t, hit.front);
                        ray.direction = normalize(refraction.direction);
                        bounce_color *= refraction.attenuation;
                    }
                    
                } else {
//                    bounce_color *= vec3<f32>(0.0);
                    pixel_color += bounce_color * mix(bottom_color, top_color, (1.0 + ray.direction.y) * 0.5);
                    break;
                }
            }
        }
        pixel_color += sample_color;
    }
    pixel_color = clamp(pixel_color / 4.0, vec3<f32>(0.0), vec3<f32>(1.0));
    if frame.frame_info.y > 9 {
        pixel_color += read_frame_buffer[buffer_pixel].xyz * (f32(frame.frame_info.y) - 10.0);
        pixel_color /= f32(frame.frame_info.y) - 9.0;
        write_frame_buffer[buffer_pixel] = vec4<f32>(pixel_color, 1.0);
    }
    textureStore(traced_image, vec2<i32>(id.xy), vec4<f32>(pixel_color, 1.0));
}

fn trace_ray(origin: vec3<f32>, point: vec3<f32>) -> Ray {
    let direction = normalize(point - origin);
    var ray: Ray;
    ray.origin = origin;
    ray.direction = direction;
    return ray;
}

fn intersect(ray: Ray) -> HitInfo {
    var t = -1.0;
    var new_t = INF;
    var pointer: i32 = -1;
    var n: vec3<f32>;
    var final_tri: vec4<u32>;

    for (var i: u32 = 0; i < arrayLength(&tri_buffer); i++) {
        let tri = tri_buffer[i];
        let e1 = vertex_buffer[tri.y].xyz - vertex_buffer[tri.x].xyz;
        let e2 = vertex_buffer[tri.z].xyz - vertex_buffer[tri.x].xyz;
        let p_vec = cross(ray.direction, e2);
        let d = dot(e1, p_vec);

        if d < 0.0001 && d > -0.0001 {continue;}

        let inv_d = 1.0 / d;
        let t_vec = ray.origin - vertex_buffer[tri.x].xyz;
        let u = dot(t_vec, p_vec) * inv_d;

        if u < 0 || u > 1 {continue;}

        let q_vec = cross(t_vec, e1);
        let v = dot(ray.direction, q_vec) * inv_d;

        if v < 0 || u + v > 1 {continue;}

        t = dot(e2, q_vec) * inv_d;

        if t > 0.0001 && t < new_t {
            new_t = t;
            n = cross(e2, e1);
            final_tri = tri;
        }
    }

    if new_t < INF {
        let front = dot(n, ray.direction) < 0;
        return HitInfo(
            true,
            front,
            new_t,
            material_buffer[final_tri.w],
            select(-n, n, front),
        );
    }

    return HitInfo(
        false,
        false,
        INF,
        material_buffer[0],
        vec3<f32>(0.0),
    );
}

fn pcg_randu32(hash: u32) -> u32 {
    let state = hash * 747796405u + 2891336453u;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn pcg_randf32(hash: u32) -> f32 {
    return f32(pcg_randu32(hash)) / 4294967296.0;
}

fn rand_cosine_hemi_vec(u1: f32, u2: f32) -> vec3<f32> {
    let r = sqrt(u1);
    let theta = 2 * PI * u2;

    let x = r * cos(theta);
    let y = r * sin(theta);
    let z = sqrt(max(0.0, 1 - u1));
    return vec3<f32>(x, y, z);
}

fn phong_reflect(seed: u32, exponent: f32, reflect: vec3<f32>) -> vec3<f32> {
    let phi = 2.0 * PI * pcg_randf32(seed);
    let cos_theta = pow(pcg_randf32(seed + 1), 1.0 / (exponent + 1));
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
    let phong_ray = vec3<f32>(sin_theta * cos(phi), sin_theta * sin(phi), cos_theta);
    return transform_vec_to_norm_space(phong_ray, reflect);
}

fn transform_vec_to_norm_space(vector: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let up = select(vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0), abs(normal.y) > 0.99);
    let tangent = normalize(cross(up, normal));
    let bitangent = cross(normal, tangent);

    let rot_mat = mat3x3<f32>(tangent, bitangent, normal);

    return rot_mat * vector;
}

fn diffuse_bounce(material: Material, in_ray: Ray, normal: vec3<f32>, rng_seed: u32) -> vec3<f32> {
    return transform_vec_to_norm_space(rand_cosine_hemi_vec(pcg_randf32(rng_seed), pcg_randf32(rng_seed + 1)), normal);
}

fn metallic_bounce(material: Material, in_ray: Ray, normal: vec3<f32>, rng_seed: u32) -> vec3<f32> {
    let reflection = reflect(in_ray.direction, normal);
    let roughness = material.emission_and_roughness.w;
    let exponent = pow(1.0 - roughness, 3.0) * 1000.0 + 1.0;
    return phong_reflect(rng_seed, exponent, reflection);
}

fn transparent_material(material: Material, in_ray: Ray, normal: vec3<f32>, rng_seed: u32, t: f32, front: bool) -> GlassRefract {
    var index = select(material.ior.x, 1.0 / material.ior.x, front);
    var refraction: GlassRefract;

    let cos_theta = min(dot(-in_ray.direction, normal), 1.0);
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta);

    let reflection_chance = schlicke(cos_theta, index);

    if index * sin_theta > 1.0 || reflection_chance > pcg_randf32(rng_seed) {
        refraction.direction = reflect(in_ray.direction, normal);
        refraction.attenuation = vec3<f32>(1.0);
        return refraction;
    }

    refraction.direction = refract(in_ray.direction, normal, index);

    if front {
        refraction.attenuation = vec3<f32>(1.0);
    } else {
        refraction.attenuation = transparent_absorbed(t, material.albedo_and_mat.xyz);
    }

    return refraction;
}

fn transparent_absorbed(distance: f32, absorbtion: vec3<f32>) -> vec3<f32> {
    return pow(vec3<f32>(E), -absorbtion * distance);
}

fn schlicke(cosine: f32, index: f32) -> f32 {
    var r0 = (index - 1.0) / (1 + index);
    r0 = r0 * r0;
    return r0 + (1.0 - r0) * pow((1.0 - cosine), 5);
}