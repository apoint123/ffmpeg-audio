---
"ffmpeg_audio_sys": minor
"ffmpeg_audio": minor
---

pr: 13

**Updated** bundled FFmpeg from 8.1.2 to 9.0.2 (libavcodec 63, libavformat 63, libavutil 61, libswresample 7). APIs removed upstream in this major release (e.g. `AVCodecContext.properties`, the private fields of `AVCodecParser`, `AVTimebaseSource`) are no longer available in the generated bindings.
