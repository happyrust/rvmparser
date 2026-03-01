use crate::store::Store;
use crate::store::node::{NodeId, NodeKind, FileInfo, ModelInfo, GroupInfo};
use crate::store::geometry::{GeometryKind, GeometryType, Polygon, Contour};
use crate::math::bbox::BBox3f;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RvmError {
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Invalid chunk: expected {expected}, got {got}")]
    InvalidChunk { expected: String, got: String },
    #[error("Unknown primitive kind: {0}")]
    UnknownPrimitiveKind(u32),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Offset mismatch in chunk {chunk}: expected {expected:#x}, got {got:#x}")]
    OffsetMismatch { chunk: String, expected: u32, got: u32 },
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    base: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0, base: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    fn offset(&self) -> u32 {
        (self.pos - self.base) as u32
    }

    fn read_u8(&mut self) -> Result<u8, RvmError> {
        if self.pos >= self.data.len() { return Err(RvmError::UnexpectedEof); }
        let v = self.data[self.pos];
        self.pos += 1;
        Ok(v)
    }

    fn read_u32_be(&mut self) -> Result<u32, RvmError> {
        if self.pos + 4 > self.data.len() { return Err(RvmError::UnexpectedEof); }
        let v = u32::from_be_bytes([
            self.data[self.pos], self.data[self.pos+1],
            self.data[self.pos+2], self.data[self.pos+3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    fn read_f32_be(&mut self) -> Result<f32, RvmError> {
        if self.pos + 4 > self.data.len() { return Err(RvmError::UnexpectedEof); }
        let bits = u32::from_be_bytes([
            self.data[self.pos], self.data[self.pos+1],
            self.data[self.pos+2], self.data[self.pos+3],
        ]);
        self.pos += 4;
        Ok(f32::from_bits(bits))
    }

    fn read_string(&mut self, store: &mut Store) -> Result<usize, RvmError> {
        let len = self.read_u32_be()? as usize;
        let byte_len = 4 * len;
        if self.pos + byte_len > self.data.len() { return Err(RvmError::UnexpectedEof); }
        let raw = &self.data[self.pos..self.pos + byte_len];
        let actual_len = raw.iter().position(|&b| b == 0).unwrap_or(byte_len);
        let s = std::str::from_utf8(&raw[..actual_len]).unwrap_or("");
        let id = store.strings.intern(s);
        self.pos += byte_len;
        Ok(id)
    }

    fn read_chunk_header(&mut self) -> Result<([u8; 4], u32, u32), RvmError> {
        let mut id = [0u8; 4];
        for byte in id.iter_mut() {
            if self.remaining() < 4 { return Err(RvmError::UnexpectedEof); }
            *byte = self.data[self.pos + 3];
            self.pos += 4;
        }
        let next_offset = self.read_u32_be()?;
        let dunno = self.read_u32_be()?;
        Ok((id, next_offset, dunno))
    }
}

fn chunk_id_str(id: &[u8; 4]) -> String {
    String::from_utf8_lossy(id).to_string()
}

fn chunk_id_eq(id: &[u8; 4], s: &[u8; 4]) -> bool {
    id == s
}

fn verify_offset(reader: &Reader, chunk: &str, expected: u32) -> Result<(), RvmError> {
    let current = reader.offset();
    if current == expected {
        Ok(())
    } else {
        Err(RvmError::OffsetMismatch {
            chunk: chunk.to_string(),
            expected,
            got: current,
        })
    }
}

pub fn parse_rvm(data: &[u8], path: &str, store: &mut Store) -> Result<(), RvmError> {
    let mut reader = Reader::new(data);
    let mut group_stack: Vec<NodeId> = Vec::new();

    let (id, next_offset, _) = reader.read_chunk_header()?;
    if !chunk_id_eq(&id, b"HEAD") {
        return Err(RvmError::InvalidChunk {
            expected: "HEAD".into(),
            got: chunk_id_str(&id),
        });
    }
    reader.base = reader.pos;
    parse_head(&mut reader, store, path, &mut group_stack, next_offset)?;

    let (id, next_offset, _) = reader.read_chunk_header()?;
    if !chunk_id_eq(&id, b"MODL") {
        return Err(RvmError::InvalidChunk {
            expected: "MODL".into(),
            got: chunk_id_str(&id),
        });
    }
    reader.base = reader.pos;
    parse_modl(&mut reader, store, &mut group_stack, next_offset)?;

    loop {
        if reader.remaining() == 0 { break; }
        let (id, next_offset, _) = reader.read_chunk_header()?;
        reader.base = reader.pos;

        if chunk_id_eq(&id, b"END:") {
            break;
        } else if chunk_id_eq(&id, b"CNTB") {
            parse_cntb(&mut reader, store, &mut group_stack, next_offset)?;
        } else if chunk_id_eq(&id, b"PRIM") || chunk_id_eq(&id, b"OBST") || chunk_id_eq(&id, b"INSU") {
            parse_prim(&mut reader, store, &group_stack, &id, next_offset)?;
        } else if chunk_id_eq(&id, b"COLR") {
            parse_colr(&mut reader, store, &group_stack, next_offset)?;
        } else if chunk_id_eq(&id, b"CNTE") {
            let _version = reader.read_u32_be()?;
        } else {
            return Err(RvmError::ParseError(format!("Unrecognized chunk {}", chunk_id_str(&id))));
        }
    }

    store.update_counts();
    Ok(())
}

fn parse_head(
    reader: &mut Reader,
    store: &mut Store,
    path: &str,
    group_stack: &mut Vec<NodeId>,
    expected_offset: u32,
) -> Result<(), RvmError> {
    let file_id = store.new_node(None, NodeKind::File);
    group_stack.push(file_id);

    let version = reader.read_u32_be()?;
    let info = reader.read_string(store)?;
    let note = reader.read_string(store)?;
    let date = reader.read_string(store)?;
    let user = reader.read_string(store)?;
    let encoding = if version >= 2 {
        reader.read_string(store)?
    } else {
        store.strings.intern("")
    };
    let path_id = store.strings.intern(path);

    store.node_mut(file_id).file_info = Some(FileInfo {
        info, note, date, user, encoding, path: path_id,
    });

    verify_offset(reader, "HEAD", expected_offset)?;
    Ok(())
}

fn parse_modl(
    reader: &mut Reader,
    store: &mut Store,
    group_stack: &mut Vec<NodeId>,
    expected_offset: u32,
) -> Result<(), RvmError> {
    let parent = *group_stack.last().ok_or(RvmError::ParseError("Empty group stack".into()))?;
    let model_id = store.new_node(Some(parent), NodeKind::Model);
    group_stack.push(model_id);

    let _version = reader.read_u32_be()?;
    let project = reader.read_string(store)?;
    let name = reader.read_string(store)?;

    store.node_mut(model_id).model_info = Some(ModelInfo { project, name });

    verify_offset(reader, "MODL", expected_offset)?;
    Ok(())
}

fn parse_cntb(
    reader: &mut Reader,
    store: &mut Store,
    group_stack: &mut Vec<NodeId>,
    expected_offset: u32,
) -> Result<(), RvmError> {
    let parent = *group_stack.last().ok_or(RvmError::ParseError("Empty group stack".into()))?;

    let parent_transparency = if let Some(gi) = &store.node(parent).group_info {
        gi.transparency
    } else {
        0
    };

    let group_id = store.new_node(Some(parent), NodeKind::Group);
    group_stack.push(group_id);

    let version = reader.read_u32_be()?;
    let name = reader.read_string(store)?;

    let mut translation = [0.0f32; 3];
    for t in translation.iter_mut() {
        *t = reader.read_f32_be()? * 0.001;
    }

    let material = reader.read_u32_be()?;

    let mut transparency = parent_transparency;
    if version > 2 {
        transparency = reader.read_u8()? as u32;
        reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;
    }

    store.node_mut(group_id).group_info = Some(GroupInfo {
        name,
        material,
        transparency,
        translation,
        ..Default::default()
    });

    verify_offset(reader, "CNTB", expected_offset)?;

    loop {
        if reader.remaining() == 0 { break; }
        let (id, next_offset, _) = reader.read_chunk_header()?;
        reader.base = reader.pos;

        if chunk_id_eq(&id, b"CNTE") {
            let _version = reader.read_u32_be()?;
            break;
        } else if chunk_id_eq(&id, b"CNTB") {
            parse_cntb(reader, store, group_stack, next_offset)?;
        } else if chunk_id_eq(&id, b"PRIM") || chunk_id_eq(&id, b"OBST") || chunk_id_eq(&id, b"INSU") {
            parse_prim(reader, store, group_stack, &id, next_offset)?;
        } else {
            return Err(RvmError::ParseError(format!(
                "In CNTB, unknown chunk id {}", chunk_id_str(&id)
            )));
        }
    }

    group_stack.pop();
    Ok(())
}

fn parse_prim(
    reader: &mut Reader,
    store: &mut Store,
    group_stack: &[NodeId],
    chunk_id: &[u8; 4],
    expected_offset: u32,
) -> Result<(), RvmError> {
    let parent = *group_stack.last().ok_or(RvmError::ParseError("Empty group stack".into()))?;

    let _version = reader.read_u32_be()?;
    let kind_num = reader.read_u32_be()?;

    let geo_id = store.new_geometry(parent);

    let geo = store.geometry_mut(geo_id);
    for i in 0..12 {
        geo.m_3x4.data[i] = reader.read_f32_be()?;
    }

    let mut bbox_data = [0.0f32; 6];
    for d in bbox_data.iter_mut() {
        *d = reader.read_f32_be()?;
    }
    geo.bbox_local = BBox3f::from_data(&bbox_data);
    geo.bbox_world = BBox3f::transform(&geo.m_3x4, &geo.bbox_local);

    let geo_type = if chunk_id_eq(chunk_id, b"PRIM") {
        GeometryType::Primitive
    } else if chunk_id_eq(chunk_id, b"OBST") {
        GeometryType::Obstruction
    } else {
        GeometryType::Insulation
    };

    let has_transparency = matches!(geo_type, GeometryType::Obstruction | GeometryType::Insulation);
    let transparency = if has_transparency {
        let t = reader.read_u8()? as u32;
        reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;
        t
    } else {
        store.node(parent).group_info.as_ref().map_or(0, |g| g.transparency)
    };

    let kind = match kind_num {
        1 => {
            let b0 = reader.read_f32_be()?;
            let b1 = reader.read_f32_be()?;
            let t0 = reader.read_f32_be()?;
            let t1 = reader.read_f32_be()?;
            let o0 = reader.read_f32_be()?;
            let o1 = reader.read_f32_be()?;
            let h = reader.read_f32_be()?;
            GeometryKind::Pyramid {
                bottom: [b0, b1], top: [t0, t1], offset: [o0, o1], height: h,
            }
        }
        2 => {
            let l0 = reader.read_f32_be()?;
            let l1 = reader.read_f32_be()?;
            let l2 = reader.read_f32_be()?;
            GeometryKind::Box { lengths: [l0, l1, l2] }
        }
        3 => {
            let ir = reader.read_f32_be()?;
            let or = reader.read_f32_be()?;
            let h = reader.read_f32_be()?;
            let a = reader.read_f32_be()?;
            GeometryKind::RectangularTorus { inner_radius: ir, outer_radius: or, height: h, angle: a }
        }
        4 => {
            let o = reader.read_f32_be()?;
            let r = reader.read_f32_be()?;
            let a = reader.read_f32_be()?;
            GeometryKind::CircularTorus { offset: o, radius: r, angle: a }
        }
        5 => {
            let br = reader.read_f32_be()?;
            let h = reader.read_f32_be()?;
            GeometryKind::EllipticalDish { base_radius: br, height: h }
        }
        6 => {
            let br = reader.read_f32_be()?;
            let h = reader.read_f32_be()?;
            GeometryKind::SphericalDish { base_radius: br, height: h }
        }
        7 => {
            let rb = reader.read_f32_be()?;
            let rt = reader.read_f32_be()?;
            let h = reader.read_f32_be()?;
            let o0 = reader.read_f32_be()?;
            let o1 = reader.read_f32_be()?;
            let bs0 = reader.read_f32_be()?;
            let bs1 = reader.read_f32_be()?;
            let ts0 = reader.read_f32_be()?;
            let ts1 = reader.read_f32_be()?;
            GeometryKind::Snout {
                radius_b: rb, radius_t: rt, height: h,
                offset: [o0, o1], bshear: [bs0, bs1], tshear: [ts0, ts1],
            }
        }
        8 => {
            let r = reader.read_f32_be()?;
            let h = reader.read_f32_be()?;
            GeometryKind::Cylinder { radius: r, height: h }
        }
        9 => {
            let d = reader.read_f32_be()?;
            GeometryKind::Sphere { diameter: d }
        }
        10 => {
            let a = reader.read_f32_be()?;
            let b = reader.read_f32_be()?;
            GeometryKind::Line { a, b }
        }
        11 => {
            let polygons_n = reader.read_u32_be()?;
            let mut polygons = Vec::with_capacity(polygons_n as usize);
            for _ in 0..polygons_n {
                let contours_n = reader.read_u32_be()?;
                let mut contours = Vec::with_capacity(contours_n as usize);
                for _ in 0..contours_n {
                    let vertices_n = reader.read_u32_be()?;
                    let mut vertices = Vec::with_capacity((3 * vertices_n) as usize);
                    let mut normals = Vec::with_capacity((3 * vertices_n) as usize);
                    for _ in 0..vertices_n {
                        for _ in 0..3 { vertices.push(reader.read_f32_be()?); }
                        for _ in 0..3 { normals.push(reader.read_f32_be()?); }
                    }
                    contours.push(Contour { vertices, normals, vertices_n });
                }
                polygons.push(Polygon { contours });
            }
            GeometryKind::FacetGroup { polygons }
        }
        _ => return Err(RvmError::UnknownPrimitiveKind(kind_num)),
    };

    let geo = store.geometry_mut(geo_id);
    geo.kind = kind;
    geo.geo_type = geo_type;
    geo.transparency = transparency;

    verify_offset(reader, "PRIM", expected_offset)?;
    Ok(())
}

fn parse_colr(
    reader: &mut Reader,
    store: &mut Store,
    group_stack: &[NodeId],
    expected_offset: u32,
) -> Result<(), RvmError> {
    let _parent = group_stack.last().ok_or(RvmError::ParseError("Empty group stack".into()))?;

    let color_kind = reader.read_u32_be()?;
    let color_index = reader.read_u32_be()?;
    let r = reader.read_u8()?;
    let g = reader.read_u8()?;
    let b = reader.read_u8()?;
    reader.read_u8()?; // padding

    store.new_color(crate::store::Color {
        color_kind,
        color_index,
        rgb: [r, g, b],
    });

    verify_offset(reader, "COLR", expected_offset)?;
    Ok(())
}
