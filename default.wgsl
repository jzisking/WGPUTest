@group(0) @binding(0)
var<uniform> transform: mat4x4<f32>;

@group(0) @binding(1)
var tex: texture_2d<f32>;

@group(0) @binding(2)
var sam: sampler;

struct VSInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>
}

struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>
}

@vertex
fn vs_main(input: VSInput) -> VSOutput {
    var output: VSOutput;
    output.position = transform * vec4<f32>(input.position, 1.0);
    output.uv = input.uv;

    return output;
}

@fragment
fn fs_main(input: VSOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, sam, input.uv);
}