# 9. More video memory for the same data on Vulkan

**Versions:** 0.19.1 (wgpu 29) against 0.18.1 (wgpu 27).

wgpu 29's Vulkan backend allocates device memory through `gpu-allocator`
where wgpu 27 used `gpu-alloc`. With Bevy's default
`WgpuSettings::memory_hints`, `MemoryHints::Performance`, wgpu asks it for
blocks of 128 to 256 MB (`wgpu-hal/src/lib.rs`,
`AllocationSizes::from_memory_hints`). An application whose buffers grow
and are replaced as meshes stream in (Bevy's mesh allocator grows its
slabs by 1.5x) ends up holding a good deal more video memory than it
uses: the process's dedicated video memory rises while wgpu's own count
of live buffers and textures stays the same.

## Numbers

`repros/video-memory`: 2000 unique 100x100-quad grid meshes streamed in
50 a frame, then 5 s of rest. Process video memory as DXGI reports it,
and wgpu's live buffer memory (its `counters` feature), RTX 4090, Vulkan:

| | dedicated | wgpu live buffers |
|---|---|---|
| 0.18.1, `--hint performance` (default) | 1441 MB | 1159 MB |
| 0.18.1, `--hint memory-usage` | 1378 MB | 1159 MB |
| 0.19.1, `--hint performance` (default) | 1821 MB | 1164 MB |
| 0.19.1, `--hint memory-usage` | 1550 to 1614 MB | 1164 MB |

In a larger application with about 1.8 GB of live buffers, 0.18.1 held
2.2 GB and 0.19.1 3.1 to 3.3 GB. On a 4 GB card (GTX 1650) the same view
drew in 9 ms in one launch and 12.6 or 17.3 ms in others, with the same
triangles drawn; in one sitting, two views took 1.76x and 1.62x their
0.18.1 time. `MemoryHints::MemoryUsage` (8 to 64 MB blocks) held 2.18 GB
and brought those views to 1.01x. Where the frame is CPU-bound,
`Performance` was 1 to 2% quicker than `MemoryUsage`.

## Workaround

```rust
RenderPlugin {
    render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
        memory_hints: MemoryHints::MemoryUsage,
        ..default()
    })),
    ..default()
}
```
