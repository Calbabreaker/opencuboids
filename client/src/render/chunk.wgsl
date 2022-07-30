struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uvs: vec2<f32>,
    @location(1) light_level: f32,
};

struct GlobalUniform {
    view_projection: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> global: GlobalUniform;

var<push_constant> block_offset: vec3<f32>;

var<private> uvs: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 1.0),
);

var<private> light_levels: array<f32, 6> = array<f32, 6>(0.8, 0.8, 0.6, 0.6, 1.0, 0.4);

@vertex
fn vs_main(@location(0) vertex: u32) -> VertexOutput {
    var out: VertexOutput;

    // Unpack vertex data
    let x = vertex & 0x3fu;
    let y = (vertex & 0xfc0u) >> 6u;
    let z = (vertex & 0x3f000u) >> 12u;
    let position = vec3<f32>(f32(x), f32(y), f32(z));
    out.position = global.view_projection * vec4<f32>(position + block_offset, 1.0);

    let uv_index = (vertex & 0xc0000u) >> 18u;
    let dir_index = (vertex & 0x700000u) >> 20u;

    out.uvs = uvs[uv_index];
    out.light_level = light_levels[dir_index];
    return out;
}

@group(1) @binding(0)
var diffuse_texture: texture_2d<f32>;
@group(1) @binding(1)
var diffuse_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(diffuse_texture, diffuse_sampler, in.uvs) * in.light_level;
    color.a = 1.0;
    return color;
}
