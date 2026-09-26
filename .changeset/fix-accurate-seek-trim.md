---
"ffmpeg_audio": patch
---

commit: 6d7e7dcce4073ba62176049d7e88d3514dc24919

**Fixed** accurate seeking trimming one sample too many when a frame timestamp is not a whole number of microseconds (e.g. seeking to 100 ms in a 48 kHz AAC stream now starts exactly at 100 ms instead of 100.02 ms).
