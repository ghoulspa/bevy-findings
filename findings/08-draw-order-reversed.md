# 8. Opaque meshes are drawn in reverse spawn order

**Versions:** 0.19.1 against 0.18.1.

Neither version sorts binned opaque draws front to back; without a depth
prepass the order they come in decides how much hidden surface the main
pass shades. 0.18.1 drew a batch set's meshes in the order they were
spawned. 0.19.1 draws them in the reverse order.

The cause: 0.19.1 queues a view's meshes from the lists in
`RenderVisibleEntities`, which are kept sorted by entity
(`bevy_render/src/camera.rs`, `iter_to_queue`), and each new bin is
appended to its batch set's draws
(`bevy_render/src/render_phase/mod.rs`, `RenderMultidrawableBatchSet::insert`)
and keeps its place while it stays visible. `Entity` orders by its bits,
which fall as the index rises (in both versions the walls below get
indices 18 to 57 on 0.18.1 and 372 to 411 on 0.19.1, and `to_bits()`
falls by one each), so the last mesh spawned sorts first. An application
that spawns near things first (a world streamed in from the player
outwards, say) gets them drawn far to near.

## Numbers

`repros/draw-order`: forty walls one behind another, each a mesh of its
own, all filling a 1920x1080 view, sixteen point lights, MSAA 4.
`--spawn near-first` spawns the nearest wall first. Median frame and
`main_opaque_pass_3d` GPU time, two launches each, RTX 4090 (Vulkan):

| | 0.18.1 frame | 0.18.1 main pass | 0.19.1 frame | 0.19.1 main pass |
|---|---|---|---|---|
| `--spawn near-first` | 2.84 ms | 0.58 ms | 7.99 ms | 4.32 ms |
| `--spawn far-first` | 7.28 ms | 4.73 ms | 3.48 ms | 0.41 ms |
| `--spawn near-first --prepass` | 2.67 ms | 0.20 ms | 3.64 ms | 0.21 ms |
| `--spawn far-first --prepass` | 2.73 ms | 0.21 ms | 3.62 ms | 0.21 ms |

On the test machine (GTX 1650, i5-9500, Vulkan), three launches each:

| | 0.18.1 frame | 0.18.1 main pass | 0.19.1 frame | 0.19.1 main pass |
|---|---|---|---|---|
| `--spawn near-first` | 4.08 ms | 3.62 ms | 44.31 ms | 43.79 ms |
| `--spawn far-first` | 125.09 ms | 120.61 ms | 6.75 ms | 6.09 ms |
| `--spawn near-first --prepass` | 4.60 ms | 3.19 ms | 6.05 ms | 3.22 ms |
| `--spawn far-first --prepass` | 7.42 ms | 3.13 ms | 6.14 ms | 3.22 ms |

The same scene, spawned the same way, costs eight to twelve times the
main pass on 0.19.1 that it did on 0.18.1, or the other way round. A depth
prepass removes the difference (the remaining gap between the versions
is finding 2's CPU floor).

## A workaround

Sort each batch set's draws near to far, by the distance from the camera
to the centre of each mesh's bounds in world space, before the main pass
(in `RenderSystems::PhaseSort`, after `sort_binned_render_phase`),
rewriting `indirect_parameters_offset_to_bin_index` and
`bin_index_to_indirect_parameters_offset_buffer` together; re-sorting
when the view has moved some distance is enough. The world centre takes
the mesh's `MeshInputUniform::world_from_local` applied to its
`MeshCullingData::aabb_center`.
`RenderMeshInstanceShared::center` looks like the obvious key but is in
the mesh's own space, as its doc says: for meshes placed by their
transform it says nothing about where they are, and a sort by it
scrambles the order instead.
