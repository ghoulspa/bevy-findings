# 1. Sun cascades ignore the camera's render layers

**Versions:** 0.19.1. 0.18.1 does not do this.

A directional light gets cascades for every active camera
(`build_directional_light_cascades`, `bevy_light/src/cascade.rs:195`). In
0.19.1 the casters drawn into a camera's cascades are chosen by the
light's render layers alone:

- `check_dir_light_mesh_visibility` culls by the light's mask
  (`bevy_light/src/lib.rs:406`, `let view_mask = maybe_view_mask...`);
- `prepare_lights` stamps each cascade view with the light's layers
  (`bevy_pbr/src/render/light.rs:1929`, `(*light_render_layers).clone()`);
- `queue_shadows` checks only those
  (`bevy_pbr/src/render/light.rs:2634`,
  `view_light_render_layers.intersects(mesh_layers)`).

0.18.1's `queue_shadows` also required the camera's layers
(`bevy_pbr/src/render/light.rs:2029-2031`, `camera_layers.intersects(mesh_layers)`).

So a second camera that draws only a small layer of its own (a
first-person viewmodel, a minimap, a render-to-texture view) gets a full
set of cascades with every shadow caster in the world in them, as long as
the light also lights that layer, which it has to for the camera's own
meshes to be lit.

## Numbers

`repros/shadow-cameras`: 3600 spheres (1280 triangles each), one
`StandardMaterial`, a sun on layers 0 and 1; `--cameras 2` adds a camera
on layer 1 alone (order 1, no clear) with one small cube. Median frame,
two launches each:

| | 0.18.1 | 0.19.1 |
|---|---|---|
| 4 cascades at 4096, one camera | 4.95, 5.06 ms | 4.58, 4.26 ms |
| 4 cascades at 4096, two cameras | 6.00, 5.48 ms | 6.89, 6.95 ms |
| cost of the second camera | about +0.7 ms | about +2.5 ms |
| 3 cascades at 2048, one camera | 4.19, 4.19 ms | 3.38, 3.39 ms |
| 3 cascades at 2048, two cameras | 5.07, 4.62 ms | 4.83, 4.81 ms |

With one camera 0.19.1 is the faster of the two; the second camera turns
that around. In a larger application with a viewmodel camera the sun's
shadow passes cost twice their GPU time on 0.19.1 (each cascade about
2x), and the frame was 1.4x-1.5x 0.18.1's at shadow-heavy settings.

The render diagnostics hide it: both cameras' cascades are named
`shadow_directional_light_0_cascade_N` and the diagnostics keep one
camera's reading (finding 5).

## A fix that worked

Stamp each camera's cascade views with the intersection of the light's
and the camera's layers in `prepare_lights`, which restores 0.18.1's
behaviour. A caster on a layer no camera draws (a first-person body that
should still cast) then needs a rule of its own; giving the cameras that
draw the default layer the light's undrawn layers as well covers it.
Separately, a camera whose meshes never take sun shadows can skip its
cascades entirely; a marker component on the camera that `prepare_lights`
reads (no cascades, and the light unshadowed for that view) took a
further 5% off.
