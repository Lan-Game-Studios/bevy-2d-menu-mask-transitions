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
@group(1) @binding(6)
var<uniform> preserve_mask_aspect: f32;

fn get_mask_uv(screen_uv: vec2<f32>) -> vec2<f32> {
    if preserve_mask_aspect < 0.5 {
        return screen_uv;
    }

    let mask_size = vec2<f32>(textureDimensions(mask_color_texture));
    let previous_size = vec2<f32>(textureDimensions(previous_texture));

    let mask_aspect = mask_size.x / mask_size.y;
    let previous_aspect = previous_size.x / previous_size.y;

    var mask_uv = screen_uv;
    if mask_aspect > previous_aspect {
        let scale_x = previous_aspect / mask_aspect;
        mask_uv.x = (screen_uv.x - 0.5) * scale_x + 0.5;
    } else {
        let scale_y = mask_aspect / previous_aspect;
        mask_uv.y = (screen_uv.y - 0.5) * scale_y + 0.5;
    }
    return mask_uv;
}

@fragment
fn fragment(mesh: UiVertexOutput) -> @location(0) vec4<f32> {
    let progress = (globals.time - startup) / duration;
    let erosion_min = progress;
    let erosion_max = erosion_min + 0.01;
    let color_previous = textureSample(previous_texture, previous_sampler, mesh.uv);
    let mask_uv = get_mask_uv(mesh.uv);
    let mask_color: vec4<f32> = textureSample(mask_color_texture, mask_color_sampler, mask_uv);
    let erosion_val = smoothstep(erosion_min, erosion_max, mask_color.r);
    return vec4<f32>(color_previous.xyz, erosion_val);
}
