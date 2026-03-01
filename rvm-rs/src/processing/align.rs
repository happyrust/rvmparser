use std::collections::VecDeque;
use glam::Vec3;
use crate::store::Store;
use crate::store::geometry::{GeometryId, GeometryKind};
use crate::store::connection::{ConnectionId, ConnectionFlags};

struct QueueItem {
    from: Option<GeometryId>,
    connection_id: ConnectionId,
    up_world: Vec3,
}

pub fn align(store: &mut Store) -> (u32, u32) {
    let mut circular_connections = 0u32;
    let mut connected_components = 0u32;

    for conn in &mut store.connections {
        conn.temp = 0;
    }

    let conn_count = store.connections.len();
    for c in &store.connections {
        if !c.has_flag(ConnectionFlags::HasRectangularSide) && c.has_flag(ConnectionFlags::HasCircularSide) {
            circular_connections += 1;
        }
    }

    let all_conn_ids: Vec<ConnectionId> = (0..conn_count).map(ConnectionId).collect();

    for conn_id in all_conn_ids {
        if store.connection(conn_id).temp != 0 { continue; }
        if store.connection(conn_id).has_flag(ConnectionFlags::HasRectangularSide) { continue; }

        let d = store.connection(conn_id).d;
        let b = if d.x.abs() > d.y.abs() && d.x.abs() > d.z.abs() {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let up_world = d.cross(b).normalize_or_zero();

        let mut queue = VecDeque::new();
        store.connection_mut(conn_id).temp = 1;
        queue.push_back(QueueItem { from: None, connection_id: conn_id, up_world });

        while let Some(item) = queue.pop_front() {
            let conn = store.connection(item.connection_id);
            let geo_ids = conn.geo;
            let offsets = conn.offset;

            for i in 0..2 {
                if let Some(geo_id) = geo_ids[i] {
                    if item.from == Some(geo_id) { continue; }

                    let geo = store.geometry(geo_id);
                    match &geo.kind {
                        GeometryKind::CircularTorus { offset: ct_offset, radius: _, angle } => {
                            handle_circular_torus(store, geo_id, offsets[i], item.up_world, *ct_offset, *angle, &mut queue);
                        }
                        GeometryKind::Cylinder { .. } | GeometryKind::Snout { .. } |
                        GeometryKind::EllipticalDish { .. } | GeometryKind::SphericalDish { .. } => {
                            handle_cylinder_snout_dish(store, geo_id, item.up_world, &mut queue);
                        }
                        _ => {}
                    }
                }
            }
        }
        connected_components += 1;
    }

    (connected_components, circular_connections)
}

fn handle_circular_torus(
    store: &mut Store,
    geo_id: GeometryId,
    offset: usize,
    up_world: Vec3,
    _ct_offset: f32,
    angle: f32,
    queue: &mut VecDeque<QueueItem>,
) {
    let m = store.geometry(geo_id).m_3x4;
    let mat3 = m.mat3();
    let mat3_inv = mat3.inverse();

    let mut up_local = (mat3_inv * up_world).normalize_or_zero();

    if offset == 1 {
        let c = angle.cos();
        let s = angle.sin();
        up_local = Vec3::new(
            c * up_local.x + s * up_local.y,
            -s * up_local.x + c * up_local.y,
            up_local.z,
        );
    }

    let start_angle = up_local.z.atan2(up_local.x);
    let start_angle = if start_angle.is_finite() { start_angle } else { 0.0 };

    store.geometry_mut(geo_id).sample_start_angle = start_angle;

    let ci = start_angle.cos();
    let si = start_angle.sin();
    let co = angle.cos();
    let so = angle.sin();
    let up_new = Vec3::new(ci, 0.0, si);

    let up_new_world_0 = mat3 * up_new;
    let up_new_world_1 = mat3 * Vec3::new(
        co * up_new.x - so * up_new.y,
        so * up_new.x + co * up_new.y,
        up_new.z,
    );

    let connections = store.geometry(geo_id).connections;
    for k in 0..2 {
        if let Some(conn_id) = connections[k] {
            let conn = store.connection(conn_id);
            if !conn.has_flag(ConnectionFlags::HasRectangularSide) && conn.temp == 0 {
                store.connection_mut(conn_id).temp = 1;
                queue.push_back(QueueItem {
                    from: Some(geo_id),
                    connection_id: conn_id,
                    up_world: if k == 0 { up_new_world_0 } else { up_new_world_1 },
                });
            }
        }
    }
}

fn handle_cylinder_snout_dish(
    store: &mut Store,
    geo_id: GeometryId,
    up_world: Vec3,
    queue: &mut VecDeque<QueueItem>,
) {
    let m = store.geometry(geo_id).m_3x4;
    let mat3 = m.mat3();
    let mat3_inv = mat3.inverse();

    let up_n = up_world.normalize_or_zero();
    let mut up_local = mat3_inv * up_n;
    up_local.z = 0.0;

    let start_angle = up_local.y.atan2(up_local.x);
    let start_angle = if start_angle.is_finite() { start_angle } else { 0.0 };

    store.geometry_mut(geo_id).sample_start_angle = start_angle;

    let up_new_world = mat3 * Vec3::new(start_angle.cos(), start_angle.sin(), 0.0);

    let connections = store.geometry(geo_id).connections;
    for k in 0..2 {
        if let Some(conn_id) = connections[k] {
            let conn = store.connection(conn_id);
            if !conn.has_flag(ConnectionFlags::HasRectangularSide) && conn.temp == 0 {
                store.connection_mut(conn_id).temp = 1;
                queue.push_back(QueueItem {
                    from: Some(geo_id),
                    connection_id: conn_id,
                    up_world: up_new_world,
                });
            }
        }
    }
}
