# 4. Shadow passes report no GPU time to `RenderDiagnosticsPlugin`

**Versions:** 0.19.1.

With `RenderDiagnosticsPlugin`, 0.18.1 reports a GPU time per shadow
view (`render/shadow_directional_light_0_cascade_0/elapsed_gpu` and so
on, and `shadow_point_light_N_±axis` for point lights). 0.19.1 reports
none: the shadow pass opens a tracing span
(`bevy_pbr/src/render/light.rs:2880`) but records no diagnostics span, so
the shadow maps' GPU time is missing from the totals.

`repros/shadow-cameras --shadows hi`: 0.18.1 lists four
`shadow_directional_light_0_cascade_N` rows (0.12 to 0.72 ms of GPU each
for the spheres); 0.19.1 lists `main_opaque_pass_3d`, `clustering`,
`upscaling`, `early_mesh_preprocessing`, `bin_unpacking` and no shadow
rows at all, with the same shadows on screen.

This is what made finding 1 hard to see: the cost was there in the frame
time but not in any pass.
