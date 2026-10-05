//! Streams in unique meshes, 50 a frame, until there are `--meshes` of them
//! (2000 by default, each a 100x100-quad grid: about 0.9 GB of vertex and
//! index data), waits 5 s, and prints the process's video memory as DXGI
//! counts it (Windows), with wgpu's own count of live buffer and texture
//! memory. Built against Bevy 0.18.1 (`v18/`) and 0.19.1 (`v19/`) from this
//! one file.
//!
//!     cargo run --release -- [--meshes N] [--hint performance|memory-usage] [--out FILE]

use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::settings::{MemoryHints, RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy::window::{PresentMode, WindowResolution};
use std::fmt::Write as _;

const PER_FRAME: usize = 50;
const SETTLE: f32 = 5.0;

#[derive(Resource)]
struct Run {
    total: usize,
    spawned: usize,
    done_at: Option<f32>,
    label: String,
    out: Option<String>,
    material: Option<Handle<StandardMaterial>>,
}

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let total: usize = arg(&args, "--meshes").and_then(|n| n.parse().ok()).unwrap_or(2000);
    let hint = arg(&args, "--hint").unwrap_or_else(|| "performance".into());
    let memory_hints = if hint == "memory-usage" { MemoryHints::MemoryUsage } else { MemoryHints::Performance };
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        visible: false,
                        resolution: WindowResolution::new(1920, 1080),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    #[cfg(feature = "v19")]
                    render_creation: RenderCreation::Automatic(Box::new(WgpuSettings { memory_hints, ..default() })),
                    #[cfg(not(feature = "v19"))]
                    render_creation: RenderCreation::Automatic(WgpuSettings { memory_hints, ..default() }),
                    ..default()
                }),
        )
        .insert_resource(Run { total, spawned: 0, done_at: None, label: format!("{total} meshes, memory hint {hint}"), out: arg(&args, "--out"), material: None })
        .add_systems(Startup, setup)
        .add_systems(Update, stream)
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>, mut run: ResMut<Run>) {
    run.material = Some(materials.add(StandardMaterial::default()));
    commands.spawn((DirectionalLight::default(), Transform::from_xyz(0.0, 50.0, 0.0).looking_at(Vec3::new(10.0, 0.0, 20.0), Vec3::Y)));
    commands.spawn((Camera3d::default(), Transform::from_xyz(0.0, 400.0, 400.0).looking_at(Vec3::ZERO, Vec3::Y)));
}

fn stream(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut run: ResMut<Run>,
    time: Res<Time<Real>>,
    device: Option<Res<RenderDevice>>,
    mut exit: MessageWriter<AppExit>,
) {
    let material = run.material.clone().unwrap();
    for _ in 0..PER_FRAME.min(run.total - run.spawned) {
        let i = run.spawned;
        // Each grid is its own mesh asset, so each gets its own allocation.
        let grid = meshes.add(Plane3d::default().mesh().size(10.0, 10.0).subdivisions(99));
        let side = (run.total as f32).sqrt().ceil() as usize;
        let (x, z) = ((i % side) as f32 * 12.0 - side as f32 * 6.0, (i / side) as f32 * 12.0 - side as f32 * 6.0);
        commands.spawn((Mesh3d(grid), MeshMaterial3d(material.clone()), Transform::from_xyz(x, 0.0, z)));
        run.spawned += 1;
    }
    if run.spawned < run.total {
        return;
    }
    let now = time.elapsed_secs();
    let done_at = *run.done_at.get_or_insert(now);
    if now - done_at < SETTLE {
        return;
    }
    let version = if cfg!(feature = "v19") { "0.19.1" } else { "0.18.1" };
    let mut out = String::new();
    let _ = writeln!(out, "bevy {version}: {}", run.label);
    match dxgi_usage() {
        Some((local, shared)) => {
            let _ = writeln!(out, "process video memory (DXGI): {local} MB dedicated, {shared} MB shared");
        }
        None => {
            let _ = writeln!(out, "process video memory (DXGI): not available");
        }
    }
    if let Some(device) = device {
        let c = device.wgpu_device().get_internal_counters();
        let mb = |v: isize| v as f64 / (1024.0 * 1024.0);
        let _ = writeln!(
            out,
            "wgpu live: {} buffers {:.0} MB, {} textures {:.0} MB (with the `counters` feature; zero without)",
            c.hal.buffers.read(),
            mb(c.hal.buffer_memory.read()),
            c.hal.textures.read(),
            mb(c.hal.texture_memory.read())
        );
    }
    print!("{out}");
    if let Some(path) = &run.out {
        let _ = std::fs::write(path, &out);
    }
    exit.write(AppExit::Success);
}

/// The process's video memory on the adapter where it uses the most, in MB.
#[cfg(windows)]
fn dxgi_usage() -> Option<(u64, u64)> {
    use windows::core::Interface;
    use windows::Win32::Graphics::Dxgi::*;
    unsafe {
        let factory: IDXGIFactory1 = CreateDXGIFactory1().ok()?;
        let mut best: Option<(u64, u64)> = None;
        let mut i = 0;
        while let Ok(a) = factory.EnumAdapters1(i) {
            i += 1;
            let Ok(a3) = a.cast::<IDXGIAdapter3>() else { continue };
            let mut l = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
            let mut n = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
            if a3.QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, &mut l).is_ok()
                && a3.QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL, &mut n).is_ok()
                && l.CurrentUsage > best.map_or(0, |(x, _)| x)
            {
                best = Some((l.CurrentUsage, n.CurrentUsage));
            }
        }
        best.map(|(l, n)| (l >> 20, n >> 20))
    }
}

#[cfg(not(windows))]
fn dxgi_usage() -> Option<(u64, u64)> {
    None
}
