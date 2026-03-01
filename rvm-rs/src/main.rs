use std::process::ExitCode;
use std::time::Instant;

use rvmparser::store::Store;
use rvmparser::parser::rvm::parse_rvm;
use rvmparser::parser::att::parse_att;
use rvmparser::processing::connect::connect;
use rvmparser::processing::align::align;
use rvmparser::processing::add_group_bbox::add_group_bboxes;
use rvmparser::processing::colorizer::Colorizer;
use rvmparser::export::obj::{export_obj, ObjExportOptions};
use rvmparser::export::json::export_json;
use rvmparser::export::gltf::{export_gltf, GltfExportOptions};
use rvmparser::export::rev::export_rev;
use rvmparser::hierarchy::flatten::Flatten;
use rvmparser::hierarchy::flatten_regex::flatten_regex;
use rvmparser::hierarchy::discard_groups::discard_groups;
use rvmparser::hierarchy::dump_names::dump_names;
use rvmparser::visitor::stats::AddStats;
use rvmparser::visitor::apply_visitor;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_help(&args[0]);
        return ExitCode::FAILURE;
    }

    let mut tolerance = 0.1f32;
    let mut _cull_scale = -10000.1f32;
    let mut group_bounding_boxes = false;
    let mut keep_regex = String::new();
    let mut discard_groups_file = String::new();
    let mut keep_groups_file = String::new();
    let mut output_json_path = String::new();
    let mut output_txt_path = String::new();
    let mut output_rev_path = String::new();
    let mut output_obj_stem = String::new();
    let mut output_gltf_path = String::new();
    let mut output_gltf_rotate_z_to_y = true;
    let mut output_gltf_center = false;
    let mut output_gltf_attributes = true;
    let mut output_gltf_merge_geos = true;
    let mut output_gltf_split_level = 0usize;
    let mut color_attribute = String::new();
    let mut _chunk_tiny_threshold = 0u32;

    let mut should_tessellate = false;
    let mut should_colorize = false;
    let mut rvm_files = Vec::new();
    let mut att_files = Vec::new();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with("--") {
            if arg == "--help" {
                print_help(&args[0]);
                return ExitCode::SUCCESS;
            } else if arg == "--group-bounding-boxes" {
                group_bounding_boxes = true;
            } else if let Some((key, val)) = arg.split_once('=') {
                match key {
                    "--keep-groups" => keep_groups_file = val.to_string(),
                    "--keep-regex" => keep_regex = val.to_string(),
                    "--discard-groups" => discard_groups_file = val.to_string(),
                    "--output-json" => output_json_path = val.to_string(),
                    "--output-txt" => output_txt_path = val.to_string(),
                    "--output-rev" => output_rev_path = val.to_string(),
                    "--output-obj" => { output_obj_stem = val.to_string(); should_tessellate = true; should_colorize = true; }
                    "--output-gltf" => { output_gltf_path = val.to_string(); should_tessellate = true; should_colorize = true; }
                    "--output-gltf-rotate-z-to-y" => output_gltf_rotate_z_to_y = parse_bool(val),
                    "--output-gltf-center" => output_gltf_center = parse_bool(val),
                    "--output-gltf-attributes" => output_gltf_attributes = parse_bool(val),
                    "--output-gltf-merge-geos" => output_gltf_merge_geos = parse_bool(val),
                    "--output-gltf-split-level" => output_gltf_split_level = val.parse().unwrap_or(0),
                    "--color-attribute" => color_attribute = val.to_string(),
                    "--tolerance" => tolerance = val.parse::<f32>().unwrap_or(0.1).max(1e-6),
                    "--cull-scale" => _cull_scale = val.parse().unwrap_or(-10000.1),
                    "--chunk-tiny" => { _chunk_tiny_threshold = val.parse().unwrap_or(0); should_tessellate = true; }
                    _ => {
                        eprintln!("Unrecognized argument '{}'", arg);
                        print_help(&args[0]);
                        return ExitCode::FAILURE;
                    }
                }
            } else {
                eprintln!("Unrecognized argument '{}'", arg);
                return ExitCode::FAILURE;
            }
        } else {
            let lower = arg.to_lowercase();
            if lower.ends_with(".rvm") {
                rvm_files.push(arg.clone());
            } else if lower.ends_with(".txt") || lower.ends_with(".att") {
                att_files.push(arg.clone());
            } else {
                eprintln!("Unknown file type: {}", arg);
            }
        }
        i += 1;
    }

    let mut store = Store::new();

    // Parse RVM files
    for path in &rvm_files {
        let t0 = Instant::now();
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(e) => { eprintln!("[E] Failed to read {}: {}", path, e); return ExitCode::FAILURE; }
        };
        match parse_rvm(&data, path, &mut store) {
            Ok(()) => eprintln!("[I] Parsed {} ({:.0}ms)", path, t0.elapsed().as_millis()),
            Err(e) => { eprintln!("[E] Failed to parse {}: {}", path, e); return ExitCode::FAILURE; }
        }
    }

    // Parse ATT files
    for path in &att_files {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(e) => { eprintln!("[E] Failed to read {}: {}", path, e); return ExitCode::FAILURE; }
        };
        match parse_att(&data, &mut store) {
            Ok(()) => eprintln!("[I] Parsed {}", path),
            Err(e) => { eprintln!("[E] Failed to parse {}: {}", path, e); return ExitCode::FAILURE; }
        }
    }

    // Colorize
    if should_colorize {
        let attr = if color_attribute.is_empty() { None } else { Some(color_attribute.as_str()) };
        let colorizer = Colorizer::new(attr);
        colorizer.colorize(&mut store);
    }

    // Discard groups
    if !discard_groups_file.is_empty() {
        let text = match std::fs::read_to_string(&discard_groups_file) {
            Ok(t) => t,
            Err(e) => { eprintln!("[E] Failed to read {}: {}", discard_groups_file, e); return ExitCode::FAILURE; }
        };
        let n = discard_groups(&mut store, &text);
        eprintln!("[I] DiscardGroups: Discarded {} groups.", n);
    }

    // Flatten regex
    if !keep_regex.is_empty() {
        let t0 = Instant::now();
        match flatten_regex(&mut store, &keep_regex) {
            Ok(()) => {
                store.update_counts();
                eprintln!("[I] Flatten regex '{}' ({:.0}ms)", keep_regex, t0.elapsed().as_millis());
            }
            Err(e) => { eprintln!("[E] {}", e); return ExitCode::FAILURE; }
        }
    }

    // Connect and align
    {
        let t0 = Instant::now();
        let (matched, total) = connect(&mut store);
        eprintln!("[I] Matched {} of {} anchors ({:.0}ms)", matched, total, t0.elapsed().as_millis());
    }
    {
        let t0 = Instant::now();
        let (components, circular) = align(&mut store);
        eprintln!("[I] {} connected components in {} circular connections ({:.0}ms)", components, circular, t0.elapsed().as_millis());
    }

    // Add group bboxes
    if should_tessellate || !output_json_path.is_empty() {
        add_group_bboxes(&mut store);
    }

    // Tessellate
    if should_tessellate {
        let t0 = Instant::now();
        let mut count = 0u32;
        let geo_count = store.geometries.len();
        for i in 0..geo_count {
            let geo_id = rvmparser::store::geometry::GeometryId(i);
            if store.geometry(geo_id).triangulation.is_none() {
                if let Some(tri) = rvmparser::export::tessellator::tessellate(&store, geo_id, tolerance) {
                    store.geometry_mut(geo_id).triangulation = Some(tri);
                    count += 1;
                }
            }
        }
        eprintln!("[I] Tessellated {} items ({:.0}ms)", count, t0.elapsed().as_millis());
    }

    // Keep groups (Flatten)
    let mut do_flatten = false;
    let mut flatten_obj = Flatten::new(&store);
    if !keep_groups_file.is_empty() {
        let text = match std::fs::read_to_string(&keep_groups_file) {
            Ok(t) => t,
            Err(e) => { eprintln!("[E] Failed to read {}: {}", keep_groups_file, e); return ExitCode::FAILURE; }
        };
        flatten_obj.set_keep_from_text(&text, &store);
        do_flatten = true;
    }

    if do_flatten {
        store = flatten_obj.run(&store);
    }

    // Export JSON
    if !output_json_path.is_empty() {
        let t0 = Instant::now();
        match export_json(&store, &output_json_path) {
            Ok(()) => eprintln!("[I] Exported JSON to {} ({:.0}ms)", output_json_path, t0.elapsed().as_millis()),
            Err(e) => { eprintln!("[E] Failed to export JSON: {}", e); return ExitCode::FAILURE; }
        }
    }

    // Export TXT (dump names)
    if !output_txt_path.is_empty() {
        match dump_names(&store, &output_txt_path) {
            Ok(()) => eprintln!("[I] Dumped names to {}", output_txt_path),
            Err(e) => { eprintln!("[E] Failed to dump names: {}", e); return ExitCode::FAILURE; }
        }
    }

    // Export REV
    if !output_rev_path.is_empty() {
        let t0 = Instant::now();
        match export_rev(&store, &output_rev_path) {
            Ok(()) => eprintln!("[I] Exported REV to {} ({:.0}ms)", output_rev_path, t0.elapsed().as_millis()),
            Err(e) => { eprintln!("[E] Failed to export REV: {}", e); return ExitCode::FAILURE; }
        }
    }

    // Export OBJ
    if !output_obj_stem.is_empty() {
        let t0 = Instant::now();
        let obj_path = format!("{}.obj", output_obj_stem);
        let mtl_path = format!("{}.mtl", output_obj_stem);
        let options = ObjExportOptions { tolerance, group_bounding_boxes };
        match export_obj(&store, &obj_path, &mtl_path, &options) {
            Ok(()) => eprintln!("[I] Exported OBJ to {} ({:.0}ms)", output_obj_stem, t0.elapsed().as_millis()),
            Err(e) => { eprintln!("[E] Failed to export OBJ: {}", e); return ExitCode::FAILURE; }
        }
    }

    // Export GLTF/GLB
    if !output_gltf_path.is_empty() {
        let t0 = Instant::now();
        let options = GltfExportOptions {
            tolerance,
            rotate_z_to_y: output_gltf_rotate_z_to_y,
            center: output_gltf_center,
            include_attributes: output_gltf_attributes,
            merge_geos: output_gltf_merge_geos,
            split_level: output_gltf_split_level,
        };
        match export_gltf(&store, &output_gltf_path, &options) {
            Ok(()) => eprintln!("[I] Exported GLTF to {} ({:.0}ms)", output_gltf_path, t0.elapsed().as_millis()),
            Err(e) => { eprintln!("[E] Failed to export GLTF: {}", e); return ExitCode::FAILURE; }
        }
    }

    // Stats
    let mut stats_visitor = AddStats::new();
    apply_visitor(&store, &mut stats_visitor);
    let s = &stats_visitor.stats;
    eprintln!("[I] Stats:");
    eprintln!("[I]     Groups                 {}", s.group_n);
    if s.group_n > 0 {
        eprintln!("[I]     Geometries             {} (grp avg={:.1})", s.geometry_n, s.geometry_n as f32 / s.group_n as f32);
    }
    eprintln!("[I]         Pyramids           {}", s.pyramid_n);
    eprintln!("[I]         Boxes              {}", s.box_n);
    eprintln!("[I]         Rectangular tori   {}", s.rectangular_torus_n);
    eprintln!("[I]         Circular tori      {}", s.circular_torus_n);
    eprintln!("[I]         Elliptical dish    {}", s.elliptical_dish_n);
    eprintln!("[I]         Spherical dish     {}", s.spherical_dish_n);
    eprintln!("[I]         Snouts             {}", s.snout_n);
    eprintln!("[I]         Cylinders          {}", s.cylinder_n);
    eprintln!("[I]         Spheres            {}", s.sphere_n);
    eprintln!("[I]         Facet groups       {}", s.facetgroup_n);
    eprintln!("[I]         Lines              {}", s.line_n);

    ExitCode::SUCCESS
}

fn parse_bool(val: &str) -> bool {
    matches!(val.to_lowercase().as_str(), "true" | "1" | "yes")
}

fn print_help(argv0: &str) {
    eprintln!(r#"
Usage: {} [options] files

Files with .rvm-suffix will be interpreted as geometry files, and files with .txt or .att suffix
will be interpreted as attribute files. A rvm file typically has a matching attribute file.

Options:
  --keep-regex=<regex>                 Prune hierarchy by regex matching.
  --keep-groups=filename.txt           List of group names to keep.
  --discard-groups=filename.txt        List of group names to discard.
  --output-json=<filename.json>        Write hierarchy with attributes to JSON.
  --output-txt=<filename.txt>          Dump all group names to a text file.
  --output-rev=filename.rev            Write database as .rev text file.
  --output-obj=<filenamestem>          Write geometry as OBJ/MTL.
  --output-gltf=<filename.gltf|.glb>  Write geometry as GLTF/GLB.
  --output-gltf-attributes=<bool>      Include attributes in GLTF extras (default: true).
  --output-gltf-center=<bool>          Center model (default: false).
  --output-gltf-rotate-z-to-y=<bool>  Rotate Z to Y (default: true).
  --output-gltf-merge-geos=<bool>     Merge geometries (default: true).
  --output-gltf-split-level=<uint>    Split level (default: 0).
  --group-bounding-boxes              Include bounding boxes.
  --color-attribute=key               Attribute key for color.
  --tolerance=value                   Tessellation tolerance (default: 0.1).
  --cull-scale=value                  Cull scale factor.
"#, argv0);
}
