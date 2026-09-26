---
"ffmpeg_audio": minor
---

commit: 6d7e7dcce4073ba62176049d7e88d3514dc24919

**Changed** `scan_exact_duration` to return the exact duration measured from zero: the time right after the last sample, rather than the span between the earliest and the latest timestamp found while scanning.
