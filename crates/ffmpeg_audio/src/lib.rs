pub mod error;
pub mod io;
pub mod log;

mod decoder;
mod demuxer;
mod resampler;

use std::{
    collections::HashMap,
    ffi::CStr,
    io::{
        Read,
        Seek,
    },
    time::Duration,
};

use decoder::Decoder;
use demuxer::Demuxer;
pub use error::{
    AudioError,
    Result,
};
pub use ffmpeg_audio_sys as sys;
use resampler::Resampler;

use crate::log::init_ffmpeg_logging;

#[derive(Debug, Clone)]
pub struct AudioCover {
    pub data: Vec<u8>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SourceAudioInfo {
    pub sample_rate: i32,
    pub channels: i32,
    pub bit_rate: i64,
    pub sample_fmt: String,
    pub codec_name: String,
    /// 原始位深：FLAC 24bit 在解码后 sample_fmt 是 s32，但 bits_per_sample 仍是 24
    pub bits_per_sample: i32,
}

pub struct AudioReader {
    resampler: Resampler,
    decoder: Decoder,
    demuxer: Demuxer,
    audio_buffer: Vec<f32>,
    is_exhausted: bool,

    source_info: SourceAudioInfo,
}

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for AudioReader {}

impl AudioReader {
    pub fn new<T>(source: T, target_sample_rate: i32, target_channels: i32) -> Result<Self>
    where
        T: Read + Seek + Send + 'static,
    {
        init_ffmpeg_logging();

        let io_ctx = io::IoContext::new(source)?;

        let demuxer = Demuxer::new(io_ctx)?;

        let codec_params = demuxer.stream_codec_params();
        let decoder = Decoder::new(codec_params)?;

        let resampler = Resampler::new(
            &decoder.channel_layout(),
            decoder.sample_fmt(),
            decoder.sample_rate(),
            target_channels,
            target_sample_rate,
            sys::AVSampleFormat_AV_SAMPLE_FMT_FLT,
        )?;

        let source_info = unsafe {
            let codec_id = (*codec_params).codec_id;
            let codec_name_ptr = sys::avcodec_get_name(codec_id);
            let codec_name = if codec_name_ptr.is_null() {
                "unknown".to_string()
            } else {
                CStr::from_ptr(codec_name_ptr)
                    .to_string_lossy()
                    .into_owned()
            };

            let src_sample_fmt = decoder.sample_fmt();
            let fmt_name_ptr = sys::av_get_sample_fmt_name(src_sample_fmt);
            let sample_fmt_str = if fmt_name_ptr.is_null() {
                "unknown".to_string()
            } else {
                CStr::from_ptr(fmt_name_ptr).to_string_lossy().into_owned()
            };

            let stream_bit_rate = (*codec_params).bit_rate;
            let bit_rate = if stream_bit_rate > 0 {
                stream_bit_rate
            } else {
                demuxer.bit_rate()
            };

            let bits_per_raw = (*codec_params).bits_per_raw_sample;
            let bits_per_coded = (*codec_params).bits_per_coded_sample;
            let bits_per_sample = if bits_per_raw > 0 {
                bits_per_raw
            } else {
                bits_per_coded
            };

            SourceAudioInfo {
                sample_rate: decoder.sample_rate(),
                channels: decoder.channels(),
                bit_rate,
                sample_fmt: sample_fmt_str,
                codec_name,
                bits_per_sample,
            }
        };

        Ok(Self {
            resampler,
            decoder,
            demuxer,
            audio_buffer: Vec::with_capacity(4096 * target_channels as usize),
            is_exhausted: false,
            source_info,
        })
    }

    #[must_use]
    pub const fn source_info(&self) -> &SourceAudioInfo {
        &self.source_info
    }

    #[must_use]
    pub fn metadata(&self) -> HashMap<String, String> {
        self.demuxer.metadata()
    }

    #[must_use]
    pub fn duration(&self) -> Option<Duration> {
        self.demuxer.duration()
    }

    #[must_use]
    pub fn cover(&self) -> Option<AudioCover> {
        self.demuxer.cover()
    }

    pub fn seek(&mut self, target: Duration) -> Result<()> {
        self.demuxer.seek_to(target)?;
        self.decoder.flush();
        self.resampler.flush()?;
        self.audio_buffer.clear();
        self.is_exhausted = false;

        Ok(())
    }

    pub fn receive_frame(&mut self) -> Result<Option<&[f32]>> {
        if self.is_exhausted {
            return Ok(None);
        }

        loop {
            match self.decoder.receive_frame() {
                Ok(Some(frame)) => {
                    self.resampler
                        .convert_and_fill(Some(frame), &mut self.audio_buffer)?;

                    if !self.audio_buffer.is_empty() {
                        return Ok(Some(&self.audio_buffer[..]));
                    }
                }

                Err(AudioError::Eagain) => match self.demuxer.read_packet()? {
                    Some(packet) => {
                        self.decoder.send_packet(packet)?;
                    }
                    None => {
                        self.decoder.send_eof_flush()?;
                    }
                },

                Ok(None) => {
                    self.resampler
                        .convert_and_fill(None, &mut self.audio_buffer)?;
                    self.is_exhausted = true;

                    if self.audio_buffer.is_empty() {
                        return Ok(None);
                    }
                    return Ok(Some(&self.audio_buffer[..]));
                }

                Err(e) => return Err(e),
            }
        }
    }
}
