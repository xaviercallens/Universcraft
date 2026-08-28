use bevy::prelude::*;
use bevy_hanabi::prelude::*;

pub struct SsfrPlugin;

impl Plugin for SsfrPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
           .add_systems(Startup, setup_ocean_sph);
    }
}

fn setup_ocean_sph(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    println!("💧 SSFR: Initializing Hanabi GPU Compute for SPH Ocean Particles...");

    let spawner = Spawner::rate(20000.0.into()); 
    let mut module = Module::default();
    
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::new(0., 5., 0.)),
        radius: module.lit(20.0),
        dimension: ShapeDimension::Volume,
    };
    
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(1.0),
    };
    
    let init_lifetime = SetAttributeModifier::new(
        Attribute::LIFETIME,
        module.lit(5.0),
    );
    
    let gravity = AccelModifier::new(module.lit(Vec3::new(0.0, -9.81, 0.0)));
    let linear_drag = LinearDragModifier::new(module.lit(2.0)); 
    
    let color_mod = ColorOverLifetimeModifier { gradient: Gradient::constant(Vec4::new(0.0, 0.4, 0.9, 0.8)) };
    let size_mod = SizeOverLifetimeModifier { gradient: Gradient::constant(Vec2::splat(0.4)), screen_space_size: false };

    let effect = EffectAsset::new(
        vec![250_000],
        spawner,
        module,
    )
    .with_name("OceanSSFR")
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .update(gravity)
    .update(linear_drag)
    .render(color_mod)
    .render(size_mod);

    let effect_handle = effects.add(effect);

    commands.spawn(ParticleEffectBundle {
        effect: ParticleEffect::new(effect_handle),
        transform: Transform::from_translation(Vec3::new(0., 5., 0.)),
        ..default()
    });
}
