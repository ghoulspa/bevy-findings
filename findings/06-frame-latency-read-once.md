# 6. `desired_maximum_frame_latency` is read once

**Versions:** 0.18.1 and 0.19.1.

`extract_windows` (`bevy_render/src/view/window/mod.rs:125`) copies
`Window::desired_maximum_frame_latency` into the `ExtractedWindow` only
in the `or_insert` that creates it (`:148`). Later changes to the field
are never extracted, so the swapchain keeps the latency the window had
on its first frame. `present_mode` beside it is compared every frame and
a change reconfigures the surface (`present_mode_changed`); the frame
latency has no such path.

In practice it means the setting cannot be changed after start-up, and a
value applied in the same frame the window appears (from a config file
read in `Startup`, say) can miss the first extract and never apply.

A fix: compare and copy it every frame as `present_mode` is, and treat a
change as a reason to reconfigure the surface.
