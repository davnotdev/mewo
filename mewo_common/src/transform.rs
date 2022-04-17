use mewo_ecs::*;

pub type Vector2 = cgmath::Vector2<f32>;

#[derive(Clone)]
pub struct Transform2d {
    position: Vector2,
    rotation: f32,
    scale: Vector2,
}
impl Component for Transform2d {}

impl Default for Transform2d {
    fn default() -> Self {
        Transform2d {
            position: Vector2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vector2::new(1.0, 1.0),
        }
    }
}

pub struct TransformPlugin;    

impl TransformPlugin {
    pub fn name() -> &'static str {
        "mewo_common_transform"
    }

    pub fn plugin(pb: &mut PluginBuilder) {
        pb  
            .component::<Transform2d>()
        ;
    }
}

