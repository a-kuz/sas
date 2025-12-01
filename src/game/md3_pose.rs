use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub struct Orientation {
    pub origin: Vec3,
    pub axis: [Vec3; 3],
}

pub fn identity_axis() -> [Vec3; 3] {
    [
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
    ]
}

pub fn axis_from_mat3(m: Mat3) -> [Vec3; 3] {
    let cols = m.to_cols_array();
    [
        vec3(cols[0], cols[1], cols[2]),
        vec3(cols[3], cols[4], cols[5]),
        vec3(cols[6], cols[7], cols[8]),
    ]
}

pub fn matrix_multiply_axis(a: [Vec3; 3], b: [Vec3; 3]) -> [Vec3; 3] {
    let mut out = [Vec3::ZERO; 3];
    for i in 0..3 {
        out[i].x = a[i].x * b[0].x + a[i].y * b[1].x + a[i].z * b[2].x;
        out[i].y = a[i].x * b[0].y + a[i].y * b[1].y + a[i].z * b[2].y;
        out[i].z = a[i].x * b[0].z + a[i].y * b[1].z + a[i].z * b[2].z;
    }
    out
}

pub fn orientation_to_mat4(orientation: &Orientation) -> Mat4 {
    Mat4::from_cols(
        vec4(
            orientation.axis[0].x,
            orientation.axis[0].y,
            orientation.axis[0].z,
            0.0,
        ),
        vec4(
            orientation.axis[1].x,
            orientation.axis[1].y,
            orientation.axis[1].z,
            0.0,
        ),
        vec4(
            orientation.axis[2].x,
            orientation.axis[2].y,
            orientation.axis[2].z,
            0.0,
        ),
        vec4(
            orientation.origin.x,
            orientation.origin.y,
            orientation.origin.z,
            1.0,
        ),
    )
}

pub fn attach_rotated_entity(
    parent: &Orientation,
    local_axis: [Vec3; 3],
    tag_pos: Vec3,
    tag_axis: [[f32; 3]; 3],
) -> Orientation {
    let mut origin = parent.origin;
    origin += parent.axis[0] * tag_pos.x;
    origin += parent.axis[1] * tag_pos.y;
    origin += parent.axis[2] * tag_pos.z;

    let tag_axis_vec = [
        vec3(tag_axis[0][0], tag_axis[0][1], tag_axis[0][2]),
        vec3(tag_axis[1][0], tag_axis[1][1], tag_axis[1][2]),
        vec3(tag_axis[2][0], tag_axis[2][1], tag_axis[2][2]),
    ];

    let temp = matrix_multiply_axis(local_axis, tag_axis_vec);
    let axis = matrix_multiply_axis(temp, parent.axis);
    Orientation { origin, axis }
}

pub fn apply_local_rotation(orientation: &mut Orientation, rotation: Mat3) {
    let rot_axis = axis_from_mat3(rotation);
    orientation.axis = matrix_multiply_axis(rot_axis, orientation.axis);
}
