use std::fs::File;
use std::io::Write;
use std::collections::BTreeMap;
use rand::Rng;
use vorothree::{BoundingBox, Tessellation, AlgorithmGrid, Wall, CellFaces};
use gltf::json;
use gltf::json::validation::{Checked, USize64};
use vorothree::geometries::{ConeGeometry, TrefoilKnotGeometry, PlaneGeometry, SphereGeometry, CylinderGeometry, TorusGeometry};

struct GltfBuilder {
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
    edge_indices: Vec<u32>,
    gen_positions: Vec<[f32; 3]>,
    gen_indices: Vec<u32>,
}

impl GltfBuilder {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            indices: Vec::new(),
            edge_indices: Vec::new(),
            gen_positions: Vec::new(),
            gen_indices: Vec::new(),
        }
    }

    fn add_cell(&mut self, vertices: &[f64], faces: &[Vec<usize>]) {
        let base_index = self.positions.len() as u32;
        
        // Add vertices
        for i in 0..(vertices.len() / 3) {
            self.positions.push([
                vertices[i * 3] as f32,
                vertices[i * 3 + 1] as f32,
                vertices[i * 3 + 2] as f32,
            ]);
        }

        // Add faces (triangulated)
        for face in faces {
            if face.len() < 3 { continue; }
            // Fan triangulation for convex polygon
            let v0 = face[0] as u32;
            for i in 1..face.len() - 1 {
                let v1 = face[i] as u32;
                let v2 = face[i + 1] as u32;
                self.indices.push(base_index + v0);
                self.indices.push(base_index + v1);
                self.indices.push(base_index + v2);
            }

            // Add edges
            for i in 0..face.len() {
                let v1 = face[i] as u32;
                let v2 = face[(i + 1) % face.len()] as u32;
                self.edge_indices.push(base_index + v1);
                self.edge_indices.push(base_index + v2);
            }
        }
    }

    fn add_generators(&mut self, generators: &[f64]) {
        let r = 0.5; // Radius for the marker
        for i in 0..(generators.len() / 3) {
            let cx = generators[i * 3] as f32;
            let cy = generators[i * 3 + 1] as f32;
            let cz = generators[i * 3 + 2] as f32;
            
            let base = self.gen_positions.len() as u32;
            
            // Octahedron vertices (6)
            self.gen_positions.push([cx + r, cy, cz]); // 0: +x
            self.gen_positions.push([cx - r, cy, cz]); // 1: -x
            self.gen_positions.push([cx, cy + r, cz]); // 2: +y
            self.gen_positions.push([cx, cy - r, cz]); // 3: -y
            self.gen_positions.push([cx, cy, cz + r]); // 4: +z
            self.gen_positions.push([cx, cy, cz - r]); // 5: -z

            // Octahedron indices (8 faces)
            let indices = [
                2, 4, 0,  2, 1, 4,  2, 5, 1,  2, 0, 5, // Top half
                3, 0, 4,  3, 4, 1,  3, 1, 5,  3, 5, 0  // Bottom half
            ];
            self.gen_indices.extend(indices.iter().map(|&idx| base + idx));
        }
    }

    fn save(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Prepare buffers
        let mut buffer_data = Vec::new();
        
        // Positions (Vec3 f32)
        let pos_offset = buffer_data.len();
        for p in &self.positions {
            for c in p {
                buffer_data.write_all(&c.to_le_bytes())?;
            }
        }
        let pos_len = buffer_data.len() - pos_offset;
        
        // Padding for indices (must be aligned to 4 bytes)
        while buffer_data.len() % 4 != 0 {
            buffer_data.push(0);
        }

        // Indices (Scalar u32)
        let ind_offset = buffer_data.len();
        for i in &self.indices {
            buffer_data.write_all(&i.to_le_bytes())?;
        }
        let ind_len = buffer_data.len() - ind_offset;

        // Padding for edge indices
        while buffer_data.len() % 4 != 0 {
            buffer_data.push(0);
        }

        // Edge Indices (Scalar u32)
        let edge_ind_offset = buffer_data.len();
        for i in &self.edge_indices {
            buffer_data.write_all(&i.to_le_bytes())?;
        }
        let edge_ind_len = buffer_data.len() - edge_ind_offset;

        // Padding for generators
        while buffer_data.len() % 4 != 0 {
            buffer_data.push(0);
        }

        // Generator Positions (Vec3 f32)
        let gen_pos_offset = buffer_data.len();
        for p in &self.gen_positions {
            for c in p {
                buffer_data.write_all(&c.to_le_bytes())?;
            }
        }
        let gen_pos_len = buffer_data.len() - gen_pos_offset;

        // Padding for generator indices
        while buffer_data.len() % 4 != 0 {
            buffer_data.push(0);
        }

        // Generator Indices (Scalar u32)
        let gen_ind_offset = buffer_data.len();
        for i in &self.gen_indices {
            buffer_data.write_all(&i.to_le_bytes())?;
        }
        let gen_ind_len = buffer_data.len() - gen_ind_offset;

        // Min/Max for positions
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for p in &self.positions {
            for i in 0..3 {
                if p[i] < min[i] { min[i] = p[i]; }
                if p[i] > max[i] { max[i] = p[i]; }
            }
        }

        // Min/Max for generators
        let mut gen_min = [f32::MAX; 3];
        let mut gen_max = [f32::MIN; 3];
        for p in &self.gen_positions {
            for i in 0..3 {
                if p[i] < gen_min[i] { gen_min[i] = p[i]; }
                if p[i] > gen_max[i] { gen_max[i] = p[i]; }
            }
        }

        let buffer = json::Buffer {
            byte_length: USize64(buffer_data.len() as u64),
            uri: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
        };

        let buffer_view_pos = json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: USize64(pos_len as u64),
            byte_offset: Some(USize64(pos_offset as u64)),
            byte_stride: Some(json::buffer::Stride(12)),
            name: None,
            target: Some(Checked::Valid(json::buffer::Target::ArrayBuffer)),
            extensions: Default::default(),
            extras: Default::default(),
        };

        let buffer_view_ind = json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: USize64(ind_len as u64),
            byte_offset: Some(USize64(ind_offset as u64)),
            byte_stride: None,
            name: None,
            target: Some(Checked::Valid(json::buffer::Target::ElementArrayBuffer)),
            extensions: Default::default(),
            extras: Default::default(),
        };

        let buffer_view_edge_ind = json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: USize64(edge_ind_len as u64),
            byte_offset: Some(USize64(edge_ind_offset as u64)),
            byte_stride: None,
            name: None,
            target: Some(Checked::Valid(json::buffer::Target::ElementArrayBuffer)),
            extensions: Default::default(),
            extras: Default::default(),
        };

        let buffer_view_gen_pos = json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: USize64(gen_pos_len as u64),
            byte_offset: Some(USize64(gen_pos_offset as u64)),
            byte_stride: Some(json::buffer::Stride(12)),
            name: None,
            target: Some(Checked::Valid(json::buffer::Target::ArrayBuffer)),
            extensions: Default::default(),
            extras: Default::default(),
        };

        let buffer_view_gen_ind = json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: USize64(gen_ind_len as u64),
            byte_offset: Some(USize64(gen_ind_offset as u64)),
            byte_stride: None,
            name: None,
            target: Some(Checked::Valid(json::buffer::Target::ElementArrayBuffer)),
            extensions: Default::default(),
            extras: Default::default(),
        };

        let accessor_pos = json::Accessor {
            buffer_view: Some(json::Index::new(0)),
            byte_offset: Some(USize64(0)),
            count: USize64(self.positions.len() as u64),
            component_type: Checked::Valid(json::accessor::GenericComponentType(json::accessor::ComponentType::F32)),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Checked::Valid(json::accessor::Type::Vec3),
            min: Some(json::Value::from(Vec::from(min))),
            max: Some(json::Value::from(Vec::from(max))),
            name: None,
            normalized: false,
            sparse: None,
        };

        let accessor_ind = json::Accessor {
            buffer_view: Some(json::Index::new(1)),
            byte_offset: Some(USize64(0)),
            count: USize64(self.indices.len() as u64),
            component_type: Checked::Valid(json::accessor::GenericComponentType(json::accessor::ComponentType::U32)),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Checked::Valid(json::accessor::Type::Scalar),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        };

        let accessor_edge_ind = json::Accessor {
            buffer_view: Some(json::Index::new(2)),
            byte_offset: Some(USize64(0)),
            count: USize64(self.edge_indices.len() as u64),
            component_type: Checked::Valid(json::accessor::GenericComponentType(json::accessor::ComponentType::U32)),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Checked::Valid(json::accessor::Type::Scalar),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        };

        let accessor_gen_pos = json::Accessor {
            buffer_view: Some(json::Index::new(3)),
            byte_offset: Some(USize64(0)),
            count: USize64(self.gen_positions.len() as u64),
            component_type: Checked::Valid(json::accessor::GenericComponentType(json::accessor::ComponentType::F32)),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Checked::Valid(json::accessor::Type::Vec3),
            min: Some(json::Value::from(Vec::from(gen_min))),
            max: Some(json::Value::from(Vec::from(gen_max))),
            name: None,
            normalized: false,
            sparse: None,
        };

        let accessor_gen_ind = json::Accessor {
            buffer_view: Some(json::Index::new(4)),
            byte_offset: Some(USize64(0)),
            count: USize64(self.gen_indices.len() as u64),
            component_type: Checked::Valid(json::accessor::GenericComponentType(json::accessor::ComponentType::U32)),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Checked::Valid(json::accessor::Type::Scalar),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        };

        let material = json::Material {
            alpha_cutoff: None,
            alpha_mode: Checked::Valid(json::material::AlphaMode::Blend),
            double_sided: true,
            name: Some("TransparentBlue".to_string()),
            pbr_metallic_roughness: json::material::PbrMetallicRoughness {
                base_color_factor: json::material::PbrBaseColorFactor([0.0, 0.0, 1.0, 0.1]),
                metallic_factor: json::material::StrengthFactor(0.0),
                roughness_factor: json::material::StrengthFactor(0.5),
                ..Default::default()
            },
            ..Default::default()
        };

        let material_edges = json::Material {
            pbr_metallic_roughness: json::material::PbrMetallicRoughness {
                base_color_factor: json::material::PbrBaseColorFactor([0.0, 0.0, 0.0, 1.0]),
                ..Default::default()
            },
            name: Some("BlackEdges".to_string()),
            ..Default::default()
        };

        let material_points = json::Material {
            pbr_metallic_roughness: json::material::PbrMetallicRoughness {
                base_color_factor: json::material::PbrBaseColorFactor([1.0, 0.0, 0.0, 1.0]),
                ..Default::default()
            },
            name: Some("RedPoints".to_string()),
            ..Default::default()
        };

        let primitive = json::mesh::Primitive {
            attributes: {
                let mut map = BTreeMap::new();
                map.insert(Checked::Valid(json::mesh::Semantic::Positions), json::Index::new(0));
                map
            },
            extensions: Default::default(),
            extras: Default::default(),
            indices: Some(json::Index::new(1)),
            material: Some(json::Index::new(0)),
            mode: Checked::Valid(json::mesh::Mode::Triangles),
            targets: None,
        };

        let primitive_edges = json::mesh::Primitive {
            attributes: {
                let mut map = BTreeMap::new();
                map.insert(Checked::Valid(json::mesh::Semantic::Positions), json::Index::new(0));
                map
            },
            extensions: Default::default(),
            extras: Default::default(),
            indices: Some(json::Index::new(2)),
            material: Some(json::Index::new(1)),
            mode: Checked::Valid(json::mesh::Mode::Lines),
            targets: None,
        };

        let primitive_points = json::mesh::Primitive {
            attributes: {
                let mut map = BTreeMap::new();
                map.insert(Checked::Valid(json::mesh::Semantic::Positions), json::Index::new(3));
                map
            },
            extensions: Default::default(),
            extras: Default::default(),
            indices: Some(json::Index::new(4)),
            material: Some(json::Index::new(2)),
            mode: Checked::Valid(json::mesh::Mode::Triangles),
            targets: None,
        };

        let mesh = json::Mesh {
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            primitives: vec![primitive, primitive_edges, primitive_points],
            weights: None,
        };

        let node = json::Node {
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            mesh: Some(json::Index::new(0)),
            name: None,
            rotation: None,
            scale: None,
            skin: None,
            translation: None,
            weights: None,
        };

        let root = json::Root {
            accessors: vec![accessor_pos, accessor_ind, accessor_edge_ind, accessor_gen_pos, accessor_gen_ind],
            animations: vec![],
            asset: json::Asset {
                generator: Some("vorothree example".to_string()),
                version: "2.0".to_string(),
                ..Default::default()
            },
            buffers: vec![buffer],
            buffer_views: vec![buffer_view_pos, buffer_view_ind, buffer_view_edge_ind, buffer_view_gen_pos, buffer_view_gen_ind],
            cameras: vec![],
            extensions: Default::default(),
            extensions_used: vec![],
            extensions_required: vec![],
            extras: Default::default(),
            images: vec![],
            materials: vec![material, material_edges, material_points],
            meshes: vec![mesh],
            nodes: vec![node],
            samplers: vec![],
            scene: Some(json::Index::new(0)),
            scenes: vec![json::Scene {
                extensions: Default::default(),
                extras: Default::default(),
                name: None,
                nodes: vec![json::Index::new(0)],
            }],
            skins: vec![],
            textures: vec![],
        };

        let json_string = json::serialize::to_string(&root)?;
        let mut json_bytes = json_string.into_bytes();
        
        // Pad JSON to 4 bytes with spaces
        while json_bytes.len() % 4 != 0 {
            json_bytes.push(0x20);
        }

        let total_length = 12 + 8 + json_bytes.len() as u32 + 8 + buffer_data.len() as u32;

        let mut file = File::create(filename)?;
        
        // Header
        file.write_all(b"glTF")?;
        file.write_all(&2u32.to_le_bytes())?;
        file.write_all(&total_length.to_le_bytes())?;

        // JSON Chunk
        file.write_all(&(json_bytes.len() as u32).to_le_bytes())?;
        file.write_all(b"JSON")?;
        file.write_all(&json_bytes)?;

        // BIN Chunk
        file.write_all(&(buffer_data.len() as u32).to_le_bytes())?;
        file.write_all(b"BIN\0")?;
        file.write_all(&buffer_data)?;

        Ok(())
    }
}

fn generate_gltf(
    tess: &Tessellation::<3, CellFaces, AlgorithmGrid>,
    generators: &[f64],
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = GltfBuilder::new();

    for i in 0..tess.count_cells() {
        if let Some(cell) = tess.get_cell(i) {
            builder.add_cell(&cell.vertices(), &cell.faces());
        }
    }
    builder.add_generators(generators);

    builder.save(filename)?;
    println!("Output saved to {}", filename);
    Ok(())
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Setup the tessellation
    let bounds = BoundingBox::new([0.0, 0.0, 0.0], [100.0, 100.0, 100.0]);

    // Generate random points
    let mut rng = rand::thread_rng();
    let mut generators = Vec::new();
    for _ in 0..400 {
        generators.push(rng.gen_range(0.0..100.0));
        generators.push(rng.gen_range(0.0..100.0));
        generators.push(rng.gen_range(0.0..100.0));
    }

    // Run 1: Plane Wall
    {
        let mut tess = Tessellation::<3, CellFaces, _>::new(bounds.clone(), AlgorithmGrid::new(10, 10, 10, &bounds));
        tess.set_generators(&generators);

        tess.add_wall(Wall::new(
            -10,
            Box::new(PlaneGeometry::new([40.0, 40.0, 40.0], [1.0, 1.0, 1.0]))
        ));
        tess.calculate();
        generate_gltf(&tess, &generators, "wall_plane.glb")?;
    }

    // Run 2: Sphere Wall
    {
        let mut tess = Tessellation::<3, CellFaces, _>::new(bounds.clone(), AlgorithmGrid::new(10, 10, 10, &bounds));
        tess.set_generators(&generators);
        tess.add_wall(Wall::new(-11, Box::new(SphereGeometry::new([50.0, 50.0, 50.0], 40.0))));
        tess.calculate();
        generate_gltf(&tess, &generators, "wall_sphere.glb")?;
    }
    
    // Run 3: Cylinder Wall
    {
        let mut tess = Tessellation::<3, CellFaces, _>::new(bounds.clone(), AlgorithmGrid::new(10, 10, 10, &bounds));
        tess.set_generators(&generators);
        tess.add_wall(Wall::new(-12, Box::new(CylinderGeometry::new([50.0, 50.0, 50.0], [0.0, 0.0, 1.0], 40.0))));
        tess.calculate();
        generate_gltf(&tess, &generators, "wall_cylinder.glb")?;
    }

    // Run 4: Torus Wall
    {
        let mut tess = Tessellation::<3, CellFaces, _>::new(bounds.clone(), AlgorithmGrid::new(10, 10, 10, &bounds));
        tess.set_generators(&generators);
        tess.add_wall(Wall::new(-13, Box::new(TorusGeometry::new([50.0, 50.0, 50.0], [0.0, 0.0, 1.0], 35.0, 10.0))));
        tess.calculate();
        generate_gltf(&tess, &generators, "wall_torus.glb")?;
    }

    // Run 5: Cone Wall (Custom)
    {
        let mut tess = Tessellation::<3, CellFaces, _>::new(bounds.clone(), AlgorithmGrid::new(10, 10, 10, &bounds));
        tess.set_generators(&generators);
        tess.add_wall(Wall::new(-14, Box::new(ConeGeometry::new(
            [50.0, 50.0, 10.0],
            [0.0, 0.0, 1.0],
            30.0f64.to_radians(),
        ))));
        tess.calculate();
        generate_gltf(&tess, &generators, "wall_cone.glb")?;
    }

    // Run 6: Trefoil Knot Wall (Custom)
    {
        let mut tess = Tessellation::<3, CellFaces, _>::new(bounds.clone(), AlgorithmGrid::new(10, 10, 10, &bounds));
        tess.set_generators(&generators);
        tess.add_wall(Wall::new(-15, Box::new(TrefoilKnotGeometry::new(
            [50.0, 50.0, 50.0],
            12.0,
            8.0,
            200
        ))));
        tess.calculate();
        generate_gltf(&tess, &generators, "wall_knot.glb")?;
    }

    Ok(())
}