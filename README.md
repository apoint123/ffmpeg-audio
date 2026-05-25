# ffmpeg-audio

Minimal Rust wrapper around FFmpeg for audio decoding. Network/TLS handled by the caller via `Read + Seek`.

## Features

- Tiny API: `AudioReader::new(impl Read + Seek, sample_rate, channels)` returns interleaved f32 stereo
- Zero environment dependency — FFmpeg is vendored; no `FFMPEG_DIR` or system FFmpeg required
- No built-in HTTP/HTTPS/TLS — wrap your network source as `Read + Seek` in Rust (e.g. ureq + rustls)
- Decoders: MP3, AAC, FLAC, Opus, Vorbis, ALAC, APE, WAV, WMA, DSD, DCA, EAC3, TrueHD
- Cross-platform: Windows / macOS (arm64 + x64) / Linux / Android / iOS

## Usage

```toml
[dependencies]
ffmpeg_audio = { git = "https://github.com/apoint123/ffmpeg-audio" }
```

```rust
use std::fs::File;
use ffmpeg_audio::AudioReader;

let mut reader = AudioReader::new(File::open("song.mp3")?, 48000, 2)?;
while let Some(samples) = reader.receive_frame()? {
    // interleaved f32 stereo (L R L R ...)
}
```

## HTTP streams

Implement `Read + Seek` over HTTP Range and pass it in:

```rust
struct HttpRangeSource { /* ureq agent + cursor */ }
impl Read for HttpRangeSource { /* ... */ }
impl Seek for HttpRangeSource { /* ... */ }

let mut reader = AudioReader::new(HttpRangeSource::new(url)?, 48000, 2)?;
```

## API

```rust
AudioReader::new(source, sample_rate, channels) -> Result<Self>
reader.source_info() -> &SourceAudioInfo
reader.duration() -> Option<Duration>
reader.metadata() -> HashMap<String, String>
reader.cover() -> Option<AudioCover>
reader.receive_frame() -> Result<Option<&[f32]>>
reader.seek(target: Duration) -> Result<()>

log::init_ffmpeg_logging()
log::set_log_level(log::AV_LOG_FATAL)
```

## Not in scope

Video, encoders, muxers, filters, swscale, hardware acceleration, devices, FFmpeg's built-in network protocols.

## License

LGPL-2.1-or-later
