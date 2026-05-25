# ffmpeg-audio

Minimal FFmpeg audio decoding wrapper. Network handled by the caller via `Read + Seek`.

## Features

- `AudioReader::new(impl Read + Seek, sample_rate, channels)` → interleaved f32
- FFmpeg vendored, no env vars
- Decoders: MP3, AAC, FLAC, Opus, Vorbis, ALAC, APE, WAV, WMA, DSD, DCA, EAC3, TrueHD

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
    // process samples
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

## Not in scope

Video, encoders, muxers, filters, swscale, hardware acceleration, devices, network protocols.
