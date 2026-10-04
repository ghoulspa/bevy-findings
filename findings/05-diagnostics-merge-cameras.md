# 5. A pass two cameras run reads as one camera's time

**Versions:** 0.18.1 and 0.19.1.

Each camera runs its own `main_opaque_pass_3d`, tonemapping and so on,
and records its diagnostics span under the same name. The render
diagnostics keep one measurement per name per frame, and the smoothing
weighs a measurement by the time since the previous one, so with two
cameras the reading is whichever camera reported first (or a mix), not
their sum.

`repros/shadow-cameras --shadows hi --shape sphere`, the
`main_opaque_pass_3d` GPU time:

| | one camera | two cameras |
|---|---|---|
| 0.18.1 | 2.23, 2.71 ms | 1.42, 1.21 ms |
| 0.19.1 | 1.91, 1.68 ms | 0.90, 0.90 ms |

The second camera draws one small cube, so the true sum is a little
above the one-camera figure; the reading halves instead. The same goes
for every pass both cameras run, the shadow cascades included (each
camera has its own set under the same names).
