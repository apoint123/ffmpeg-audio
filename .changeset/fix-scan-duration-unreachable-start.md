---
"ffmpeg_audio": patch
---

commit: 6d7e7dcce4073ba62176049d7e88d3514dc24919

**Fixed** `scan_exact_duration` under-reporting the duration of streams whose first samples cannot be reached by seeking (e.g. 892.666 ms instead of 899.667 ms for a Matroska file with negative timestamps), and reporting a 2 s WAV file as 1.999999 s.
