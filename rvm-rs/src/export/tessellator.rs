use std::f32::consts::PI;
use glam::Vec3;
use crate::store::Store;
use crate::store::geometry::{GeometryId, GeometryKind, Triangulation};
use crate::math::get_scale;

pub fn sagitta_based_segment_count(arc: f32, radius: f32, scale: f32, tolerance: f32, min_seg: u32) -> u32 {
    let r = radius * scale;
    if r <= 0.0 || tolerance <= 0.0 {
        return min_seg;
    }
    let half_angle = tolerance / r;
    if half_angle >= 1.0 {
        return min_seg;
    }
    let full_angle = 2.0 * half_angle.acos();
    let n = (arc / full_angle).ceil() as u32;
    n.max(min_seg)
}

pub fn tessellate(store: &Store, geo_id: GeometryId, tolerance: f32) -> Option<Triangulation> {
    let geo = store.geometry(geo_id);
    let scale = get_scale(&geo.m_3x4);

    match &geo.kind {
        GeometryKind::Box { lengths } => Some(tessellate_box(lengths)),
        GeometryKind::Pyramid { bottom, top, offset, height } =>
            Some(tessellate_pyramid(bottom, top, offset, *height)),
        GeometryKind::Cylinder { radius, height } =>
            Some(tessellate_cylinder(*radius, *height, scale, tolerance, geo.sample_start_angle)),
        GeometryKind::Snout { radius_b, radius_t, height, offset, bshear, tshear } =>
            Some(tessellate_snout(*radius_b, *radius_t, *height, offset, bshear, tshear, scale, tolerance, geo.sample_start_angle)),
        GeometryKind::Sphere { diameter } =>
            Some(tessellate_sphere(*diameter * 0.5, scale, tolerance)),
        GeometryKind::EllipticalDish { base_radius, height } =>
            Some(tessellate_elliptical_dish(*base_radius, *height, scale, tolerance)),
        GeometryKind::SphericalDish { base_radius, height } =>
            Some(tessellate_spherical_dish(*base_radius, *height, scale, tolerance)),
        GeometryKind::CircularTorus { offset: ct_offset, radius, angle } =>
            Some(tessellate_circular_torus(*ct_offset, *radius, *angle, scale, tolerance, geo.sample_start_angle)),
        GeometryKind::RectangularTorus { inner_radius, outer_radius, height, angle } =>
            Some(tessellate_rectangular_torus(*inner_radius, *outer_radius, *height, *angle)),
        GeometryKind::Line { a, b } =>
            Some(tessellate_line(*a, *b)),
        GeometryKind::FacetGroup { polygons } =>
            Some(tessellate_facet_group(polygons)),
    }
}

fn tessellate_box(lengths: &[f32; 3]) -> Triangulation {
    let hx = lengths[0] * 0.5;
    let hy = lengths[1] * 0.5;
    let hz = lengths[2] * 0.5;

    let corners = [
        Vec3::new(-hx, -hy, -hz), Vec3::new( hx, -hy, -hz),
        Vec3::new( hx,  hy, -hz), Vec3::new(-hx,  hy, -hz),
        Vec3::new(-hx, -hy,  hz), Vec3::new( hx, -hy,  hz),
        Vec3::new( hx,  hy,  hz), Vec3::new(-hx,  hy,  hz),
    ];
    let faces: [(usize, usize, usize, usize, Vec3); 6] = [
        (0, 3, 2, 1, Vec3::new( 0.0,  0.0, -1.0)),
        (4, 5, 6, 7, Vec3::new( 0.0,  0.0,  1.0)),
        (0, 1, 5, 4, Vec3::new( 0.0, -1.0,  0.0)),
        (2, 3, 7, 6, Vec3::new( 0.0,  1.0,  0.0)),
        (0, 4, 7, 3, Vec3::new(-1.0,  0.0,  0.0)),
        (1, 2, 6, 5, Vec3::new( 1.0,  0.0,  0.0)),
    ];

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    for (a, b, c, d, n) in &faces {
        let base = (vertices.len() / 3) as u32;
        for &idx in &[*a, *b, *c, *d] {
            vertices.extend_from_slice(&[corners[idx].x, corners[idx].y, corners[idx].z]);
            normals.extend_from_slice(&[n.x, n.y, n.z]);
        }
        indices.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
    }

    Triangulation {
        vertices_n: (vertices.len() / 3) as u32,
        triangles_n: (indices.len() / 3) as u32,
        vertices, normals, indices,
        error: 0.0,
    }
}

fn tessellate_pyramid(bottom: &[f32; 2], top: &[f32; 2], offset: &[f32; 2], height: f32) -> Triangulation {
    let bx = bottom[0] * 0.5;
    let by = bottom[1] * 0.5;
    let tx = top[0] * 0.5;
    let ty = top[1] * 0.5;
    let ox = offset[0];
    let oy = offset[1];

    let bot = [
        Vec3::new(-bx, -by, 0.0),
        Vec3::new( bx, -by, 0.0),
        Vec3::new( bx,  by, 0.0),
        Vec3::new(-bx,  by, 0.0),
    ];
    let top_v = [
        Vec3::new(ox - tx, oy - ty, height),
        Vec3::new(ox + tx, oy - ty, height),
        Vec3::new(ox + tx, oy + ty, height),
        Vec3::new(ox - tx, oy + ty, height),
    ];

    let mut vertices = Vec::new();
    let mut normals_vec = Vec::new();
    let mut indices = Vec::new();

    // bottom
    let n_bot = Vec3::new(0.0, 0.0, -1.0);
    let base = 0u32;
    for v in &bot {
        vertices.extend_from_slice(&[v.x, v.y, v.z]);
        normals_vec.extend_from_slice(&[n_bot.x, n_bot.y, n_bot.z]);
    }
    indices.extend_from_slice(&[base, base+2, base+1, base, base+3, base+2]);

    // top
    let n_top = Vec3::new(0.0, 0.0, 1.0);
    let base = 4u32;
    for v in &top_v {
        vertices.extend_from_slice(&[v.x, v.y, v.z]);
        normals_vec.extend_from_slice(&[n_top.x, n_top.y, n_top.z]);
    }
    indices.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);

    // sides
    for i in 0..4 {
        let j = (i + 1) % 4;
        let b0 = bot[i];
        let b1 = bot[j];
        let t0 = top_v[i];
        let t1 = top_v[j];
        let e1 = b1 - b0;
        let e2 = t0 - b0;
        let n = e1.cross(e2).normalize_or_zero();

        let base = (vertices.len() / 3) as u32;
        for v in &[b0, b1, t1, t0] {
            vertices.extend_from_slice(&[v.x, v.y, v.z]);
            normals_vec.extend_from_slice(&[n.x, n.y, n.z]);
        }
        indices.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
    }

    Triangulation {
        vertices_n: (vertices.len() / 3) as u32,
        triangles_n: (indices.len() / 3) as u32,
        vertices, normals: normals_vec, indices,
        error: 0.0,
    }
}

fn tessellate_cylinder(radius: f32, height: f32, scale: f32, tolerance: f32, start_angle: f32) -> Triangulation {
    let n_seg = sagitta_based_segment_count(2.0 * PI, radius, scale, tolerance, 8);
    let mut vertices = Vec::new();
    let mut normals_vec = Vec::new();
    let mut indices = Vec::new();

    // side
    for i in 0..=n_seg {
        let t = i as f32 / n_seg as f32;
        let angle = start_angle + t * 2.0 * PI;
        let c = angle.cos();
        let s = angle.sin();

        vertices.extend_from_slice(&[radius * c, radius * s, 0.0]);
        normals_vec.extend_from_slice(&[c, s, 0.0]);
        vertices.extend_from_slice(&[radius * c, radius * s, height]);
        normals_vec.extend_from_slice(&[c, s, 0.0]);
    }

    // C++ winding: quadIndices(2*i, 2*ii, 2*ii+1, 2*i+1) => (a,b,c, a,c,d)
    for i in 0..n_seg {
        let b = i * 2;
        indices.extend_from_slice(&[b, b+2, b+3, b, b+3, b+1]);
    }

    // bottom cap
    let center_b = (vertices.len() / 3) as u32;
    vertices.extend_from_slice(&[0.0, 0.0, 0.0]);
    normals_vec.extend_from_slice(&[0.0, 0.0, -1.0]);
    for i in 0..=n_seg {
        let t = i as f32 / n_seg as f32;
        let angle = start_angle + t * 2.0 * PI;
        vertices.extend_from_slice(&[radius * angle.cos(), radius * angle.sin(), 0.0]);
        normals_vec.extend_from_slice(&[0.0, 0.0, -1.0]);
    }
    for i in 0..n_seg {
        indices.extend_from_slice(&[center_b, center_b + 1 + (i+1), center_b + 1 + i]);
    }

    // top cap
    let center_t = (vertices.len() / 3) as u32;
    vertices.extend_from_slice(&[0.0, 0.0, height]);
    normals_vec.extend_from_slice(&[0.0, 0.0, 1.0]);
    for i in 0..=n_seg {
        let t = i as f32 / n_seg as f32;
        let angle = start_angle + t * 2.0 * PI;
        vertices.extend_from_slice(&[radius * angle.cos(), radius * angle.sin(), height]);
        normals_vec.extend_from_slice(&[0.0, 0.0, 1.0]);
    }
    for i in 0..n_seg {
        indices.extend_from_slice(&[center_t, center_t + 1 + i, center_t + 1 + (i+1)]);
    }

    Triangulation {
        vertices_n: (vertices.len() / 3) as u32,
        triangles_n: (indices.len() / 3) as u32,
        vertices, normals: normals_vec, indices,
        error: 0.0,
    }
}

fn tessellate_snout(
    radius_b: f32, radius_t: f32, height: f32,
    offset: &[f32; 2], bshear: &[f32; 2], tshear: &[f32; 2],
    scale: f32, tolerance: f32, start_angle: f32,
) -> Triangulation {
    let max_r = radius_b.max(radius_t);
    let n_seg = sagitta_based_segment_count(2.0 * PI, max_r, scale, tolerance, 8);
    let mut vertices = Vec::new();
    let mut normals_vec = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=n_seg {
        let t = i as f32 / n_seg as f32;
        let angle = start_angle + t * 2.0 * PI;
        let c = angle.cos();
        let s = angle.sin();

        let zb = radius_b * (c * bshear[0] + s * bshear[1]);
        let zt = height + radius_t * (c * tshear[0] + s * tshear[1]);

        let bv = Vec3::new(radius_b * c, radius_b * s, zb);
        let tv = Vec3::new(offset[0] + radius_t * c, offset[1] + radius_t * s, zt);

        let n_b = Vec3::new(c, s, 0.0);

        vertices.extend_from_slice(&[bv.x, bv.y, bv.z]);
        normals_vec.extend_from_slice(&[n_b.x, n_b.y, n_b.z]);
        vertices.extend_from_slice(&[tv.x, tv.y, tv.z]);
        normals_vec.extend_from_slice(&[n_b.x, n_b.y, n_b.z]);
    }

    for i in 0..n_seg {
        let b = i * 2;
        indices.extend_from_slice(&[b, b+2, b+3, b, b+3, b+1]);
    }

    // bottom cap
    let center_b = (vertices.len() / 3) as u32;
    vertices.extend_from_slice(&[0.0, 0.0, 0.0]);
    normals_vec.extend_from_slice(&[0.0, 0.0, -1.0]);
    for i in 0..=n_seg {
        let t = i as f32 / n_seg as f32;
        let angle = start_angle + t * 2.0 * PI;
        let c = angle.cos();
        let s = angle.sin();
        let zb = radius_b * (c * bshear[0] + s * bshear[1]);
        vertices.extend_from_slice(&[radius_b * c, radius_b * s, zb]);
        normals_vec.extend_from_slice(&[0.0, 0.0, -1.0]);
    }
    for i in 0..n_seg {
        indices.extend_from_slice(&[center_b, center_b + 1 + (i+1), center_b + 1 + i]);
    }

    // top cap
    let center_t = (vertices.len() / 3) as u32;
    vertices.extend_from_slice(&[offset[0], offset[1], height]);
    normals_vec.extend_from_slice(&[0.0, 0.0, 1.0]);
    for i in 0..=n_seg {
        let t = i as f32 / n_seg as f32;
        let angle = start_angle + t * 2.0 * PI;
        let c = angle.cos();
        let s = angle.sin();
        let zt = height + radius_t * (c * tshear[0] + s * tshear[1]);
        vertices.extend_from_slice(&[offset[0] + radius_t * c, offset[1] + radius_t * s, zt]);
        normals_vec.extend_from_slice(&[0.0, 0.0, 1.0]);
    }
    for i in 0..n_seg {
        indices.extend_from_slice(&[center_t, center_t + 1 + i, center_t + 1 + (i+1)]);
    }

    Triangulation {
        vertices_n: (vertices.len() / 3) as u32,
        triangles_n: (indices.len() / 3) as u32,
        vertices, normals: normals_vec, indices,
        error: 0.0,
    }
}

fn tessellate_sphere(radius: f32, scale: f32, tolerance: f32) -> Triangulation {
    let n_lat = sagitta_based_segment_count(PI, radius, scale, tolerance, 4);
    let n_lon = sagitta_based_segment_count(2.0 * PI, radius, scale, tolerance, 8);
    let mut vertices = Vec::new();
    let mut normals_vec = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=n_lat {
        let phi = PI * i as f32 / n_lat as f32;
        let sp = phi.sin();
        let cp = phi.cos();
        for j in 0..=n_lon {
            let theta = 2.0 * PI * j as f32 / n_lon as f32;
            let st = theta.sin();
            let ct = theta.cos();
            let nx = sp * ct;
            let ny = sp * st;
            let nz = cp;
            vertices.extend_from_slice(&[radius * nx, radius * ny, radius * nz]);
            normals_vec.extend_from_slice(&[nx, ny, nz]);
        }
    }

    for i in 0..n_lat {
        for j in 0..n_lon {
            let a = i * (n_lon + 1) + j;
            let b = a + n_lon + 1;
            if i != 0 {
                indices.extend_from_slice(&[a, b, a + 1]);
            }
            if i != n_lat - 1 {
                indices.extend_from_slice(&[a + 1, b, b + 1]);
            }
        }
    }

    Triangulation {
        vertices_n: (vertices.len() / 3) as u32,
        triangles_n: (indices.len() / 3) as u32,
        vertices, normals: normals_vec, indices,
        error: 0.0,
    }
}

fn tessellate_elliptical_dish(base_radius: f32, height: f32, scale: f32, tolerance: f32) -> Triangulation {
    tessellate_dish_shape(base_radius, height, scale, tolerance, true)
}

fn tessellate_spherical_dish(base_radius: f32, height: f32, scale: f32, tolerance: f32) -> Triangulation {
    tessellate_dish_shape(base_radius, height, scale, tolerance, false)
}

fn tessellate_dish_shape(base_radius: f32, height: f32, scale: f32, tolerance: f32, _elliptical: bool) -> Triangulation {
    let n_lat = sagitta_based_segment_count(PI * 0.5, base_radius, scale, tolerance, 4);
    let n_lon = sagitta_based_segment_count(2.0 * PI, base_radius, scale, tolerance, 8);

    let mut vertices = Vec::new();
    let mut normals_vec = Vec::new();
    let mut indices = Vec::new();

    let arc = (height / base_radius).min(1.0).max(-1.0).asin();

    for i in 0..=n_lat {
        let t = i as f32 / n_lat as f32;
        let phi = t * arc;
        let sp = phi.sin();
        let cp = phi.cos();
        for j in 0..=n_lon {
            let theta = 2.0 * PI * j as f32 / n_lon as f32;
            let ct = theta.cos();
            let st = theta.sin();
            let x = base_radius * cp * ct;
            let y = base_radius * cp * st;
            let z = height * sp;
            let nx = cp * ct;
            let ny = cp * st;
            let nz = sp;
            vertices.extend_from_slice(&[x, y, z]);
            normals_vec.extend_from_slice(&[nx, ny, nz]);
        }
    }

    for i in 0..n_lat {
        for j in 0..n_lon {
            let a = i * (n_lon + 1) + j;
            let b = a + n_lon + 1;
            indices.extend_from_slice(&[a, b, a + 1]);
            indices.extend_from_slice(&[a + 1, b, b + 1]);
        }
    }

    Triangulation {
        vertices_n: (vertices.len() / 3) as u32,
        triangles_n: (indices.len() / 3) as u32,
        vertices, normals: normals_vec, indices,
        error: 0.0,
    }
}

fn tessellate_circular_torus(ct_offset: f32, radius: f32, angle: f32, scale: f32, tolerance: f32, start_angle: f32) -> Triangulation {
    let n_seg = sagitta_based_segment_count(angle.abs(), ct_offset, scale, tolerance, 4);
    let n_pipe = sagitta_based_segment_count(2.0 * PI, radius, scale, tolerance, 8);

    let mut vertices = Vec::new();
    let mut normals_vec = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=n_seg {
        let t = i as f32 / n_seg as f32;
        let phi = t * angle;
        let cp = phi.cos();
        let sp = phi.sin();

        for j in 0..=n_pipe {
            let s = j as f32 / n_pipe as f32;
            let theta = start_angle + s * 2.0 * PI;
            let ct = theta.cos();
            let st = theta.sin();

            let x = (ct_offset + radius * ct) * cp;
            let y = (ct_offset + radius * ct) * sp;
            let z = radius * st;

            let nx = ct * cp;
            let ny = ct * sp;
            let nz = st;

            vertices.extend_from_slice(&[x, y, z]);
            normals_vec.extend_from_slice(&[nx, ny, nz]);
        }
    }

    for i in 0..n_seg {
        for j in 0..n_pipe {
            let a = i * (n_pipe + 1) + j;
            let b = a + n_pipe + 1;
            indices.extend_from_slice(&[a, b, a + 1]);
            indices.extend_from_slice(&[a + 1, b, b + 1]);
        }
    }

    Triangulation {
        vertices_n: (vertices.len() / 3) as u32,
        triangles_n: (indices.len() / 3) as u32,
        vertices, normals: normals_vec, indices,
        error: 0.0,
    }
}

fn tessellate_rectangular_torus(inner_radius: f32, outer_radius: f32, height: f32, angle: f32) -> Triangulation {
    let n_seg = (angle.abs() / (PI / 18.0)).ceil().max(1.0) as u32;
    let hh = height * 0.5;

    let mut vertices = Vec::new();
    let mut normals_vec = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=n_seg {
        let t = i as f32 / n_seg as f32;
        let phi = t * angle;
        let cp = phi.cos();
        let sp = phi.sin();

        let profile = [
            (inner_radius, -hh), (outer_radius, -hh),
            (outer_radius,  hh), (inner_radius,  hh),
        ];

        for &(r, z) in &profile {
            vertices.extend_from_slice(&[r * cp, r * sp, z]);
            let n = Vec3::new(cp, sp, 0.0).normalize_or_zero();
            normals_vec.extend_from_slice(&[n.x, n.y, n.z]);
        }
    }

    for i in 0..n_seg {
        let base = i * 4;
        for j in 0..4 {
            let j1 = (j + 1) % 4;
            let a = base + j;
            let b = base + j1;
            let c = base + 4 + j1;
            let d = base + 4 + j;
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }

    Triangulation {
        vertices_n: (vertices.len() / 3) as u32,
        triangles_n: (indices.len() / 3) as u32,
        vertices, normals: normals_vec, indices,
        error: 0.0,
    }
}

fn tessellate_line(a: f32, b: f32) -> Triangulation {
    let vertices = vec![0.0, 0.0, 0.0, a, b, 0.0];
    let normals = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
    Triangulation {
        vertices_n: 2,
        triangles_n: 0,
        vertices,
        normals,
        indices: Vec::new(),
        error: 0.0,
    }
}

fn tessellate_facet_group(polygons: &[crate::store::geometry::Polygon]) -> Triangulation {
    let mut vertices = Vec::new();
    let mut normals_vec = Vec::new();
    let mut indices = Vec::new();

    for poly in polygons {
        for contour in &poly.contours {
            let n = contour.vertices_n;
            if n < 3 { continue; }

            let base = (vertices.len() / 3) as u32;
            vertices.extend_from_slice(&contour.vertices);
            normals_vec.extend_from_slice(&contour.normals);

            // Simple fan triangulation
            for i in 1..n - 1 {
                indices.extend_from_slice(&[base, base + i, base + i + 1]);
            }
        }
    }

    Triangulation {
        vertices_n: (vertices.len() / 3) as u32,
        triangles_n: (indices.len() / 3) as u32,
        vertices, normals: normals_vec, indices,
        error: 0.0,
    }
}
