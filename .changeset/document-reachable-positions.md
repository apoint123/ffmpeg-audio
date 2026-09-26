---
"ffmpeg_audio": patch
---

commit: 6d7e7dcce4073ba62176049d7e88d3514dc24919

**Documented** that `SeekMode::Coarse` and `SeekMode::Accurate` start at the nearest reachable position when the container cannot reach positions before the target, and that `AudioReader::duration` returns the duration declared by the container, which is an estimate.
