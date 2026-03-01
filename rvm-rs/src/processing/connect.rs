use glam::Vec3;
use crate::store::Store;
use crate::store::node::NodeId;
use crate::store::geometry::{GeometryId, GeometryKind};
use crate::store::connection::ConnectionFlags;

struct Anchor {
    geo_id: GeometryId,
    p: Vec3,
    d: Vec3,
    offset: usize,
    flags: ConnectionFlags,
    matched: bool,
}

pub fn connect(store: &mut Store) -> (u32, u32) {
    let mut anchors: Vec<Anchor> = Vec::new();
    let mut matched = 0u32;

    let root_ids: Vec<NodeId> = store.root_ids.clone();
    for root_id in root_ids {
        let model_ids: Vec<NodeId> = store.node(root_id).children.clone();
        for model_id in model_ids {
            let group_ids: Vec<NodeId> = store.node(model_id).children.clone();
            for group_id in group_ids {
                collect_anchors_recurse(store, group_id, &mut anchors);
                let offset = 0;
                match_anchors(store, &mut anchors, offset, &mut matched);
            }
        }
    }

    let total = anchors.len() as u32 + matched;
    (matched, total)
}

fn collect_anchors_recurse(store: &Store, node_id: NodeId, anchors: &mut Vec<Anchor>) {
    let node = store.node(node_id);
    let children: Vec<NodeId> = node.children.clone();
    for child_id in children {
        collect_anchors_recurse(store, child_id, anchors);
    }

    let geo_ids: Vec<GeometryId> = node.geometry_ids.clone();
    for geo_id in geo_ids {
        extract_anchors(store, geo_id, anchors);
    }
}

fn extract_anchors(store: &Store, geo_id: GeometryId, anchors: &mut Vec<Anchor>) {
    let geo = store.geometry(geo_id);
    let m = &geo.m_3x4;

    match &geo.kind {
        GeometryKind::Cylinder { radius: _, height } => {
            let hh = *height * 0.5;
            add_anchor(anchors, geo_id, m, Vec3::new(0.0, 0.0, -hh), Vec3::new(0.0, 0.0, -1.0), 0, ConnectionFlags::HasCircularSide);
            add_anchor(anchors, geo_id, m, Vec3::new(0.0, 0.0, hh), Vec3::new(0.0, 0.0, 1.0), 1, ConnectionFlags::HasCircularSide);
        }
        GeometryKind::Snout { radius_b: _, radius_t: _, height, offset, bshear, tshear } => {
            let hh = *height * 0.5;
            let n0 = Vec3::new(
                bshear[0].sin() * bshear[1].cos(),
                bshear[1].sin(),
                -(bshear[0].cos() * bshear[1].cos()),
            );
            let n1 = Vec3::new(
                -(tshear[0].sin() * tshear[1].cos()),
                -(tshear[1].sin()),
                tshear[0].cos() * tshear[1].cos(),
            );
            add_anchor(anchors, geo_id, m, Vec3::new(-0.5 * offset[0], -0.5 * offset[1], -hh), n0, 0, ConnectionFlags::HasCircularSide);
            add_anchor(anchors, geo_id, m, Vec3::new(0.5 * offset[0], 0.5 * offset[1], hh), n1, 1, ConnectionFlags::HasCircularSide);
        }
        GeometryKind::CircularTorus { offset: ct_offset, radius: _, angle } => {
            let c = angle.cos();
            let s = angle.sin();
            add_anchor(anchors, geo_id, m, Vec3::new(*ct_offset, 0.0, 0.0), Vec3::new(0.0, -1.0, 0.0), 0, ConnectionFlags::HasCircularSide);
            add_anchor(anchors, geo_id, m, Vec3::new(ct_offset * c, ct_offset * s, 0.0), Vec3::new(-s, c, 0.0), 1, ConnectionFlags::HasCircularSide);
        }
        GeometryKind::EllipticalDish { .. } | GeometryKind::SphericalDish { .. } => {
            add_anchor(anchors, geo_id, m, Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0), 0, ConnectionFlags::HasCircularSide);
        }
        GeometryKind::Box { lengths } => {
            let hx = lengths[0] * 0.5;
            let hy = lengths[1] * 0.5;
            let hz = lengths[2] * 0.5;
            let ns = [
                Vec3::new(-1.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, -1.0, 0.0), Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, 1.0),
            ];
            let ps = [
                Vec3::new(-hx, 0.0, 0.0), Vec3::new(hx, 0.0, 0.0),
                Vec3::new(0.0, -hy, 0.0), Vec3::new(0.0, hy, 0.0),
                Vec3::new(0.0, 0.0, -hz), Vec3::new(0.0, 0.0, hz),
            ];
            for i in 0..6 {
                add_anchor(anchors, geo_id, m, ps[i], ns[i], i, ConnectionFlags::HasRectangularSide);
            }
        }
        GeometryKind::RectangularTorus { inner_radius, outer_radius, height: _, angle } => {
            let c = angle.cos();
            let s = angle.sin();
            let mid = 0.5 * (inner_radius + outer_radius);
            add_anchor(anchors, geo_id, m, Vec3::new(mid, 0.0, 0.0), Vec3::new(0.0, -1.0, 0.0), 0, ConnectionFlags::HasRectangularSide);
            add_anchor(anchors, geo_id, m, Vec3::new(mid * c, mid * s, 0.0), Vec3::new(-s, c, 0.0), 1, ConnectionFlags::HasRectangularSide);
        }
        GeometryKind::Pyramid { .. } => {
            // Simplified: just add bottom/top anchors
            if let GeometryKind::Pyramid { bottom: _, top: _, offset: poff, height } = &geo.kind {
                let hh = height * 0.5;
                add_anchor(anchors, geo_id, m, Vec3::new(0.0, 0.0, -hh), Vec3::new(0.0, 0.0, -1.0), 4, ConnectionFlags::HasRectangularSide);
                add_anchor(anchors, geo_id, m, Vec3::new(poff[0], poff[1], hh), Vec3::new(0.0, 0.0, 1.0), 5, ConnectionFlags::HasRectangularSide);
            }
        }
        _ => {}
    }
}

fn add_anchor(
    anchors: &mut Vec<Anchor>,
    geo_id: GeometryId,
    m: &crate::math::Mat3x4f,
    local_p: Vec3,
    local_d: Vec3,
    offset: usize,
    flags: ConnectionFlags,
) {
    let world_p = m.transform_point(local_p);
    let world_d = m.mat3().mul_vec3(local_d).normalize_or_zero();

    anchors.push(Anchor {
        geo_id,
        p: world_p,
        d: world_d,
        offset,
        flags,
        matched: false,
    });
}

fn match_anchors(store: &mut Store, anchors: &mut Vec<Anchor>, _offset: usize, matched_count: &mut u32) {
    let epsilon = 0.001f32;
    let ee = epsilon * epsilon;

    anchors.sort_by(|a, b| a.p.x.partial_cmp(&b.p.x).unwrap_or(std::cmp::Ordering::Equal));
    let n = anchors.len();

    for j in 0..n {
        if anchors[j].matched { continue; }
        for i in (j + 1)..n {
            if anchors[i].p.x > anchors[j].p.x + epsilon { break; }
            if anchors[i].matched { continue; }

            let dist_sq = anchors[j].p.distance_squared(anchors[i].p);
            let aligned = anchors[j].d.dot(anchors[i].d) < -0.98;

            if dist_sq <= ee && aligned {
                let conn_id = store.new_connection();
                let conn = store.connection_mut(conn_id);
                conn.geo[0] = Some(anchors[j].geo_id);
                conn.geo[1] = Some(anchors[i].geo_id);
                conn.offset[0] = anchors[j].offset;
                conn.offset[1] = anchors[i].offset;
                conn.p = anchors[j].p;
                conn.d = anchors[j].d;
                conn.set_flag(anchors[j].flags);
                conn.set_flag(anchors[i].flags);

                let geo_j = anchors[j].geo_id;
                let off_j = anchors[j].offset;
                let geo_i = anchors[i].geo_id;
                let off_i = anchors[i].offset;

                store.geometry_mut(geo_j).connections[off_j] = Some(conn_id);
                store.geometry_mut(geo_i).connections[off_i] = Some(conn_id);

                anchors[j].matched = true;
                anchors[i].matched = true;
                *matched_count += 2;
                break;
            }
        }
    }

    anchors.retain(|a| !a.matched);
}
