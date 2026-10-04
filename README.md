# bevy-findings

Performance and behaviour findings in [Bevy](https://bevy.org), found while
moving an application from 0.18 to 0.19. Each one has the version, the
source location, numbers, and where possible a small self-contained
program that shows it on stock Bevy with nothing else added.

These are notes, not issues: none has been filed upstream from here,
because Bevy's [contribution policy](https://bevy.org/learn/contribute/policies/ai/)
does not accept issues written with AI, and these were found and written
up with one. If one is useful to you, the repro and the numbers are yours
to take to the tracker in your own words.

| # | Finding | Versions | Against 0.18 | Status |
|---|---|---|---|---|
| [1](findings/01-cascades-ignore-camera-layers.md) | Every camera's sun cascades draw every caster on the light's layers, whatever the camera's layers: a second camera (a viewmodel's) redraws the world's shadows | 0.19.1 | regression (0.18.1 filtered by camera) | unreported |
| [2](findings/02-cpu-floor.md) | A higher per-frame CPU floor than 0.18 in a stock scene | 0.19.1 | regression | tracked upstream |
| [3](findings/03-clustering-buffers-every-frame.md) | GPU clustering builds new buffers for every view every frame | 0.19.1 | new in 0.19, part of its slowdown | unreported |
| [4](findings/04-shadow-passes-no-gpu-timings.md) | Shadow passes report no GPU time to `RenderDiagnosticsPlugin` | 0.19.1 | regression (0.18.1 reported them) | unreported |
| [5](findings/05-diagnostics-merge-cameras.md) | A pass two cameras run reads as one camera's time in the render diagnostics | 0.18.1, 0.19.1 | not a regression (same in 0.18.1) | unreported |
| [6](findings/06-frame-latency-read-once.md) | `Window::desired_maximum_frame_latency` is read only when the window is first extracted | 0.18.1, 0.19.1 | not a regression (same in 0.18.1) | unreported |
| [7](findings/07-macos-exit-deadlock.md) | Exiting on macOS deadlocks about a third of the time (the render thread waits on a main-thread task while the main thread waits on it) | 0.19.1 | long-standing (open since 2024); not seen on 0.18.1 here | open upstream since 2024, fix PR closed unmerged |

## Test machine

Unless an entry says otherwise: GTX 1650 (4 GB), Core i5-9500, 16 GB,
Windows 11 24H2, NVIDIA 617.14, Vulkan, a 1920x1080 60 Hz display. Builds
are `--release`; two versions are timed in one sitting, launch by launch,
alternating which goes first.

## Repros

`repros/<name>/` holds one `src/main.rs` and a `Cargo.toml` per Bevy
version (`v18/`, `v19/`), so the same program builds against each:

    cd repros/shadow-cameras/v19
    cargo run --release -- --shadows hi --shape sphere --cameras 2
