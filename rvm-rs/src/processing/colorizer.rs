use std::collections::HashMap;
use crate::store::Store;
use crate::store::node::NodeId;
use crate::store::geometry::GeometryId;

pub struct Colorizer {
    color_name_by_material_id: HashMap<u32, String>,
    color_by_name: HashMap<String, u32>,
    color_attribute: Option<String>,
    default_color: u32,
}

impl Colorizer {
    pub fn new(color_attribute: Option<&str>) -> Self {
        let mut c = Self {
            color_name_by_material_id: HashMap::new(),
            color_by_name: HashMap::new(),
            color_attribute: color_attribute.map(|s| s.to_string()),
            default_color: 0x787878,
        };
        c.init_tables();
        c
    }

    fn init_tables(&mut self) {
        let mat_colors: Vec<(u32, &str)> = vec![
            (1, "Black"), (2, "Red"), (3, "Orange"), (4, "Yellow"), (5, "Green"),
            (6, "Cyan"), (7, "Blue"), (8, "Magenta"), (9, "Brown"), (10, "White"),
            (11, "Salmon"), (12, "LightGrey"), (13, "Grey"), (14, "Plum"),
            (15, "WhiteSmoke"), (16, "Maroon"), (17, "SpringGreen"), (18, "Wheat"),
            (19, "Gold"), (20, "RoyalBlue"), (21, "LightGold"), (22, "DeepPink"),
            (23, "ForestGreen"), (24, "BrightOrange"), (25, "Ivory"), (26, "Chocolate"),
            (27, "SteelBlue"), (28, "White"), (29, "Midnight"), (30, "NavyBlue"),
            (31, "Pink"), (32, "CoralRed"),
            (206, "Black"), (207, "White"), (208, "WhiteSmoke"), (209, "Ivory"),
            (210, "Grey"), (211, "LightGrey"), (212, "DarkGrey"), (213, "DarkSlate"),
            (214, "Red"), (215, "BrightRed"), (216, "CoralRed"), (217, "Tomato"),
            (218, "Plum"), (219, "DeepPink"), (220, "Pink"), (221, "Salmon"),
            (222, "Orange"), (223, "BrightOrange"), (224, "OrangeRed"), (225, "Maroon"),
            (226, "Yellow"), (227, "Gold"), (228, "LightYellow"), (229, "LightGold"),
            (230, "YellowGreen"), (231, "SpringGreen"), (232, "Green"), (233, "ForestGreen"),
            (234, "DarkGreen"), (235, "Cyan"), (236, "Turquoise"), (237, "Aquamarine"),
            (238, "Blue"), (239, "RoyalBlue"), (240, "NavyBlue"), (241, "PowderBlue"),
            (242, "Midnight"), (243, "SteelBlue"), (244, "Indigo"), (245, "Mauve"),
            (246, "Violet"), (247, "Magenta"), (248, "Beige"), (249, "Wheat"),
            (250, "Tan"), (251, "SandyBrown"), (252, "Brown"), (253, "Khaki"),
            (254, "Chocolate"), (255, "DarkBrown"),
        ];
        for (id, name) in mat_colors {
            self.color_name_by_material_id.insert(id, name.to_string());
        }

        let named_colors: Vec<(&str, u32)> = vec![
            ("Black", 0x000000), ("White", 0xffffff), ("Red", 0xcc0000),
            ("Green", 0x00cc00), ("Blue", 0x0000cc), ("Yellow", 0xcccc00),
            ("Cyan", 0x00eded), ("Magenta", 0xdd00dd), ("Orange", 0xed9900),
            ("Brown", 0xcc2b2b), ("Pink", 0xcc919e), ("Grey", 0xa8a8a8),
            ("LightGrey", 0xbfbfbf), ("DarkGrey", 0x518c8c),
            ("Salmon", 0xf97f70), ("Plum", 0x8c668c), ("WhiteSmoke", 0xf4f4f4),
            ("Maroon", 0x8e236b), ("SpringGreen", 0x00ff7f), ("Wheat", 0xf4ddb2),
            ("Gold", 0xedc933), ("RoyalBlue", 0x4775ff), ("LightGold", 0xede8aa),
            ("DeepPink", 0xed1189), ("ForestGreen", 0x238e23),
            ("BrightOrange", 0xffa500), ("Ivory", 0xedede0), ("Chocolate", 0xed7521),
            ("SteelBlue", 0x4782b5), ("Midnight", 0x2d2d4f), ("NavyBlue", 0x00007f),
            ("CoralRed", 0xcc5b44), ("BrightRed", 0xff0000),
            ("Indigo", 0x330066), ("Mauve", 0x660099), ("Violet", 0xed82ed),
            ("Tomato", 0xff6347), ("YellowGreen", 0x99cc33), ("Aquamarine", 0x75edc6),
            ("DarkSlate", 0x2d4f4f), ("Khaki", 0x9e9e5e), ("Turquoise", 0x00bfcc),
            ("Beige", 0xf4f4db), ("OrangeRed", 0xff7f00), ("Tan", 0xdb9370),
            ("SandyBrown", 0xf4a55e), ("DarkGreen", 0x2d4f2d),
            ("PowderBlue", 0xafe0e5), ("LightYellow", 0xededd1),
            ("DarkBrown", 0x8c4414),
        ];
        for (name, color) in named_colors {
            self.color_by_name.insert(name.to_string(), color);
            self.color_by_name.insert(name.to_lowercase(), color);
        }
        self.color_by_name.insert("Default".to_string(), 0x787878);
    }

    pub fn colorize(&self, store: &mut Store) {
        let root_ids: Vec<NodeId> = store.root_ids.clone();
        for root_id in root_ids {
            let model_ids: Vec<NodeId> = store.node(root_id).children.clone();
            for model_id in model_ids {
                let group_ids: Vec<NodeId> = store.node(model_id).children.clone();
                for group_id in group_ids {
                    self.colorize_recurse(store, group_id, "Default", self.default_color, false);
                }
            }
        }
    }

    fn colorize_recurse(
        &self,
        store: &mut Store,
        node_id: NodeId,
        parent_color_name: &str,
        parent_color: u32,
        parent_override: bool,
    ) {
        let mut color_name = parent_color_name.to_string();
        let mut color = parent_color;
        let mut is_override = parent_override;

        if !is_override {
            let material = store.node(node_id).group_info.as_ref().map(|gi| gi.material).unwrap_or(0);
            if material != 0 {
                if let Some(name) = self.color_name_by_material_id.get(&material) {
                    if let Some(&c) = self.color_by_name.get(name) {
                        color_name = name.clone();
                        color = c;
                    }
                }
            }
        }

        // Check color attribute
        if let Some(ref attr_key) = self.color_attribute {
            let node = store.node(node_id);
            for attr in &node.attributes {
                let key = store.strings.get(attr.key);
                if key == attr_key {
                    let val = store.strings.get(attr.val).to_string();
                    if let Some(&c) = self.color_by_name.get(&val) {
                        color_name = val;
                        color = c;
                        is_override = true;
                    }
                    break;
                }
            }
        }

        let geo_ids: Vec<GeometryId> = store.node(node_id).geometry_ids.clone();
        for geo_id in geo_ids {
            let geo = store.geometry_mut(geo_id);
            geo.color = color;
            geo.color_name = Some(color_name.clone());
        }

        let children: Vec<NodeId> = store.node(node_id).children.clone();
        for child_id in children {
            self.colorize_recurse(store, child_id, &color_name, color, is_override);
        }
    }
}
