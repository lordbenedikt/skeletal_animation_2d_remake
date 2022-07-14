use crate::*;

pub struct State {
    pub action: transform::Action,
    pub cursor_anchor: Vec2,
    pub original_transforms: Vec<Transform>,
    pub selected_entities: Vec<Entity>,
}
impl State {
    pub fn new() -> State {
        State {
            action: transform::Action::None,
            cursor_anchor: Vec2::new(0., 0.),
            original_transforms: vec![],
            selected_entities: vec![],
        }
    }
}