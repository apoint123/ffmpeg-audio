# Changelog

All notable changes to `ffmpeg_audio` and `ffmpeg_audio_sys` will be documented in this file.

## [Unreleased]

<!-- Changes not yet released go here -->

### ffmpeg_audio

#### Breaking Changes

- **Removed** the public `TimeBase` type and the `core::time` module. Timestamp handling is now internal to the crate.
- **Changed** `scan_exact_duration` to return the exact duration measured from zero: the time right after the last sample, rather than the span between the earliest and the latest timestamp found while scanning.

#### Changed

- **Changed** timestamps and durations (`AudioFrame::pts`, `AudioFrame::duration`, `stream_position`, `scan_exact_duration`) to be computed exactly and rounded down to whole nanoseconds, instead of being rounded to microseconds at every step. Seeking to a reported timestamp now lands on the same sample again.
- **Documented** that `SeekMode::Coarse` and `SeekMode::Accurate` start at the nearest reachable position when the container cannot reach positions before the target, and that `AudioReader::duration` returns the duration declared by the container, which is an estimate.

#### Fixed

- **Fixed** accurate seeking trimming one sample too many when a frame timestamp is not a whole number of microseconds (e.g. seeking to 100 ms in a 48 kHz AAC stream now starts exactly at 100 ms instead of 100.02 ms).
- **Fixed** accurate seeking discarding the preroll trim of the frame it lands on, which could deliver preroll samples when seeking close to the start of a stream with negative timestamps.
- **Fixed** `scan_exact_duration` under-reporting the duration of streams whose first samples cannot be reached by seeking (e.g. 892.666 ms instead of 899.667 ms for a Matroska file with negative timestamps), and reporting a 2 s WAV file as 1.999999 s.

### ffmpeg_audio_sys

#### Breaking Changes

- **Updated** bundled FFmpeg from 8.1.2 to 9.0.2 (libavcodec 63, libavformat 63, libavutil 61, libswresample 7). APIs removed upstream in this major release (e.g. `AVCodecContext.properties`, the private fields of `AVCodecParser`, `AVTimebaseSource`) are no longer available in the generated bindings.

#### Fixed

- **Fixed** the build script reusing FFmpeg sources extracted from a previous version of the bundled archives, so incremental builds could silently keep compiling the old FFmpeg after the archives were updated.

---

## [0.3.1] - 2026-08-05

### ffmpeg_audio

#### Refactored

- **Refactored** HTTP reconnection handling to automatically reconnect when the server closes the connection unexpectedly, using bounded exponential backoff.
- **Refactored** HTTP range request validation to verify the `Content-Range` start offset and total length for the initial request and subsequent seeks or reconnects.

---

## [0.3.0] - 2026-08-05

### ffmpeg_audio

#### Breaking Changes

- **Changed** HTTP cancellation to use the new thread-safe, opaque `HttpCancelHandle` API.
- **Removed** `HttpAudioSource::new_with_token`; use `HttpAudioSource::new_with_cancel_handle` instead.

#### Added

- **Added** `HttpCancelHandle::cancel`, `reset`, and `is_cancelled` for controlling cancellation and reusing an HTTP audio source.

#### Fixed

- **Fixed** cancellation behavior so ongoing network operations can be interrupted immediately and subsequent operations can continue after resetting the handle.

### ffmpeg_audio_sys

- No functional changes in this release.

---

## [0.2.0] - 2026-07-21

### ffmpeg_audio

#### Breaking Changes

- **Changed** log feature to be disabled by default.

#### Added

- **Added** API to get raw PCM data directly, bypassing the resampler.

#### Refactored

- **Refactored** HTTP stream implementation using `tokio` and `reqwest` to support cancellation at any point.
- **Refactored** negative PTS handling to be unified across the codebase.
- **Refactored** stream scanning to skip irrelevant streams.
- **Refactored** cover stream scanning to continue after encountering invalid streams.
- **Refactored** duration scanning for more precise results.
- **Refactored** seeking for more precise position accuracy.
- **Refactored** added more defensive code paths.

#### Fixed

- **Fixed** unified audio timeline and hardened resampling safety boundaries.

### ffmpeg_audio_sys

- No functional changes in this release.

---

## [0.1.2] - 2026-07-14

### ffmpeg_audio

- **Added** `Send` implementation for `ChannelLayout`, enabling it to be safely transferred across threads.
- **Added** `Sync` implementation for `ChannelLayout`, enabling shared references across threads.
- **Added** `Send` implementation for `Resampler`, enabling it to be safely transferred across threads.

### ffmpeg_audio_sys

- No functional changes in this release (version bump only).

---

## [0.1.1] - 2026-07-14

### ffmpeg_audio

- **Changed** package description from "High-level Rust audio processing, decoding, and resampling engine based on FFmpeg." to "A lightweight FFmpeg audio decoding wrapper designed for music player applications."

### ffmpeg_audio_sys

- **Changed** package description from "Raw FFI bindings for FFmpeg audio processing." to "Raw FFmpeg FFI bindings for ffmpeg_audio."

---

## [0.1.0] - 2026-07-14

### ffmpeg_audio

- Initial release of the high-level audio decoding and resampling crate built on top of `ffmpeg_audio_sys`.

### ffmpeg_audio_sys

- Initial release of raw FFmpeg FFI bindings.

[unreleased]: https://github.com/apoint123/ffmpeg-audio/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/apoint123/ffmpeg-audio/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/apoint123/ffmpeg-audio/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/apoint123/ffmpeg-audio/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/apoint123/ffmpeg-audio/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/apoint123/ffmpeg-audio/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/apoint123/ffmpeg-audio/releases/tag/v0.1.0
