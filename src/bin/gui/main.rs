use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_picking::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

const CYLINDER_HEIGHT: f32 = 0.25;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins((
            DefaultPickingPlugins
                .build()
                .disable::<DefaultHighlightingPlugin>(),
            PanOrbitCameraPlugin,
        ))
        .add_systems(Startup, setup_system)
        .add_systems(Update, camera_control_ui)
        .run();
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Circular base
    commands.spawn((PbrBundle {
        mesh: meshes.add(shape::Circle::new(3.0).into()),
        material: materials.add(Color::BEIGE.with_l(0.4).into()),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    },));

    // Stones
    for stack in 0..8 {
        let angle = stack as f32 * std::f32::consts::FRAC_PI_4;
        let x = angle.cos() * 2.0;
        let y = angle.sin() * 2.0;
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cylinder {
                    height: CYLINDER_HEIGHT,
                    resolution: 64,
                    ..default()
                })),
                material: materials.add(Color::BLACK.with_a(0.2).into()),
                transform: Transform::from_xyz(x, CYLINDER_HEIGHT / 2.0, y),
                ..default()
            },
            PickableBundle::default(),
            On::<Pointer<Over>>::run(decrease_transparency),
            On::<Pointer<Out>>::run(reset_transparency),
            On::<Pointer<Click>>::run(remove_transparency),
        ));
    }

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn((
        Camera3dBundle::default(),
        PanOrbitCamera {
            beta: Some(0.4),
            radius: Some(8.0),
            beta_lower_limit: Some(0.0),
            button_orbit: MouseButton::Middle,
            button_pan: MouseButton::Middle,
            modifier_pan: Some(KeyCode::ShiftLeft),
            ..Default::default()
        },
    ));
}

fn camera_control_ui(mut camera_query: Query<&mut PanOrbitCamera>, mut ctx: EguiContexts) {
    let mut cam = camera_query.single_mut();
    egui::Window::new("Camera Controls").show(ctx.ctx_mut(), |ui| {
        if ui.button("Reset camera").clicked() {
            cam.target_focus = Vec3::ZERO;
            cam.target_alpha = 0.0;
            cam.target_beta = 0.4;
            cam.target_radius = 8.0;
            cam.force_update = true;
        }
        ui.checkbox(&mut cam.enabled, "Camera controls");
    });
}

fn remove_transparency(
    event: Listener<Pointer<Click>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    stones: Query<&Handle<StandardMaterial>>,
) {
    let material = materials
        .get_mut(stones.get(event.target).unwrap())
        .unwrap();
    material.base_color = material.base_color.with_a(1.0);
}

fn reset_transparency(
    event: Listener<Pointer<Out>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    stones: Query<&Handle<StandardMaterial>>,
) {
    let material = materials
        .get_mut(stones.get(event.target).unwrap())
        .unwrap();
    if material.base_color.a() >= 1.0 {
        return;
    }
    material.base_color = material.base_color.with_a(0.0);
}

fn decrease_transparency(
    event: Listener<Pointer<Over>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    stones: Query<&Handle<StandardMaterial>>,
) {
    let material = materials
        .get_mut(stones.get(event.target).unwrap())
        .unwrap();
    if material.base_color.a() >= 1.0 {
        return;
    }
    material.base_color = material.base_color.with_a(0.7);
}
