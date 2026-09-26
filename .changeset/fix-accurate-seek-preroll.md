---
"ffmpeg_audio": patch
---

commit: 6d7e7dcce4073ba62176049d7e88d3514dc24919

**Fixed** accurate seeking discarding the preroll trim of the frame it lands on, which could deliver preroll samples when seeking close to the start of a stream with negative timestamps.
