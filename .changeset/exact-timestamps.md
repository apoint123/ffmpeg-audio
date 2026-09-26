---
"ffmpeg_audio": patch
---

commit: 6d7e7dcce4073ba62176049d7e88d3514dc24919

**Changed** timestamps and durations (`AudioFrame::pts`, `AudioFrame::duration`, `stream_position`, `scan_exact_duration`) to be computed exactly and rounded down to whole nanoseconds, instead of being rounded to microseconds at every step. Seeking to a reported timestamp now lands on the same sample again.
