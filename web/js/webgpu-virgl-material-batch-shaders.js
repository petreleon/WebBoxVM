const COLOR_SHADER = `
struct Output { @builtin(position) position: vec4f, @location(0) color: vec4f }
@vertex fn vertex_main(@location(0) position: vec4f, @location(1) color: vec4f) -> Output {
  var output: Output; output.position = position; output.position.z = (position.z + position.w) * .5; output.color = color; return output;
}
@fragment fn fragment_main(input: Output) -> @location(0) vec4f { return input.color; }
`;
const TEXTURE_SHADER = `
@group(0) @binding(0) var source: texture_2d<f32>; @group(0) @binding(1) var sampled: sampler;
struct Output { @builtin(position) position: vec4f, @location(0) uv: vec2f }
@vertex fn vertex_main(@location(0) position: vec4f, @location(1) uv: vec2f) -> Output {
  var output: Output; output.position = position; output.position.z = (position.z + position.w) * .5; output.uv = uv; return output;
}
@fragment fn fragment_main(input: Output) -> @location(0) vec4f { return textureSampleLevel(source, sampled, vec2f(input.uv.x, 1. - input.uv.y), 0.); }
`;
const TEXTURE_COLOR_SHADER = `
@group(0) @binding(0) var source: texture_2d<f32>; @group(0) @binding(1) var sampled: sampler;
struct Output { @builtin(position) position: vec4f, @location(0) color: vec4f, @location(1) uv: vec2f }
@vertex fn vertex_main(@location(0) position: vec4f, @location(1) color: vec4f, @location(2) uv: vec2f) -> Output {
  var output: Output; output.position = position; output.position.z = (position.z + position.w) * .5; output.color = color; output.uv = uv; return output;
}
@fragment fn fragment_main(input: Output) -> @location(0) vec4f { return textureSampleLevel(source, sampled, vec2f(input.uv.x, 1. - input.uv.y), 0.) * input.color; }
`;
const PAIR_SHADER = `
@group(0) @binding(0) var left: texture_2d<f32>; @group(0) @binding(1) var left_sampler: sampler;
@group(0) @binding(2) var right: texture_2d<f32>; @group(0) @binding(3) var right_sampler: sampler;
struct Output { @builtin(position) position: vec4f, @location(0) uv: vec2f }
@vertex fn vertex_main(@location(0) position: vec4f, @location(1) uv: vec2f) -> Output {
  var output: Output; output.position = position; output.position.z = (position.z + position.w) * .5; output.uv = uv; return output;
}
@fragment fn fragment_main(input: Output) -> @location(0) vec4f {
  let uv = vec2f(input.uv.x, 1. - input.uv.y); return textureSampleLevel(left, left_sampler, uv, 0.) * textureSampleLevel(right, right_sampler, uv, 0.);
}
`;

export const SOURCE_OVER = {
  alpha: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "one" },
  color: { dstFactor: "one-minus-src-alpha", operation: "add", srcFactor: "src-alpha" },
};

export function materialShader(material) {
  if (material === "solid" || material === "vertex-color") return COLOR_SHADER;
  if (material === "texture") return TEXTURE_SHADER;
  if (material === "texture-color") return TEXTURE_COLOR_SHADER;
  return PAIR_SHADER;
}

export function materialVertexLayout(material) {
  if (material === "solid" || material === "vertex-color") return {
    arrayStride: 32, attributes: [{ format: "float32x4", offset: 0, shaderLocation: 0 }, { format: "float32x4", offset: 16, shaderLocation: 1 }],
  };
  if (material === "texture" || material === "texture-pair") return {
    arrayStride: 24, attributes: [{ format: "float32x4", offset: 0, shaderLocation: 0 }, { format: "float32x2", offset: 16, shaderLocation: 1 }],
  };
  return {
    arrayStride: 40, attributes: [{ format: "float32x4", offset: 0, shaderLocation: 0 }, { format: "float32x4", offset: 16, shaderLocation: 1 }, { format: "float32x2", offset: 32, shaderLocation: 2 }],
  };
}

export function materialTextures(draw) {
  return draw.material === "texture-pair" ? draw.textures : draw.texture ? [draw.texture] : [];
}
