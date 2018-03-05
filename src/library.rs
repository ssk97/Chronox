pub extern crate cgmath;
use self::cgmath::*;
pub fn checkDir <A: Angle>(input_angle: A, target_angle: A) -> i8 {
    if input_angle < target_angle {
        if (input_angle - target_angle).normalize() < A::turn_div_2() {
            return 1;
        } else {
            return -1;
        }
    } else {
        if (input_angle - target_angle).normalize() < A::turn_div_2() {
            return -1;
        } else {
            return 1;
        }
    }
}
pub fn pt_dir<T: BaseFloat, A: Angle>(from: Point2 <T>, to: Point2 <T>) -> A{
    let offset: Vector2<T> = from - to;
    return offset.angle(Vector2::unit_x());
}
pub fn lendir<T: BaseFloat, A: Angle>(len: f32, dir: A) -> Vector2 <T>{
    return dir.sin_cos()*len;
}