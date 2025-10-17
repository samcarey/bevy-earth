use std::f32::consts::PI;

use crate::errors::CoordError;
use bevy::prelude::*;
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy_mod_picking::prelude::*;
use gdal::errors::GdalError;
use gdal::raster::ResampleAlg;
use gdal::spatial_ref::{CoordTransform, SpatialRef};
use gdal::Dataset;

const EARTH_RADIUS: f32 = 300.0;

pub fn generate_faces(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Get raster map
    let rs =
        RasterData::new("assets/WorldElevation/ETOPO_2022_v1_60s_N90W180_surface.tif").unwrap();

    let faces = vec![
        Vec3::X,
        Vec3::NEG_X,
        Vec3::Y,
        Vec3::NEG_Y,
        Vec3::Z,
        Vec3::NEG_Z,
    ];

    let offsets = vec![(0.0, 0.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0)];

    let _rng = rand::thread_rng();

    for direction in faces {
        for offset in &offsets {
            commands.spawn((
                PbrBundle {
                    mesh: meshes.add(generate_face(direction, 600, offset.0, offset.1, &rs)),
                    material: materials.add(StandardMaterial {
                        base_color_texture: Some(
                            asset_server.load("WorldTextures/earth_color_10K.png"),
                        ),
                        metallic_roughness_texture: Some(
                            asset_server.load("WorldTextures/specular_map_inverted_8k.png"),
                        ),
                        perceptual_roughness: 1.0,
                        // normal_map_texture: Some(
                        //     asset_server.load("WorldTextures/topography_21K.png"),
                        // ),
                        ..default()
                    }),
                    ..default()
                },
                PickableBundle::default(), // Makes the entity pickable
                RaycastPickTarget::default(),
                On::<Pointer<Click>>::run(|event: Listener<Pointer<Click>>| {
                    info!("Clicked on entity {:?}", event);
                    let hit = event.hit;
                    if let Some(pos) = hit.position {
                        let coords: Coordinates = pos.into();
                        let (latitude, longitude) = coords.as_degrees();
                        info!(
                            "Latlon of selected point: Lat: {}, Lon: {}",
                            latitude, longitude
                        );
                    }
                }),
            ));
        }
    }
}

pub fn generate_face(
    normal: Vec3,
    resolution: u32,
    x_offset: f32,
    y_offset: f32,
    rs: &RasterData,
) -> Mesh {
    let axis_a = Vec3::new(normal.y, normal.z, normal.x); // Horizontal
    let axis_b = axis_a.cross(normal); // Vertical

    // Create a vec of verticies and indicies
    let mut verticies: Vec<Vec3> = Vec::new();
    let mut uvs = Vec::new();
    let mut indicies: Vec<u32> = Vec::new();
    let mut normals = Vec::new();
    let mut first_longitude = 0.0;
    for y in 0..(resolution) {
        for x in 0..(resolution) {
            let i = x + y * resolution;

            let percent = Vec2::new(x as f32, y as f32) / (resolution - 1) as f32;
            let point_on_unit_cube =
                normal + (percent.x - x_offset) * axis_a + (percent.y - y_offset) * axis_b;
            let point_coords: Coordinates = point_on_unit_cube.normalize().into();
            let (lat, lon) = point_coords.as_degrees();

            let height_offset = rs.get_coordinate_height(lat as f64, lon as f64);
            let normalized_point = if let Ok(Some(offset)) = height_offset {
                let height = if offset > 0.0 { offset / 300.0 } else { 0.0 };
                point_on_unit_cube.normalize() * (EARTH_RADIUS + (height) as f32)
            } else {
                point_on_unit_cube.normalize() * EARTH_RADIUS
            };

            verticies.push(normalized_point);
            let (mut u, v) = point_coords.convert_to_uv_mercator();

            if y == 0 && x == 0 {
                first_longitude = lon;
            }
            // In the middle latitudes, if we start on a negative longitude but then wind up crossing to a positive longitude, set u to 0.0 to prevent a seam
            if first_longitude < 0.0 && lon > 0.0 && lat < 89.0 && lat > -89.0 {
                u = 0.0;
            }
            // If we are below -40 degrees latitude and the tile starts at 180 degrees, set u to 0.0 to prevent a seam
            if x == 0 && lon == 180.0 && lat < -40.0 {
                u = 0.0;
            }
            uvs.push([u, v]);
            normals.push(-point_on_unit_cube.normalize());

            if x != resolution - 1 && y != resolution - 1 {
                // First triangle
                indicies.push(i);
                indicies.push(i + resolution);
                indicies.push(i + resolution + 1);

                // Second triangle
                indicies.push(i);
                indicies.push(i + resolution + 1);
                indicies.push(i + 1);
            }
        }
    }
    let indicies = mesh::Indices::U32(indicies);
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indicies));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verticies);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.generate_tangents().unwrap();
    mesh
}

pub fn generate_mesh() -> Mesh {
    let vertices = [
        ([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0]),
        ([1.0, 2.0, 1.0], [0.0, 1.0, 0.0], [1.0, 1.0]),
        ([2.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0]),
    ];
    let indices = mesh::Indices::U32(vec![0, 2, 1, 0, 3, 2]);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for (position, normal, uv) in vertices.iter() {
        positions.push(*position);
        normals.push(*normal);
        uvs.push(*uv);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

#[derive(Debug)]
pub struct Coordinates {
    // Stored internally in radians
    pub latitude: f32,
    pub longitude: f32,
}

impl From<Vec3> for Coordinates {
    fn from(value: Vec3) -> Self {
        let normalized_point = value.normalize();
        let latitude = normalized_point.y.asin();
        let longitude = normalized_point.x.atan2(normalized_point.z);
        Coordinates {
            latitude,
            longitude,
        }
    }
}

impl Coordinates {
    pub fn as_degrees(&self) -> (f32, f32) {
        let latitude = self.latitude * (180.0 / PI);
        let longitude = self.longitude * (180.0 / PI);
        (latitude, longitude)
    }

    pub fn convert_to_uv_mercator(&self) -> (f32, f32) {
        let (lat, lon) = self.as_degrees();
        let v = map_latitude(lat).unwrap();
        let u = map_longitude(lon).unwrap();
        (u, v)
    }

    #[allow(dead_code)]
    pub fn from_degrees(latitude: f32, longitude: f32) -> Result<Self, CoordError> {
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(CoordError {
                msg: "Invalid latitude: {lat:?}".to_string(),
            });
        }
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(CoordError {
                msg: "Invalid longitude: {lon:?}".to_string(),
            });
        }
        let latitude = latitude / (180.0 / PI);
        let longitude = longitude / (180.0 / PI);
        Ok(Coordinates {
            latitude,
            longitude,
        })
    }

    pub fn get_point_on_sphere(&self) -> Vec3 {
        let y = self.latitude.sin();
        let r = self.latitude.cos();
        let x = self.longitude.sin() * r;
        let z = self.longitude.cos() * r;
        Vec3::new(x, y, z).normalize() * EARTH_RADIUS
    }

    /// Calculate great circle arc between two coordinates with adjustable height
    pub fn arc_to(&self, other: &Coordinates, num_segments: u32, arc_height: f32) -> Vec<Vec3> {
        let start_point = self.get_point_on_sphere().normalize();
        let end_point = other.get_point_on_sphere().normalize();
        
        // Calculate the angle between the two points
        let dot_product = start_point.dot(end_point).clamp(-1.0, 1.0);
        let angle = dot_product.acos();
        
        // If points are very close, just return direct line
        if angle < 0.001 {
            return vec![
                start_point * EARTH_RADIUS,
                end_point * EARTH_RADIUS
            ];
        }
        
        let mut points = Vec::new();
        
        for i in 0..=num_segments {
            let t = i as f32 / num_segments as f32;
            
            // Spherical linear interpolation (slerp)
            let sin_angle = angle.sin();
            let a = ((1.0 - t) * angle).sin() / sin_angle;
            let b = (t * angle).sin() / sin_angle;
            
            let interpolated = (start_point * a + end_point * b).normalize();
            
            // Calculate height offset using a parabolic curve
            // Height is 0 at endpoints (t=0 and t=1) and maximum at t=0.5
            let height_multiplier = 4.0 * t * (1.0 - t); // Parabolic curve: peaks at t=0.5
            let height_offset = arc_height * height_multiplier;
            
            // Apply the height offset
            let radius = EARTH_RADIUS + height_offset;
            points.push(interpolated * radius);
        }
        
        points
    }
}

/// Component to store arc line data
#[derive(Component)]
pub struct ArcLine {
    pub from: Coordinates,
    pub to: Coordinates,
    pub color: Color,
    pub segments: u32,
    pub arc_height: f32,  // Height above the sphere surface at the arc's peak
}

impl ArcLine {
    pub fn new(from_lat: f32, from_lon: f32, to_lat: f32, to_lon: f32) -> Result<Self, CoordError> {
        Ok(Self {
            from: Coordinates::from_degrees(from_lat, from_lon)?,
            to: Coordinates::from_degrees(to_lat, to_lon)?,
            color: Color::YELLOW,
            segments: 50,
            arc_height: 50.0,  // Default height above surface
        })
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_segments(mut self, segments: u32) -> Self {
        self.segments = segments;
        self
    }

    pub fn with_arc_height(mut self, height: f32) -> Self {
        self.arc_height = height;
        self
    }
}

/// System to spawn arc line meshes
pub fn spawn_arc_line_meshes(
    mut commands: Commands,
    query: Query<(Entity, &ArcLine), Added<ArcLine>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, arc) in query.iter() {
        let points = arc.from.arc_to(&arc.to, arc.segments, arc.arc_height);
        let line_mesh = create_line_mesh(&points, 1.0); // Line thickness
        
        commands.entity(entity).insert(PbrBundle {
            mesh: meshes.add(line_mesh),
            material: materials.add(StandardMaterial {
                base_color: arc.color,
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            ..default()
        });
    }
}

/// Create a mesh representing a line with thickness (double-sided)
fn create_line_mesh(points: &[Vec3], thickness: f32) -> Mesh {
    if points.len() < 2 {
        return Mesh::new(PrimitiveTopology::TriangleList);
    }

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    let mut uvs = Vec::new();

    for i in 0..(points.len() - 1) {
        let start = points[i];
        let end = points[i + 1];
        
        // Calculate direction and perpendicular vectors for the line segment
        let direction = (end - start).normalize();
        let to_center = -start.normalize(); // Vector pointing toward earth center
        let perpendicular = direction.cross(to_center).normalize();
        
        // Create a quad for this line segment
        let half_thickness = thickness * 0.5;
        
        // Four corners of the quad
        let v0 = start - perpendicular * half_thickness;
        let v1 = start + perpendicular * half_thickness;
        let v2 = end + perpendicular * half_thickness;
        let v3 = end - perpendicular * half_thickness;
        
        let base_index = vertices.len() as u32;
        
        // Add vertices for front-facing quad (outward normals)
        vertices.extend_from_slice(&[v0, v1, v2, v3]);
        let outward_normal = start.normalize();
        normals.extend_from_slice(&[outward_normal, outward_normal, outward_normal, outward_normal]);
        uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        
        // Add indices for front-facing triangles
        indices.extend_from_slice(&[
            base_index, base_index + 1, base_index + 2,
            base_index, base_index + 2, base_index + 3,
        ]);
        
        // Add vertices for back-facing quad (inward normals)
        vertices.extend_from_slice(&[v0, v1, v2, v3]);
        let inward_normal = -start.normalize();
        normals.extend_from_slice(&[inward_normal, inward_normal, inward_normal, inward_normal]);
        uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        
        // Add indices for back-facing triangles (reversed winding order)
        let back_base = base_index + 4;
        indices.extend_from_slice(&[
            back_base, back_base + 2, back_base + 1,
            back_base, back_base + 3, back_base + 2,
        ]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(mesh::Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

fn map_latitude(lat: f32) -> Result<f32, CoordError> {
    // 90 -> 0 maps to 0.0 to 0.5
    // 0 -> -90 maps to 0.5 to 1.0
    // Ensure latitude is valid
    if !(-90.0..=90.0).contains(&lat) {
        return Err(CoordError {
            msg: "Invalid latitude: {lat:?}".to_string(),
        });
    }
    if (90.0..=0.0).contains(&lat) {
        Ok(map((90.0, 0.0), (0.0, 0.5), lat))
    } else {
        Ok(map((0.0, -90.0), (0.5, 1.0), lat))
    }
}

fn map_longitude(lon: f32) -> Result<f32, CoordError> {
    // -180 -> 0 maps to 0.0 to 0.5
    // 0 -> 180 maps to 0.5 to 1.0
    //Ensure longitude is valid
    if !(-180.0..=180.0).contains(&lon) {
        return Err(CoordError {
            msg: "Invalid longitude: {lon:?}".to_string(),
        });
    }
    if (-180.0..=0.0).contains(&lon) {
        Ok(map((-180.0, 0.0), (0.0, 0.5), lon))
    } else {
        Ok(map((0.0, 180.0), (0.5, 1.0), lon))
    }
}

fn map(range_a: (f32, f32), range_b: (f32, f32), value: f32) -> f32 {
    range_b.0 + (value - range_a.0) * (range_b.1 - range_b.0) / (range_a.1 - range_a.0)
}

pub struct RasterData {
    pub dataset: Dataset,
    pub transform: CoordTransform,
}

impl RasterData {
    pub fn new(path: &str) -> Result<Self, GdalError> {
        let dataset = Dataset::open(path)?;
        let srs = dataset.spatial_ref()?;
        let target_srs = SpatialRef::from_epsg(4326)?;
        let transform = gdal::spatial_ref::CoordTransform::new(&srs, &target_srs)?;
        Ok(Self { dataset, transform })
    }
    pub fn get_coordinate_height(
        &self,
        latitude: f64,
        longitude: f64,
    ) -> Result<Option<f64>, GdalError> {
        let (lat, lon) = (latitude, longitude);
        self.transform
            .transform_coords(&mut [lon], &mut [lat], &mut [])?;
        let raster_band = self.dataset.rasterband(1)?;
        let transform = self.dataset.geo_transform().unwrap();
        let x = (lon - transform[0]) / transform[1];
        let y = (lat - transform[3]) / transform[5];
        let mut res_buffer = raster_band.read_as::<f64>(
            (x as isize, y as isize),
            (1, 1),
            (1, 1),
            Some(ResampleAlg::Average),
        )?;
        Ok(res_buffer.data.pop())
    }
}

pub fn load_tiff() {
    let ds = Dataset::open("assets/WorldElevation/black_sea.tif").unwrap();

    let tgt_latitude = 44.579543;
    let tgt_longitude = 33.396264;

    println!(
        "This {} is in '{}' and has {} bands.",
        ds.driver().long_name(),
        ds.spatial_ref().unwrap().name().unwrap(),
        ds.raster_count()
    );
    let srs = ds.spatial_ref().unwrap();
    let target_srs = SpatialRef::from_epsg(4326).unwrap();
    let transform = gdal::spatial_ref::CoordTransform::new(&srs, &target_srs).unwrap();
    transform
        .transform_coords(&mut [tgt_longitude], &mut [tgt_latitude], &mut [])
        .unwrap();
    println!("Target coords: {tgt_latitude}, {tgt_longitude}");
    let raster_band = ds.rasterband(1).unwrap();
    let mut res_buffer = raster_band
        .read_as::<f64>(
            (tgt_latitude as isize, tgt_longitude as isize),
            (1, 1),
            (1, 1),
            None,
        )
        .unwrap();
    let value = res_buffer.data.pop().unwrap();
    println!("Value: {value:?}");
}

#[cfg(test)]
mod tests {
    use gdal::{programs::raster, raster::ResampleAlg};

    use super::*;
    use crate::map::{map_latitude, map_longitude};

    #[test]
    fn test_latitude_mapping() {
        let north_pole = 90.0;
        let south_pole = -90.0;
        let equator = 0.0;

        assert_eq!(map_latitude(north_pole).unwrap(), 0.0);
        assert_eq!(map_latitude(south_pole).unwrap(), 1.0);
        assert_eq!(map_latitude(equator).unwrap(), 0.5);
    }

    #[test]
    fn test_longitude_mapping() {
        let west = -180.0;
        let east = 180.0;
        let meridian = 0.0;
        assert_eq!(map_longitude(west).unwrap(), 0.0);
        assert_eq!(map_longitude(east).unwrap(), 1.0);
        assert_eq!(map_longitude(meridian).unwrap(), 0.5);
    }

    #[test]
    fn test_latlon_to_uv_mapping() {
        let cords = Coordinates::from_degrees(90.0, 180.0).unwrap();
        let (u, v) = cords.convert_to_uv_mercator();
        assert_eq!(v, 0.0);
        assert_eq!(u, 1.0);
    }

    #[test]
    fn test_raster_map() {
        let raster_data =
            RasterData::new("assets/Bathymetry/gebco_2023_n47.7905_s39.9243_w25.6311_e42.9895.tif")
                .unwrap();

        // Mt Elbrus
        let tgt_latitude = 43.351851;
        let tgt_longitude = 42.4368771;

        let elevation = raster_data
            .get_coordinate_height(tgt_latitude, tgt_longitude)
            .unwrap()
            .unwrap();

        assert_eq!(elevation, 5392.0);
    }

}
