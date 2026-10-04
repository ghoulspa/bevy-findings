//! A field of shapes under a cascaded sun, timed in a hidden 1920x1080
//! window with vsync off. Built against Bevy 0.18.1 (`v18/`) and 0.19.1
//! (`v19/`) from this one file.
//!
//!     cargo run --release -- [--shadows hi|med|off] [--shape cube|sphere] [--cameras 1|2] [--out FILE]
//!
//! `hi` is four cascades at 4096, `med` three at 2048. `--cameras 2` adds a
//! second camera drawing only render layer 1 over the first (a first-person
//! viewmodel's setup), with one small cube on that layer; the sun lights
//! layers 0 and 1. After 5 s of warm-up it measures 10 s of frames and
//! prints the median, mean and 99th percentile frame time and the
//! `RenderDiagnosticsPlugin` timings.

use bevy::camera::visibility::RenderLayers;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::*;
use bevy::render::diagnostic::RenderDiagnosticsPlugin;
use bevy::window::{PresentMode, WindowResolution};
use std::fmt::Write as _;
use std::time::Instant;

const WARMUP: f32 = 5.0;
const MEASURE: f32 = 10.0;

#[derive(Resource)]
struct Run {
    label: String,
    shadows: bool,
    cascades: usize,
    spheres: bool,
    cameras: u32,
    out: Option<String>,
    start: Option<Instant>,
    frames: Vec<f32>,
}

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let shadows = arg(&args, "--shadows").unwrap_or_else(|| "hi".into());
    let shape = arg(&args, "--shape").unwrap_or_else(|| "sphere".into());
    let cameras: u32 = arg(&args, "--cameras").and_then(|c| c.parse().ok()).unwrap_or(1);
    let (on, cascades, res) = match shadows.as_str() {
        "off" => (false, 1, 512),
        "med" => (true, 3, 2048),
        _ => (true, 4, 4096),
    };
    let label = format!("shadows {shadows}, {shape}s, {cameras} camera(s)");
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                visible: false,
                resolution: WindowResolution::new(1920, 1080),
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((FrameTimeDiagnosticsPlugin::default(), RenderDiagnosticsPlugin))
        .insert_resource(DirectionalLightShadowMap { size: res })
        .insert_resource(Run {
            label,
            shadows: on,
            cascades,
            spheres: shape == "sphere",
            cameras,
            out: arg(&args, "--out"),
            start: None,
            frames: Vec::new(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, measure)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, run: Res<Run>) {
    let material = materials.add(StandardMaterial { base_color: Color::srgb(0.8, 0.7, 0.6), perceptual_roughness: 0.9, ..default() });
    // 1280 triangles a sphere makes the frame GPU-bound on a mid-range card.
    let shape = if run.spheres { meshes.add(Sphere::new(0.6).mesh().ico(3).unwrap()) } else { meshes.add(Cuboid::new(1.0, 1.0, 1.0)) };
    commands.spawn((Mesh3d(meshes.add(Plane3d::default().mesh().size(600.0, 600.0))), MeshMaterial3d(material.clone())));
    let mut seed = 12345u32;
    let mut rand = || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) as f32 / (1 << 24) as f32
    };
    for i in 0..60 {
        for j in 0..60 {
            let (w, h, d) = (1.0 + 2.0 * rand(), 1.0 + 11.0 * rand(), 1.0 + 2.0 * rand());
            let (x, z) = (i as f32 * 5.0 - 150.0, j as f32 * 5.0 - 150.0);
            commands.spawn((Mesh3d(shape.clone()), MeshMaterial3d(material.clone()), Transform::from_xyz(x, h / 2.0, z).with_scale(Vec3::new(w, h, d))));
        }
    }
    commands.spawn((
        DirectionalLight {
            illuminance: 9000.0,
            #[cfg(feature = "v19")]
            shadow_maps_enabled: run.shadows,
            #[cfg(not(feature = "v19"))]
            shadows_enabled: run.shadows,
            ..default()
        },
        Transform::from_xyz(0.0, 100.0, 0.0).looking_at(Vec3::new(40.0, 0.0, 70.0), Vec3::Y),
        CascadeShadowConfigBuilder { num_cascades: run.cascades, maximum_distance: 250.0, ..default() }.build(),
        RenderLayers::from_layers(&[0, 1]),
    ));
    let eye = Transform::from_xyz(0.0, 25.0, 170.0).looking_at(Vec3::new(0.0, 0.0, 60.0), Vec3::Y);
    commands.spawn((Camera3d::default(), eye));
    if run.cameras >= 2 {
        commands.spawn((
            Camera3d::default(),
            Camera { order: 1, clear_color: ClearColorConfig::None, ..default() },
            eye,
            RenderLayers::layer(1),
        ));
        commands.spawn((Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 2.0))), MeshMaterial3d(material), Transform::from_xyz(0.6, 24.5, 168.5), RenderLayers::layer(1)));
    }
}

fn measure(time: Res<Time<Real>>, mut run: ResMut<Run>, diagnostics: Res<DiagnosticsStore>, adapter: Option<Res<bevy::render::renderer::RenderAdapterInfo>>, mut exit: MessageWriter<AppExit>) {
    if time.elapsed_secs() < WARMUP {
        return;
    }
    let start = *run.start.get_or_insert_with(Instant::now);
    run.frames.push(time.delta_secs() * 1000.0);
    if start.elapsed().as_secs_f32() < MEASURE {
        return;
    }
    let mut f = run.frames.clone();
    f.sort_by(f32::total_cmp);
    let version = if cfg!(feature = "v19") { "0.19.1" } else { "0.18.1" };
    let mut out = String::new();
    let _ = writeln!(out, "bevy {version}: {}", run.label);
    if let Some(a) = adapter {
        let _ = writeln!(out, "adapter {} via {:?}", a.name, a.backend);
    }
    let mean = f.iter().sum::<f32>() / f.len() as f32;
    let _ = writeln!(out, "{} frames: median {:.3} ms, mean {mean:.3} ms, p99 {:.3} ms", f.len(), f[f.len() / 2], f[f.len() * 99 / 100]);
    let mut rows: Vec<(f64, String)> = diagnostics
        .iter()
        .filter(|d| d.path().as_str().starts_with("render/") && d.path().as_str().ends_with("elapsed_gpu"))
        .filter_map(|d| d.average().map(|v| (v, format!("{v:9.3} ms  {}", d.path().as_str()))))
        .collect();
    rows.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (_, r) in rows {
        let _ = writeln!(out, "{r}");
    }
    print!("{out}");
    if let Some(path) = &run.out {
        let _ = std::fs::write(path, &out);
    }
    exit.write(AppExit::Success);
}
