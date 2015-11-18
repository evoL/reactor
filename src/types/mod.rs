pub use self::point::Point;
pub use self::affine_transformation::AffineTransformation;
pub use self::transform::Transform;

pub type Number = f32;

pub trait Applicable {
    fn apply(&self, point: &Point) -> Point;
}

mod point;
pub mod affine_transformation;
pub mod transform;
