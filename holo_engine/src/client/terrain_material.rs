//! HoloEngine Terrain Material
//! Custom Bevy material for Triplanar PBR & Whittaker Biome Blending WGSL Shader

#[cfg(feature = "full")]
use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[cfg(feature = "full")]
#[derive(AsBindGroup, TypePath, Debug, Clone, Asset)]
pub struct TerrainMaterial {
    #[uniform(0)]
    pub view_proj: Mat4,
    #[uniform(0)]
    pub camera_pos: Vec3,
    #[uniform(0)]
    pub sun_dir: Vec3,
    #[uniform(0)]
    pub sun_color: Vec3,
    #[uniform(0)]
    pub ambient_light: Vec3,

    // If you need a noise texture
    // #[texture(1)]
    // #[sampler(2)]
    // pub noise_texture: Option<Handle<Image>>,
}

#[cfg(feature = "full")]
impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/triplanar_pbr.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/triplanar_pbr.wgsl".into()
    }
}
