#import bevy_ui::ui_vertex_output::UiVertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

@group(1) @binding(0)
var mask_color_texture: texture_2d<f32>;
@group(1) @binding(1)
var mask_color_sampler: sampler;
@group(1) @binding(2)
var previous_texture: texture_2d<f32>;
@group(1) @binding(3)
var previous_sampler: sampler;
@group(1) @binding(4)
var<uniform> startup: f32;
@group(1) @binding(5)
var<uniform> duration: f32;

@fragment
fn fragment(mesh: UiVertexOutput) -> @location(0) vec4<f32> {
    let progress = (globals.time - startup) / duration;
    let erosion_min = progress;
    let erosion_max = erosion_min + 0.01;
    let color_previous = textureSample(previous_texture, previous_sampler, mesh.uv);
    let mask_color: vec4<f32> = textureSample(mask_color_texture, mask_color_sampler, mesh.uv);
    let erosion_val = smoothstep(erosion_min, erosion_max, mask_color.r);
    return vec4<f32>(color_previous.xyz, erosion_val);
}

