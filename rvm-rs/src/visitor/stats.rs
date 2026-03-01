use crate::store::Store;
use crate::store::Stats;
use crate::store::geometry::{GeometryId, GeometryKind};
use super::StoreVisitor;

pub struct AddStats {
    pub stats: Stats,
}

impl AddStats {
    pub fn new() -> Self {
        Self { stats: Stats::default() }
    }
}

impl StoreVisitor for AddStats {
    fn begin_group(&mut self, _store: &Store, _node: crate::store::node::NodeId) {
        self.stats.group_n += 1;
    }

    fn geometry(&mut self, store: &Store, geo_id: GeometryId) {
        self.stats.geometry_n += 1;
        let geo = store.geometry(geo_id);
        match &geo.kind {
            GeometryKind::Pyramid { .. } => self.stats.pyramid_n += 1,
            GeometryKind::Box { .. } => self.stats.box_n += 1,
            GeometryKind::RectangularTorus { .. } => self.stats.rectangular_torus_n += 1,
            GeometryKind::CircularTorus { .. } => self.stats.circular_torus_n += 1,
            GeometryKind::EllipticalDish { .. } => self.stats.elliptical_dish_n += 1,
            GeometryKind::SphericalDish { .. } => self.stats.spherical_dish_n += 1,
            GeometryKind::Snout { .. } => self.stats.snout_n += 1,
            GeometryKind::Cylinder { .. } => self.stats.cylinder_n += 1,
            GeometryKind::Sphere { .. } => self.stats.sphere_n += 1,
            GeometryKind::Line { .. } => self.stats.line_n += 1,
            GeometryKind::FacetGroup { polygons } => {
                self.stats.facetgroup_n += 1;
                for poly in polygons {
                    self.stats.facetgroup_polygon_n += 1;
                    for cont in &poly.contours {
                        self.stats.facetgroup_polygon_n_contours_n += 1;
                        self.stats.facetgroup_polygon_n_vertices_n += cont.vertices_n;
                        if cont.vertices_n == 3 { self.stats.facetgroup_triangles_n += 1; }
                        else if cont.vertices_n == 4 { self.stats.facetgroup_quads_n += 1; }
                    }
                }
            }
        }
    }
}
