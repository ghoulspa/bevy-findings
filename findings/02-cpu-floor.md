# 2. A higher CPU floor per frame

**Versions:** 0.19.1 against 0.18.1.

`repros/shadow-cameras` with cubes (light enough that the CPU sets the
frame) and no shadows, median frame, two launches each:

| | 0.18.1 | 0.19.1 |
|---|---|---|
| `--shape cube --shadows off` | 1.97, 1.75 ms | 2.44, 2.43 ms |

About half a millisecond a frame on a Core i5-9500. In a larger
application the gap at its lowest settings was 0.8-1.1 ms a frame, most
of it on the render thread: 0.19 runs each view's passes as a schedule of
systems, and per view that costs more than 0.18's render graph did.

Upstream already tracks this:
[bevyengine/bevy#25576](https://github.com/bevyengine/bevy/issues/25576)
(every point and spot shadow view runs a whole `Core3d` schedule) and
[bevyengine/bevy#24957](https://github.com/bevyengine/bevy/pull/24957)
(restores the parallel command buffer `finish`, milestone 0.20).

Two related costs found on the way, with what took them down:

- A `SingleThreadedExecutor` on `Core3d` (and on the schedule a light's
  shadow views run) is cheaper than the multithreaded executor there,
  since the passes encode into one command buffer in order anyway.
- With the single-threaded executor, `RenderContext` hands a system's
  command encoder back only when the schedule applies its deferred
  buffers, at the end, so every pass system records into an encoder and
  a command buffer of its own (21 command buffers a frame for two
  cameras). Handing the encoder on when the system returns (a `Drop` on
  `RenderContext`) gave one or two.
