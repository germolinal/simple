use geometry::Vector3D;

use crate::Float;

/// Checks if two vectors are in the same hemisphere, in spherical coordinates.
/// I.e., this only works for local coordinates.
pub(crate) fn same_heisphere(w: Vector3D, wp: Vector3D) -> bool {
    w.z * wp.z > 0.0
}

pub(crate) fn cos_theta(v: Vector3D) -> Float {
    v.z
}

pub(crate) fn cos2_theta(v: Vector3D) -> Float {
    v.z * v.z
}

pub(crate) fn abs_cos_theta(v: Vector3D) -> Float {
    v.z.abs()
}

pub(crate) fn sin2_theta(v: Vector3D) -> Float {
    (1.0 - cos2_theta(v)).max(0.0) // NaN is ignored
}

pub(crate) fn sin_theta(v: Vector3D) -> Float {
    sin2_theta(v).sqrt()
}

pub(crate) fn tan_theta(v: Vector3D) -> Float {
    sin_theta(v) / cos_theta(v)
}

pub(crate) fn tan2_theta(v: Vector3D) -> Float {
    sin2_theta(v) / cos2_theta(v)
}

pub(crate) fn cos_phi(v: Vector3D) -> Float {
    let sin_theta = sin_theta(v);
    if sin_theta == 0.0 {
        1.0
    } else {
        (v.x / sin_theta).clamp(-1.0, 1.0)
    }
}

pub(crate) fn sin_phi(v: Vector3D) -> Float {
    let sin_theta = sin_theta(v);
    if sin_theta == 0.0 {
        0.0
    } else {
        (v.y / sin_theta).clamp(-1.0, 1.0)
    }
}

/// the cosine of the angles projected on the XY plane by two vectors
pub(crate) fn cos_d_phi(wa: Vector3D, wb: Vector3D) -> Float {
    let waxy = wa.x.powi(2) + wa.y.powi(2);
    let wbxy = wb.x.powi(2) + wb.y.powi(2);
    if waxy < 1e-9 || wbxy < 1e-9 {
        1.0
    } else {
        let num = wa.x * wb.x + wa.y * wb.y;
        let denom = (waxy * wbxy).sqrt();
        (num / denom).clamp(-1., 1.)
    }
}
