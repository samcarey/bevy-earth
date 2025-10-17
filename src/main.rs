use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;

use bevy_earth::map::{self, ArcLine, Coordinates};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_prototype_debug_lines::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Sample8)
        .add_plugins(DefaultPlugins)
        // .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(PanOrbitCameraPlugin)
        .add_plugin(DebugLinesPlugin::default())
        .add_plugins(DefaultPickingPlugins)
        .add_startup_system(spawn_scene)
        .add_startup_system(map::generate_faces)
        .add_startup_system(spawn_city_population_spheres)
        .add_startup_system(spawn_example_arc_lines)
        .add_startup_system(spawn_austin_arc_lines)
        .add_system(map::spawn_arc_line_meshes)
        // .add_system(direction_lines)
        .run();
}

fn spawn_visualization_bubbles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let coords = Coordinates::from_degrees(29.7604, 95.3698)
        .unwrap()
        .get_point_on_sphere();
    commands.spawn(PbrBundle {
        mesh: meshes.add(
            Mesh::try_from(shape::Icosphere {
                radius: 10.0,
                subdivisions: 32,
            })
            .unwrap(),
        ),
        material: materials.add(StandardMaterial {
            base_color: Color::hex("#ffd891").unwrap(),
            unlit: true,
            ..default()
        }),
        transform: Transform::from_xyz(coords.x, coords.y, coords.z),
        ..default()
    });
}

fn spawn_city_population_spheres(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Cities data: (name, latitude, longitude, population in millions)
    let major_cities: Vec<(String, f32, f32, f32)> = vec![
        (String::from("Tokyo"), 35.6762, 139.6503, 37.4),
        (String::from("Delhi"), 28.6139, 77.2090, 32.9),
        (String::from("Shanghai"), 31.2304, 121.4737, 28.5),
        (String::from("São Paulo"), -23.5505, -46.6333, 22.4),
        (String::from("Mexico City"), 19.4326, -99.1332, 22.2),
        (String::from("Cairo"), 30.0444, 31.2357, 21.3),
        (String::from("Mumbai"), 19.0760, 72.8777, 20.7),
        (String::from("Beijing"), 39.9042, 116.4074, 20.5),
        (String::from("Dhaka"), 23.8103, 90.4125, 19.6),
        (String::from("Osaka"), 34.6937, 135.5023, 19.2),
        (String::from("New York"), 40.7128, -74.0060, 18.8),
        (String::from("Karachi"), 24.8607, 67.0011, 16.5),
        (String::from("Buenos Aires"), -34.6037, -58.3816, 15.2),
        (String::from("Istanbul"), 41.0082, 28.9784, 15.1),
        (String::from("Kolkata"), 22.5726, 88.3639, 14.9),
        (String::from("Lagos"), 6.5244, 3.3792, 14.8),
        (String::from("London"), 51.5074, -0.1278, 14.3),
        (String::from("Los Angeles"), 34.0522, -118.2437, 13.2),
        (String::from("Manila"), 14.5995, 120.9842, 13.1),
        (String::from("Rio de Janeiro"), -22.9068, -43.1729, 13.0),
        (String::from("Tianjin"), 39.3434, 117.3616, 12.8),
        (String::from("Kinshasa"), -4.4419, 15.2663, 12.6),
        (String::from("Paris"), 48.8566, 2.3522, 11.1),
        (String::from("Shenzhen"), 22.5431, 114.0579, 10.6),
        (String::from("Jakarta"), -6.2088, 106.8456, 10.6),
        (String::from("Bangalore"), 12.9716, 77.5946, 10.5),
        (String::from("Moscow"), 55.7558, 37.6173, 10.5),
        (String::from("Chennai"), 13.0827, 80.2707, 10.0),
        (String::from("Lima"), -12.0464, -77.0428, 9.7),
        (String::from("Bangkok"), 13.7563, 100.5018, 9.6),
        (String::from("Seoul"), 37.5665, 126.9780, 9.5),
        (String::from("Hyderabad"), 17.3850, 78.4867, 9.5),
        (String::from("Chengdu"), 30.5728, 104.0668, 9.3),
        (String::from("Singapore"), 1.3521, 103.8198, 5.7),
        (String::from("Ho Chi Minh City"), 10.8231, 106.6297, 9.1),
        (String::from("Toronto"), 43.6532, -79.3832, 6.4),
        (String::from("Sydney"), -33.8688, 151.2093, 5.3),
        (String::from("Johannesburg"), -26.2041, 28.0473, 5.9),
        (String::from("Chicago"), 41.8781, -87.6298, 8.9),
        (String::from("Taipei"), 25.0330, 121.5654, 7.4),
    ];

    // Define constants for scaling the spheres
    const BASE_RADIUS: f32 = 2.0; // Minimum radius for smallest city
    const SCALE_FACTOR: f32 = 0.5; // Multiplier for population to radius conversion
    const MIN_POPULATION: f32 = 5.0; // For normalization purposes
    const MAX_POPULATION: f32 = 40.0; // For normalization purposes

    // Create a component to store city information
    #[derive(Component)]
    struct CityMarker {
        name: String,
        population: f32,
    }

    // Create a mesh that will be reused for all cities
    let sphere_mesh = meshes.add(
        Mesh::try_from(shape::Icosphere {
            radius: 1.0, // We'll scale this in the transform
            subdivisions: 32,
        })
        .unwrap(),
    );

    // Spawn a sphere for each city
    for (name, latitude, longitude, population) in major_cities {
        // Convert latitude and longitude to 3D coordinates on the sphere
        let coords = Coordinates::from_degrees(latitude, longitude)
            .unwrap()
            .get_point_on_sphere();

        // Calculate sphere size based on population
        // Using a logarithmic scale to prevent extremely large cities from dominating
        let normalized_population =
            (population - MIN_POPULATION) / (MAX_POPULATION - MIN_POPULATION);
        let size = BASE_RADIUS + (normalized_population * SCALE_FACTOR * 10.0);

        // Calculate color based on population (gradient from yellow to red)
        let t = normalized_population.clamp(0.0, 1.0);
        let color = Color::rgb(
            1.0,             // Red stays at 1.0
            1.0 - (t * 0.7), // Green decreases with population
            0.5 - (t * 0.4), // Blue decreases with population
        );

        // Spawn the city sphere
        commands.spawn((
            PbrBundle {
                mesh: sphere_mesh.clone(),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    unlit: true,
                    ..default()
                }),
                transform: Transform::from_translation(Vec3::new(coords.x, coords.y, coords.z))
                    .with_scale(Vec3::splat(size)),
                ..default()
            },
            CityMarker { name, population },
        ));
    }
}

fn spawn_scene(
    mut commands: Commands,
    _assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         color: Color::WHITE,
    //         range: 10000.0,
    //         intensity: 1500.0,
    //         radius: 4.0,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(0.0, 10.0, 30.0),
    //     ..default()
    // });
    ambient_light.color = Color::WHITE;
    ambient_light.brightness = 1.0;

    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 1.0,
        maximum_distance: 3.0,
        ..default()
    }
    .build();

    // Sun
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            illuminance: 400.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
        cascade_shadow_config,
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-400.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        RaycastPickCamera::default(),
        PanOrbitCamera {
            zoom_sensitivity: 0.1,
            orbit_sensitivity: 0.1,
            pan_sensitivity: 0.1,
            ..default()
        },
    ));
    // commands.spawn(FogSettings {
    //     color: Color::rgba(0.1, 0.2, 0.4, 1.0),
    //     directional_light_color: Color::rgba(1.0, 0.95, 0.75, 0.5),
    //     directional_light_exponent: 30.0,
    //     falloff: FogFalloff::from_visibility_colors(
    //         15.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
    //         Color::rgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
    //         Color::rgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
    //     ),
    // });

    // let coords = Coordinates::from_degrees(29.7604, 95.3698)
    //     .unwrap()
    //     .get_point_on_sphere();
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(
    //         Mesh::try_from(shape::Icosphere {
    //             radius: 10.0,
    //             subdivisions: 32,
    //         })
    //         .unwrap(),
    //     ),
    //     material: materials.add(StandardMaterial {
    //         base_color: Color::hex("#ffd891").unwrap(),
    //         unlit: true,
    //         ..default()
    //     }),
    //     transform: Transform::from_xyz(coords.x, coords.y, coords.z),
    //     ..default()
    // });
}

fn spawn_example_arc_lines(mut commands: Commands) {
    // Example arc lines between major cities with varying heights
    if let Ok(arc) = ArcLine::new(40.7128, -74.0060, 51.5074, -0.1278) { // New York to London
        commands.spawn(arc.with_color(Color::RED).with_segments(60).with_arc_height(30.0));
    }
    
    if let Ok(arc) = ArcLine::new(35.6762, 139.6503, -34.6037, -58.3816) { // Tokyo to Buenos Aires
        commands.spawn(arc.with_color(Color::BLUE).with_segments(80).with_arc_height(80.0));
    }
    
    if let Ok(arc) = ArcLine::new(55.7558, 37.6173, -33.8688, 151.2093) { // Moscow to Sydney
        commands.spawn(arc.with_color(Color::GREEN).with_segments(70).with_arc_height(60.0));
    }
    
    if let Ok(arc) = ArcLine::new(19.4326, -99.1332, 28.6139, 77.2090) { // Mexico City to Delhi
        commands.spawn(arc.with_color(Color::ORANGE).with_segments(65).with_arc_height(45.0));
    }
}

fn spawn_austin_arc_lines(mut commands: Commands) {
    // Austin, Texas coordinates
    let austin_lat = 30.2672;
    let austin_lon = -97.7431;
    
    // All major cities from the population system with "random" arc heights
    let cities_with_heights = vec![
        ("Tokyo", 35.6762, 139.6503, 85.0),
        ("Delhi", 28.6139, 77.2090, 42.0),
        ("Shanghai", 31.2304, 121.4737, 67.0),
        ("São Paulo", -23.5505, -46.6333, 38.0),
        ("Mexico City", 19.4326, -99.1332, 15.0), // Lower since it's closer
        ("Cairo", 30.0444, 31.2357, 73.0),
        ("Mumbai", 19.0760, 72.8777, 55.0),
        ("Beijing", 39.9042, 116.4074, 91.0),
        ("Dhaka", 23.8103, 90.4125, 29.0),
        ("Osaka", 34.6937, 135.5023, 78.0),
        ("New York", 40.7128, -74.0060, 22.0), // Lower since it's closer
        ("Karachi", 24.8607, 67.0011, 46.0),
        ("Buenos Aires", -34.6037, -58.3816, 51.0),
        ("Istanbul", 41.0082, 28.9784, 64.0),
        ("Kolkata", 22.5726, 88.3639, 33.0),
        ("Lagos", 6.5244, 3.3792, 58.0),
        ("London", 51.5074, -0.1278, 82.0),
        ("Los Angeles", 34.0522, -118.2437, 8.0), // Very low since it's close
        ("Manila", 14.5995, 120.9842, 76.0),
        ("Rio de Janeiro", -22.9068, -43.1729, 44.0),
        ("Tianjin", 39.3434, 117.3616, 89.0),
        ("Kinshasa", -4.4419, 15.2663, 61.0),
        ("Paris", 48.8566, 2.3522, 75.0),
        ("Shenzhen", 22.5431, 114.0579, 68.0),
        ("Jakarta", -6.2088, 106.8456, 72.0),
        ("Bangalore", 12.9716, 77.5946, 37.0),
        ("Moscow", 55.7558, 37.6173, 94.0),
        ("Chennai", 13.0827, 80.2707, 41.0),
        ("Lima", -12.0464, -77.0428, 26.0),
        ("Bangkok", 13.7563, 100.5018, 53.0),
        ("Seoul", 37.5665, 126.9780, 83.0),
        ("Hyderabad", 17.3850, 78.4867, 35.0),
        ("Chengdu", 30.5728, 104.0668, 69.0),
        ("Singapore", 1.3521, 103.8198, 77.0),
        ("Ho Chi Minh City", 10.8231, 106.6297, 48.0),
        ("Toronto", 43.6532, -79.3832, 18.0), // Lower since it's closer
        ("Sydney", -33.8688, 151.2093, 95.0),
        ("Johannesburg", -26.2041, 28.0473, 62.0),
        ("Chicago", 41.8781, -87.6298, 12.0), // Lower since it's closer
        ("Taipei", 25.0330, 121.5654, 71.0),
    ];
    
    for (name, lat, lon, height) in cities_with_heights {
        if let Ok(arc) = ArcLine::new(lat, lon, austin_lat, austin_lon) {
            commands.spawn(arc
                .with_color(Color::CYAN)
                .with_segments(50)
                .with_arc_height(height)
            );
        }
    }
}

fn direction_lines(_time: Res<Time>, mut lines: ResMut<DebugLines>) {
    lines.line(Vec3::ZERO, Vec3::new(0.0, 50.0, 0.0), 0.0);
    lines.line_colored(Vec3::ZERO, Vec3::new(50.0, 0.0, 0.0), 0.0, Color::RED);
    lines.line_colored(Vec3::ZERO, Vec3::new(0.0, 0.0, 50.0), 0.0, Color::BLUE);
}
