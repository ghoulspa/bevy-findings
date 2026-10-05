//! Forty walls one behind another, each its own mesh, all filling the view,
//! timed in a hidden 1920x1080 window with vsync off. Built against Bevy
//! 0.18.1 (`v18/`) and 0.19.1 (`v19/`) from this one file.
//!
//!     cargo run --release -- [--spawn far-first|near-first] [--prepass] [--out FILE]
//!
//! `--spawn` is the order the walls are spawned in; `--prepass` gives the
//! camera a `DepthPrepass`. Sixteen point lights (no shadows) in front of
//! the walls give each fragment some work. After 5 s of warm-up it measures
//! 10 s of frames and prints the median, mean and 99th percentile frame time
//! and the `RenderDiagnosticsPlugin` GPU timings.

use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::diagnostic::RenderDiagnosticsPlugin;
use bevy::window::{PresentMode, WindowResolution};
use std::fmt::Write as _;
use std::time::Instant;

const WARMUP: f32 = 5.0;
const MEASURE: f32 = 10.0;
const WALLS: usize = 40;

#[derive(Resource)]
struct Run {
    label: String,
    far_first: bool,
    prepass: bool,
    out: Option<String>,
    start: Option<Instant>,
    frames: Vec<f32>,
}

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let spawn = arg(&args, "--spawn").unwrap_or_else(|| "far-first".into());
    let prepass = args.iter().any(|a| a == "--prepass");
    let label = format!("spawned {spawn}, depth prepass {}", if prepass { "on" } else { "off" });
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
        .insert_resource(Run { label, far_first: spawn == "far-first", prepass, out: arg(&args, "--out"), start: None, frames: Vec::new() })
        .add_systems(Startup, setup)
        .add_systems(Update, measure)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>, run: Res<Run>) {
    let material = materials.add(StandardMaterial { base_color: Color::srgb(0.8, 0.7, 0.6), perceptual_roughness: 0.6, ..default() });
    let mut order: Vec<usize> = (0..WALLS).collect();
    if run.far_first {
        order.reverse();
    }
    for i in order {
        // Each wall is a mesh of its own, so each is a bin and a draw of its own.
        let distance = 6.0 + 2.0 * i as f32;
        let wall = meshes.add(Cuboid::new(1.8 * distance, 1.0 * distance, 0.2));
        commands.spawn((Mesh3d(wall), MeshMaterial3d(material.clone()), Transform::from_xyz(0.0, 0.0, -distance)));
    }
    for i in 0..16 {
        let (x, y) = ((i % 4) as f32 * 2.0 - 3.0, (i / 4) as f32 * 1.2 - 1.8);
        commands.spawn((PointLight { intensity: 40_000.0, range: 60.0, ..default() }, Transform::from_xyz(x, y, -4.0)));
    }
    let camera = commands.spawn((Camera3d::default(), Transform::default())).id();
    if run.prepass {
        commands.entity(camera).insert(DepthPrepass);
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
