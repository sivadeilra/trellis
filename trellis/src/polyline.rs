use crate::vec2::Vec2;

#[derive(Clone, Debug, Default)]
pub struct Ppoly_t {
    pub ps: Vec<Vec2<f64>>,
}

pub type Ppolyline_t = Ppoly_t;
