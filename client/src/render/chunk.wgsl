struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uvs: vec2<f32>,
    @location(2) light_level: f32,
};

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

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = global.view_projection * vec4<f32>(vertex.position + block_offset, 1.0);
    out.uvs = vertex.uvs;
    out.light_level = vertex.light_level;
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
