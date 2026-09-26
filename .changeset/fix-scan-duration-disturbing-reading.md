---
"ffmpeg_audio": patch
---

commit: 6676910472a059443a25bfd7547c5298a9a1d516

**Fixed** `scan_exact_duration` disturbing reading: after a coarse seek it moved reading back to the start, and mid-stream it could drop samples (e.g. 16 samples in Matroska files, whose timestamps are rounded to milliseconds) or shift frame boundaries. Reading now continues with exactly the frames it would have delivered, and `stream_position` keeps its value until the next frame.
