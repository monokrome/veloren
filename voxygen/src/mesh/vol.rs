use vek::*;

use common::vol::{ReadVol, Vox};

use crate::render::{
    mesh::{Mesh, Quad},
    Pipeline,
};

/// Given volume, position, and cardinal directions, compute each vertex's AO value.
/// `dirs` should be a slice of length 5 so that the sliding window of size 2 over the slice
/// yields each vertex' adjacent positions.
fn get_ao_quad<V: ReadVol>(vol: &V, pos: Vec3<i32>, dirs: &[Vec3<i32>]) -> Vec4<f32> {
    dirs.windows(2)
        .map(|offs| {
            let (s1, s2) = (
                vol.get(pos + offs[0])
                    .map(|v| !v.is_empty())
                    .unwrap_or(false),
                vol.get(pos + offs[1])
                    .map(|v| !v.is_empty())
                    .unwrap_or(false),
            );

            if s1 && s2 {
                0.0
            } else {
                let corner = vol
                    .get(pos + offs[0] + offs[1])
                    .map(|v| !v.is_empty())
                    .unwrap_or(false);
                // Map both 1 and 2 neighbors to 0.5 occlusion.
                if s1 || s2 || corner {
                    0.5
                } else {
                    1.0
                }
            }
        })
        .collect::<Vec4<f32>>()
}

// Utility function
fn create_quad<P: Pipeline, F: Fn(Vec3<f32>, Vec3<f32>, Rgb<f32>) -> P::Vertex>(
    origin: Vec3<f32>,
    unit_x: Vec3<f32>,
    unit_y: Vec3<f32>,
    norm: Vec3<f32>,
    col: Rgb<f32>,
    ao: Vec4<f32>,
    vcons: &F,
) -> Quad<P> {
    let ao_scale = 0.95;
    let dark = col * (1.0 - ao_scale);

    let ao_map = ao.map(|e| e.powf(1.5));

    if ao[0].min(ao[2]) < ao[1].min(ao[3]) {
        Quad::new(
            vcons(origin + unit_y, norm, Rgb::lerp(dark, col, ao_map[3])),
            vcons(origin, norm, Rgb::lerp(dark, col, ao_map[0])),
            vcons(origin + unit_x, norm, Rgb::lerp(dark, col, ao_map[1])),
            vcons(
                origin + unit_x + unit_y,
                norm,
                Rgb::lerp(dark, col, ao_map[2]),
            ),
        )
    } else {
        Quad::new(
            vcons(origin, norm, Rgb::lerp(dark, col, ao_map[0])),
            vcons(origin + unit_x, norm, Rgb::lerp(dark, col, ao_map[1])),
            vcons(
                origin + unit_x + unit_y,
                norm,
                Rgb::lerp(dark, col, ao_map[2]),
            ),
            vcons(origin + unit_y, norm, Rgb::lerp(dark, col, ao_map[3])),
        )
    }
}

pub fn push_vox_verts<
    V: ReadVol,
    P: Pipeline,
    F: Fn(Vec3<f32>, Vec3<f32>, Rgb<f32>) -> P::Vertex,
>(
    mesh: &mut Mesh<P>,
    vol: &V,
    pos: Vec3<i32>,
    offs: Vec3<f32>,
    col: Rgb<f32>,
    vcons: F,
    error_makes_face: bool,
) {
    let (x, y, z) = (Vec3::unit_x(), Vec3::unit_y(), Vec3::unit_z());

    // -x
    if vol
        .get(pos - Vec3::unit_x())
        .map(|v| v.is_empty())
        .unwrap_or(error_makes_face)
    {
        mesh.push_quad(create_quad(
            offs,
            Vec3::unit_z(),
            Vec3::unit_y(),
            -Vec3::unit_x(),
            col,
            get_ao_quad(vol, pos - Vec3::unit_x(), &[-z, -y, z, y, -z]),
            &vcons,
        ));
    }
    // +x
    if vol
        .get(pos + Vec3::unit_x())
        .map(|v| v.is_empty())
        .unwrap_or(error_makes_face)
    {
        mesh.push_quad(create_quad(
            offs + Vec3::unit_x(),
            Vec3::unit_y(),
            Vec3::unit_z(),
            Vec3::unit_x(),
            col,
            get_ao_quad(vol, pos + Vec3::unit_x(), &[-y, -z, y, z, -y]),
            &vcons,
        ));
    }
    // -y
    if vol
        .get(pos - Vec3::unit_y())
        .map(|v| v.is_empty())
        .unwrap_or(error_makes_face)
    {
        mesh.push_quad(create_quad(
            offs,
            Vec3::unit_x(),
            Vec3::unit_z(),
            -Vec3::unit_y(),
            col,
            get_ao_quad(vol, pos - Vec3::unit_y(), &[-x, -z, x, z, -x]),
            &vcons,
        ));
    }
    // +y
    if vol
        .get(pos + Vec3::unit_y())
        .map(|v| v.is_empty())
        .unwrap_or(error_makes_face)
    {
        mesh.push_quad(create_quad(
            offs + Vec3::unit_y(),
            Vec3::unit_z(),
            Vec3::unit_x(),
            Vec3::unit_y(),
            col,
            get_ao_quad(vol, pos + Vec3::unit_y(), &[-z, -x, z, x, -z]),
            &vcons,
        ));
    }
    // -z
    if vol
        .get(pos - Vec3::unit_z())
        .map(|v| v.is_empty())
        .unwrap_or(error_makes_face)
    {
        mesh.push_quad(create_quad(
            offs,
            Vec3::unit_y(),
            Vec3::unit_x(),
            -Vec3::unit_z(),
            col,
            get_ao_quad(vol, pos - Vec3::unit_z(), &[-y, -x, y, x, -y]),
            &vcons,
        ));
    }
    // +z
    if vol
        .get(pos + Vec3::unit_z())
        .map(|v| v.is_empty())
        .unwrap_or(error_makes_face)
    {
        mesh.push_quad(create_quad(
            offs + Vec3::unit_z(),
            Vec3::unit_x(),
            Vec3::unit_y(),
            Vec3::unit_z(),
            col,
            get_ao_quad(vol, pos + Vec3::unit_z(), &[-x, -y, x, y, -x]),
            &vcons,
        ));
    }
}
