@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, write>;

struct Camera {
    inverse_projection: mat4x4<f32>,
    cam_to_world: mat4x4<f32>,
}

@group(0) @binding(1)
var<uniform> camera: Camera;

fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    return state;
}

fn random_float(value: u32) -> f32 {
    return f32(hash(value)) / 4294967295.0;
}

struct State {
    color: vec3<f32>,
    is: bool,
}

fn state_at(pos: vec3<i32>) -> State {
    let p = vec3<f32>(abs(pos % 200) - 100);

    var state: State;
    state.is = dot(p, p) < 100.0;
    if state.is {
        let x = random_float(u32(pos.x * 33) + u32(pos.y * 147) + u32(pos.z * 319));
        let y = random_float(u32(pos.x * 13) + u32(pos.y * 17) + u32(pos.z * 501));
        let z = random_float(u32(pos.x * 277) + u32(pos.y * 79) + u32(pos.z * 31));
        state.color = vec3<f32>(x, y, z);
    } else {
        state.color = vec3<f32>(0.0);
    }
    
    return state;
}

fn frac0(v: f32) -> f32 {
    return (v - floor(v));
}
fn frac1(v: f32) -> f32 {
    return (1.0 - v + floor(v));
}
const EPSILON = 0.0001;

struct RayHit {
    fpos: vec3<f32>,
    norm: vec3<f32>,
    dist: f32,
    is: bool,
    color: vec3<f32>,
}

fn cast_ray(origin: vec3<f32>, dir: vec3<f32>) -> RayHit {
    var step = sign(dir);
    var delta = min(step / dir, vec3(1.0 / EPSILON));
    var tmax = vec3(0.0);
    if step.x > 0.0 {
        tmax.x = delta.x * frac1(origin.x);
    } else {
        tmax.x = delta.x * frac0(origin.x);
    }
    if step.y > 0.0 {
        tmax.y = delta.y * frac1(origin.y);
    } else {
        tmax.y = delta.y * frac0(origin.y);
    }
    if step.z > 0.0 {
        tmax.z = delta.z * frac1(origin.z);
    } else {
        tmax.z = delta.z * frac0(origin.z);
    }
    var pos = floor(origin);
    var norm = -dir;
    var dist = 0.0;

    var m = 1.0;

    loop {
        let state = state_at(vec3<i32>(pos));
        if state.is {
            var result: RayHit;
            result.fpos = origin + dir * dist;
            result.norm = norm;
            result.dist = dist;
            result.is = true;
            result.color = state.color;

            return result;
        }
        if tmax.x < tmax.y {
            if tmax.x < tmax.z {
                pos.x = pos.x + step.x;
                norm = vec3(step.x, 0.0, 0.0);
                dist = tmax.x;
                tmax.x = tmax.x + delta.x;
            } else {
                pos.z = pos.z + step.z;
                norm = vec3(0.0, 0.0, step.z);
                dist = tmax.z;
                tmax.z = tmax.z + delta.z;
            }
        } else {
            if tmax.y < tmax.z {
                pos.y = pos.y + step.y;
                norm = vec3(0.0, step.y, 0.0);
                dist = tmax.y;
                tmax.y = tmax.y + delta.y;
            } else {
                pos.z = pos.z + step.z;
                dist = tmax.z;
                norm = vec3(0.0, 0.0, step.z);
                tmax.z = tmax.z + delta.z;
            }
        }
        if dist > 250.0 * m {
            m = m * 1.1;
            step = step * 1.1;
            // delta = min(step / dir, vec3(1.0 / EPSILON));
            delta = delta * 1.1;
            if m > 64.0 {
                break;
            }
        }
    }

    let ndist = min(tmax.x, min(tmax.y, tmax.z));
    var result: RayHit;
    result.fpos = origin + dir * ndist;
    result.norm = norm;
    result.dist = dist;
    result.is = false;
    result.color = vec3<f32>(0.0);

    return result;
}

fn trace_ray(origin: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
    let result = cast_ray(origin, dir);
    let light_dir = normalize(vec3<f32>(0.1, -1.0, 0.1));

    if result.is {
        var light: f32;
        // if cast_ray(result.fpos - light_dir * 0.01, -light_dir).is {
        //     light = 0.0;
        // } else {
        //     light = (1.0 + dot(light_dir, result.norm)) / 2.0;
        // }
        light = 1.0;

        let ambient = 0.3 + 0.7 * (1.0 + dot(vec3(0.0, -1.0, 0.0), result.norm)) / 2.0;

        return result.color * (ambient * 0.3 + light * 0.7);
    } else {
        return vec3(0.0);
    }
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let sz2 = textureDimensions(texture);
    let sz = f32(min(sz2.x, sz2.y));
    let lpos = (vec2<f32>(invocation_id.xy) + 0.5) / sz * 2.0 - 1.0;
    let l_dir = camera.inverse_projection * vec4<f32>(lpos.x, -lpos.y, 0.0, 1.0);

    let dir = normalize(camera.cam_to_world * vec4<f32>(l_dir.xyz, 0.0));

    let pos = camera.cam_to_world * vec4<f32>(0.0, 0.0, 0.0, 1.0);

    let c = trace_ray(pos.xyz, dir.xyz);
    let color = vec4<f32>(c, 1.0);

    textureStore(texture, location, color);
}
