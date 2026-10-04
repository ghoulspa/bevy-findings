# 3. GPU clustering builds new buffers for every view every frame

**Versions:** 0.19.1 (GPU clustering is new in 0.19).

`prepare_clusters_for_gpu_clustering` (`bevy_pbr/src/cluster/gpu.rs:1600`)
runs every frame and, for each view, creates a fresh
`ViewClusterBindings` (`:1637`) and a fresh `ViewGpuClusteringBuffers`
(`:1661`), fills them and inserts them on the view. The new objects have
no GPU buffers yet, so writing them creates new ones: the cluster offsets
and counts, the index lists, the cluster metadata, the Z-slice list and
the scratchpad, for every camera, every frame.

In a chrome trace on a Core i5-9500 (two cameras, a daytime scene with
almost no lights) the system took 0.57 ms a frame. Keeping the components
on the view and reusing them (clear, reserve, write; the GPU buffers are
reallocated only when they grow) took it to 0.25 ms.

One catch for anyone doing the same: the clustering passes rely on the
Z-slice and scratchpad buffers starting each frame zeroed, as a newly
created buffer is. Kept buffers hold the last frame's values, and the
next frame's light lists grow on top of them: the pictures stay right
(the extra lights are out of range) but the main pass got 2.7 ms slower
in a scene with many lights. Clearing the two buffers
(`CommandEncoder::clear_buffer`) at the start of `cluster_on_gpu` fixes
it.
