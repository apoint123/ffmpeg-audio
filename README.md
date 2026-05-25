# ffmpeg-audio

> 最小化的 FFmpeg 音频解码 Rust 封装，专为"在 Rust 侧自己控制 IO"的场景设计。

## 特性

- **极简 API** —— `AudioReader::new(impl Read + Seek, sample_rate, channels)` 一行起步，内置 demux + decode + resample，输出交错 f32 stereo
- **网络透明** —— FFmpeg 永远当本地文件处理；HTTP/HTTPS 这一段在 Rust 侧用任意 `Read + Seek` 实现（如 ureq/reqwest + rustls），FFmpeg 完全不知道是网络
- **零环境依赖** —— FFmpeg 源码以 vendor zip 形式内嵌（每平台预生成的 configure 产物 + 精简 C 文件），`cargo build` 一气呵成，不需要 `FFMPEG_DIR` / `PKG_CONFIG_PATH`，也不依赖系统 FFmpeg
- **小巧** —— 仅含 `libavcodec` / `libavformat` / `libavutil` / `libswresample`；不带 `avfilter` / `avdevice` / `swscale` / 编码器 / muxer / 网络协议栈
- **音频解码齐全** —— MP3 / AAC / FLAC / Opus / Vorbis / ALAC / APE / WAV / WMA / DSD / DCA / EAC3 / TrueHD 等常见格式
- **快编译** —— 首次约 30 秒（cc 静编 ~200 个 C 文件），增量约 1 秒
- **跨平台** —— Windows / macOS (arm64 + x64) / Linux / Android / iOS 预生成配置

## 快速开始

`Cargo.toml`：

```toml
[dependencies]
ffmpeg_audio = { git = "https://github.com/apoint123/ffmpeg-audio" }
```

解码本地音频：

```rust
use std::fs::File;
use ffmpeg_audio::AudioReader;

let file = File::open("song.mp3")?;
let mut reader = AudioReader::new(file, 48000, 2)?;

println!("codec: {}", reader.source_info().codec_name);
println!("duration: {:?}", reader.duration());

while let Some(samples) = reader.receive_frame()? {
    // samples 是交错 f32 stereo (L R L R ...)
}
```

更多示例见 [`crates/ffmpeg_audio/examples/`](crates/ffmpeg_audio/examples/)。

## 网络流：把 HTTP 包成 Read + Seek

这是 ffmpeg-audio 跟其他 FFmpeg wrapper 最大的区别 —— **不内置 HTTP/HTTPS/TLS 协议**，避免静编 mbedTLS / OpenSSL 的依赖泥潭。网络这一段在 Rust 侧实现，TLS 走 rustls 即可零系统依赖。

最小思路（完整实现见 [SPlayer-Next 的 `HttpRangeSource`](https://github.com/SPlayer-Dev/SPlayer-Next/blob/main/native/audio-engine/src/http_source.rs)）：

```rust
struct HttpRangeSource {
    url: String,
    agent: ureq::Agent,
    total_size: u64,
    pos: u64,
}

impl Read for HttpRangeSource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // 发 Range 请求拉一段，按 buf 大小返回
    }
}

impl Seek for HttpRangeSource {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        // 改 self.pos，下次 read 自动从新位置发 Range
    }
}

let mut reader = AudioReader::new(HttpRangeSource::new(url)?, 48000, 2)?;
```

**取消阻塞 IO**：在 source 内部持有 `Arc<AtomicBool>` cancel flag，外部 set true 让 `read` 返回 `Interrupted`，FFmpeg 看到 IO 错误自然退出解码循环。无需 `AVIOInterruptCB`。

**预读 buffer 优化**：FFmpeg AVIO 默认 32KB 一次回调，一首 5MB 歌会触发 ~160 次 read。在 `HttpRangeSource` 内做 128KB 预读 buffer，能把 GET 数压到 ~40，rodio 缓冲完全吸收。

## 真实使用案例

[SPlayer-Next](https://github.com/SPlayer-Dev/SPlayer-Next) —— Electron + Vue 3 + Rust NAPI 桌面音乐播放器。`native/audio-engine` 用 ffmpeg-audio 同时处理：

- 本地音频文件（直接喂 `std::fs::File`）
- HTTPS 流媒体（自定义 `HttpRangeSource`，rustls 跨平台一致）
- seek / 中断 / 元数据 / 封面 / ReplayGain / 嵌入歌词

从 `ffmpeg-next` 迁移过来的收益：

| 项 | `ffmpeg-next` + 系统 FFmpeg | `ffmpeg_audio` |
|---|---|---|
| 环境依赖 | `FFMPEG_DIR` + `pkg-config` + 系统装 FFmpeg dev | 零 |
| HTTPS 流播放 | 不通（系统 FFmpeg 多数没编 mbedTLS） | 通（rustls 在 Rust 侧） |
| 增量编译 | ~10 秒 | ~1 秒 |
| CI 工作流 | 需下载预编译 FFmpeg / 设环境变量 / 安装 nasm | 直接 `cargo build` |
| 二进制运行时依赖 | 系统 FFmpeg .so / .dll | 零 |

## API 一览

```rust
// 打开 + 元信息
AudioReader::new(source, sample_rate, channels) -> Result<Self>
reader.source_info() -> &SourceAudioInfo   // codec_name / sample_rate / channels / bit_rate / sample_fmt / bits_per_sample
reader.duration() -> Option<Duration>
reader.metadata() -> HashMap<String, String>   // 容器和音频流上所有 tag
reader.cover() -> Option<AudioCover>           // attached_pic

// 解码 / seek
reader.receive_frame() -> Result<Option<&[f32]>>   // 交错 stereo f32
reader.seek(target: Duration) -> Result<()>

// 日志
log::init_ffmpeg_logging()                     // 接 ffmpeg 日志到 tracing
log::set_log_level(log::AV_LOG_FATAL)          // 覆盖默认级别
```

## 项目结构

```
ffmpeg-audio/
├── crates/
│   ├── ffmpeg_audio_sys/     底层 sys crate（裸 FFI + vendor zip）
│   │   ├── vendor/           ffmpeg_slim.zip + configs.zip
│   │   └── build.rs          解压 → cc 静编 → bindgen
│   └── ffmpeg_audio/         高层 safe wrapper
│       ├── src/              error / log / io / demuxer / decoder / resampler
│       ├── examples/         play / metadata
│       └── tests/
└── scripts/
    ├── generate_config.sh    在目标平台跑 ffmpeg ./configure，生成 build_out_xxx/
    └── extract_slim.py       从 dryrun log 提取实际需要的 .c 文件，打包 vendor zip
```

## 验证

```bash
cargo test -p ffmpeg_audio
cargo run --release --example metadata -p ffmpeg_audio -- path/to/song.flac
cargo run --release --example play -p ffmpeg_audio -- path/to/song.mp3
```

## 不在范围内

为保持最小化，以下功能**不会**纳入：视频解码、编码器、muxer、滤镜（lavfilter）、图像缩放（swscale）、硬件加速、输入/输出设备、**FFmpeg 内置网络协议（HTTP/HTTPS/TLS）**。

网络流请在 Rust 侧通过 `Read + Seek` 自行实现 —— 这是设计选择而非缺失，理由是避免 mbedTLS / OpenSSL 的静编泥潭，rustls 跨平台一致且纯静态。

## License

LGPL-2.1-or-later（继承 FFmpeg）
