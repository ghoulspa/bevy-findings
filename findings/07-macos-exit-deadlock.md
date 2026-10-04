# 7. Exiting on macOS deadlocks about a third of the time

**Versions:** 0.19.1 on macOS (Apple M6, macOS 27). Upstream issue
[bevyengine/bevy#12912](https://github.com/bevyengine/bevy/issues/12912)
(open since 2024); a fix,
[bevyengine/bevy#24059](https://github.com/bevyengine/bevy/pull/24059),
was closed unmerged in September 2026 after waiting on review.

After `AppExit`, the main thread drops `RenderAppChannels`, which blocks in
`recv_blocking` until the render thread hands the render sub-app back
(`bevy_render/src/pipelined_rendering.rs`). The render thread can be in the
middle of its last `update`, its `MultiThreadedExecutor` waiting on a task
that only the main thread may run (a `NonSend` system such as
`create_surfaces` on macOS). Neither side moves; the process has to be
killed.

`sample` of a hung process:

```
main thread:   World::clear_all -> drop RenderAppChannels
               -> async_channel RecvInner::wait -> pthread_cond_wait
render thread: SubApp::update -> MultiThreadedExecutor::run
               -> block_on -> park -> pthread_cond_wait
every worker:  idle
```

## Numbers

An application that loads a scene, waits two seconds and sends
`AppExit`, run in a loop on the Mac mini:

| build | exits | hung |
|---|---|---|
| 0.19.1 as released | 12 | 4 |
| 0.19.1 with #24059's fix | 10 | 0 |

The same application on 0.18.1 did not hang in about ten exits during
the same session, and Windows builds of 0.19.1 exited cleanly every time
(several hundred launches).

## The fix

\#24059's change, applied to 0.19.1 as is: keep a clone of the
`MainThreadExecutor` in `RenderAppChannels` and, in its `Drop`, wait with
`ComputeTaskPool::scope_with_executor(true, Some(&executor), ...)` instead
of `recv_blocking`, so the main thread keeps running main-thread tasks
while it waits.
