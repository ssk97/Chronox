pub extern crate cgmath;
use self::cgmath::*;
use std::ops::Neg;
pub fn check_dir <T, A>(input_angle: A, target_angle: A) -> T
    where T: BaseFloat, A: Angle<Unitless = T>, {
    if input_angle < target_angle {
        if (target_angle - input_angle).normalize() < A::turn_div_2() {
            return T::one();
        } else {
            return T::zero()-T::one();
        }
    } else {
        if (input_angle - target_angle).normalize() < A::turn_div_2() {
            return T::zero()-T::one();
        } else {
            return T::one();
        }
    }
}
pub fn pt_dir<T, A>(from: Point2 <T>, to: Point2 <T>) -> A
where T: BaseFloat, A: Angle<Unitless = T>, {
    let offset: Vector2<T> = to - from;
    return A::atan2(offset.y, offset.x);
}
pub fn lendir<T, A>(len: T, dir: A) -> Vector2 <T>
where T: BaseFloat, A: Angle<Unitless = T>, {
    let (y, x) = dir.sin_cos();
    return vec2(x, y)*len;
}

pub fn bound<T>(val: T, max: T) -> T
where T: PartialOrd+Neg<Output = T>+Copy{
    if val > max{
        return max;
    }
    if val < -max{
        return -max;
    }
    return val;
}