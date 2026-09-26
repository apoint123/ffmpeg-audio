---
"ffmpeg_audio_sys": patch
---

commit: f31bdec485f83121e907b0beb686fa6222041f61

**Fixed** the build script reusing FFmpeg sources extracted from a previous version of the bundled archives, so incremental builds could silently keep compiling the old FFmpeg after the archives were updated.
