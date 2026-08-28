use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TriplanarMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub sand_color: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    pub sand_normal: Handle<Image>,
    #[texture(4)]
    #[sampler(5)]
    pub sand_rough: Handle<Image>,

    #[texture(6)]
    #[sampler(7)]
    pub moss_color: Handle<Image>,
    #[texture(8)]
    #[sampler(9)]
    pub moss_normal: Handle<Image>,
    #[texture(10)]
    #[sampler(11)]
    pub moss_rough: Handle<Image>,

    #[uniform(12)]
    pub blend_sharpness: f32,
    #[uniform(13)]
    pub texture_scale: f32,
}

impl Material for TriplanarMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/triplanar_pbr.wgsl".into()
    }
}
