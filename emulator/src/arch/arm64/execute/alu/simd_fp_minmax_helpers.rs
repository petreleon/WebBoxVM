pub(in crate::arch::arm64::execute) trait FpMinMax:
    Copy + PartialOrd
{
    fn is_nan_value(self) -> bool;
    fn is_sign_positive_value(self) -> bool;
    fn is_sign_negative_value(self) -> bool;
    fn nan() -> Self;
    fn positive_zero() -> Self;
    fn negative_zero() -> Self;
}

impl FpMinMax for f32 {
    fn is_nan_value(self) -> bool {
        self.is_nan()
    }

    fn is_sign_positive_value(self) -> bool {
        self.is_sign_positive()
    }

    fn is_sign_negative_value(self) -> bool {
        self.is_sign_negative()
    }

    fn nan() -> Self {
        f32::NAN
    }

    fn positive_zero() -> Self {
        0.0
    }

    fn negative_zero() -> Self {
        -0.0
    }
}

impl FpMinMax for f64 {
    fn is_nan_value(self) -> bool {
        self.is_nan()
    }

    fn is_sign_positive_value(self) -> bool {
        self.is_sign_positive()
    }

    fn is_sign_negative_value(self) -> bool {
        self.is_sign_negative()
    }

    fn nan() -> Self {
        f64::NAN
    }

    fn positive_zero() -> Self {
        0.0
    }

    fn negative_zero() -> Self {
        -0.0
    }
}

pub(in crate::arch::arm64::execute) fn fp_max<T: FpMinMax>(lhs: T, rhs: T) -> T {
    if lhs.is_nan_value() || rhs.is_nan_value() {
        T::nan()
    } else if both_zero(lhs, rhs) {
        if lhs.is_sign_positive_value() || rhs.is_sign_positive_value() {
            T::positive_zero()
        } else {
            T::negative_zero()
        }
    } else if lhs >= rhs {
        lhs
    } else {
        rhs
    }
}

pub(in crate::arch::arm64::execute) fn fp_min<T: FpMinMax>(lhs: T, rhs: T) -> T {
    if lhs.is_nan_value() || rhs.is_nan_value() {
        T::nan()
    } else if both_zero(lhs, rhs) {
        if lhs.is_sign_negative_value() || rhs.is_sign_negative_value() {
            T::negative_zero()
        } else {
            T::positive_zero()
        }
    } else if lhs <= rhs {
        lhs
    } else {
        rhs
    }
}

pub(in crate::arch::arm64::execute) fn fp_max_num<T: FpMinMax>(lhs: T, rhs: T) -> T {
    match (lhs.is_nan_value(), rhs.is_nan_value()) {
        (true, false) => rhs,
        (false, true) => lhs,
        _ => fp_max(lhs, rhs),
    }
}

pub(in crate::arch::arm64::execute) fn fp_min_num<T: FpMinMax>(lhs: T, rhs: T) -> T {
    match (lhs.is_nan_value(), rhs.is_nan_value()) {
        (true, false) => rhs,
        (false, true) => lhs,
        _ => fp_min(lhs, rhs),
    }
}

fn both_zero<T: FpMinMax>(lhs: T, rhs: T) -> bool {
    lhs == T::positive_zero() && rhs == T::positive_zero()
}
