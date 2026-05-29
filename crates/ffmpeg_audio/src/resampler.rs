use std::{
    mem::{
        self,
        MaybeUninit,
    },
    ptr,
};

use crate::{
    AudioFrame,
    error::{
        AudioError,
        Result,
    },
    format::AudioSample,
    sys,
};

/// Configuration options for the audio resampler.
///
/// Use the builder pattern to construct resampling parameters such as
/// target sample rate, number of channels, and audio data format.
#[derive(Debug, Clone)]
pub struct ResampleOptions {
    pub target_sample_rate: i32,
    pub target_channels: i32,
    pub target_sample_fmt: sys::AVSampleFormat,
}

impl Default for ResampleOptions {
    fn default() -> Self {
        Self {
            target_sample_rate: 44100,
            target_channels: 2,
            target_sample_fmt: sys::AVSampleFormat_AV_SAMPLE_FMT_FLT,
        }
    }
}

impl ResampleOptions {
    /// Creates a new [`ResampleOptions`] builder with default settings
    /// (44100 Hz, Stereo, 32-bit Float).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.target_sample_rate <= 0 {
            return Err(AudioError::InvalidParameter(
                "Target sample rate must be greater than 0".to_string(),
            ));
        }
        if self.target_channels <= 0 {
            return Err(AudioError::InvalidParameter(
                "Target channels must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }

    /// Sets the target sample rate (in Hz).
    #[must_use]
    pub const fn sample_rate(mut self, rate: i32) -> Self {
        self.target_sample_rate = rate;
        self
    }

    /// Sets the target number of audio channels.
    ///
    /// For example, `1` for Mono, `2` for Stereo.
    #[must_use]
    pub const fn channels(mut self, channels: i32) -> Self {
        self.target_channels = channels;
        self
    }

    /// Sets the target audio sample format.
    #[must_use]
    pub const fn format<T: AudioSample>(mut self) -> Self {
        self.target_sample_fmt = T::FORMAT;
        self
    }
}

pub struct Resampler {
    ctx: *mut sys::SwrContext,
    options: ResampleOptions,
    out_buffer: Vec<MaybeUninit<f64>>,
    output_samples: usize,
}

impl Resampler {
    pub fn new(
        in_layout: &sys::AVChannelLayout,
        in_sample_fmt: sys::AVSampleFormat,
        in_sample_rate: i32,
        options: ResampleOptions,
    ) -> Result<Self> {
        options.validate()?;

        unsafe {
            if sys::av_sample_fmt_is_planar(options.target_sample_fmt) == 1 {
                return Err(AudioError::from_ffmpeg(sys::AVERROR_INVALIDDATA));
            }

            let mut out_layout = mem::zeroed::<sys::AVChannelLayout>();
            sys::av_channel_layout_default(&raw mut out_layout, options.target_channels);

            let mut ctx = ptr::null_mut();
            let ret = sys::swr_alloc_set_opts2(
                &raw mut ctx,
                &raw const out_layout,
                options.target_sample_fmt,
                options.target_sample_rate,
                in_layout,
                in_sample_fmt,
                in_sample_rate,
                0,
                ptr::null_mut(),
            );
            crate::fferr!(ret);

            if ctx.is_null() {
                return Err(AudioError::from_ffmpeg(sys::AVERROR_ENOMEM));
            }

            let ret = sys::swr_init(ctx);
            if ret < 0 {
                sys::swr_free(&raw mut ctx);
                return Err(AudioError::from_ffmpeg(ret));
            }

            sys::av_channel_layout_uninit(&raw mut out_layout);

            Ok(Self {
                ctx,
                options,
                out_buffer: Vec::new(),
                output_samples: 0,
            })
        }
    }

    pub fn flush(&mut self) -> Result<()> {
        unsafe {
            let ret = sys::swr_init(self.ctx);
            crate::fferr!(ret);
            Ok(())
        }
    }

    /// Processes a single raw audio frame and writes the converted samples
    /// into the internal buffer.
    ///
    /// Passing `None` as the frame will flush any remaining buffered samples
    /// at the end of the stream.
    ///
    /// ## Returns
    /// - `Ok(true)` if valid data was generated and is ready to be read.
    /// - `Ok(false)` if more input frames are needed to produce an output.
    /// - `Err` if a format mismatch or FFmpeg internal error occurs.
    pub fn process<T: AudioSample>(&mut self, frame: Option<&AudioFrame<'_>>) -> Result<bool> {
        if T::FORMAT != self.options.target_sample_fmt {
            return Err(AudioError::FormatMismatch);
        }

        unsafe {
            let raw_ptr = frame.map_or(ptr::null(), AudioFrame::as_ptr);

            let (in_data, in_samples) = if raw_ptr.is_null() {
                (ptr::null(), 0)
            } else {
                (
                    (*raw_ptr).extended_data as *const *const u8,
                    (*raw_ptr).nb_samples,
                )
            };

            let expected_out_samples = sys::swr_get_out_samples(self.ctx, in_samples);
            crate::fferr!(expected_out_samples);

            if expected_out_samples <= 0 {
                self.output_samples = 0;
                return Ok(false);
            }

            let out_channels = self.options.target_channels as usize;
            let total_expected_samples = (expected_out_samples as usize) * out_channels;
            let bytes_needed = total_expected_samples * mem::size_of::<T>();

            let f64_count = bytes_needed.div_ceil(mem::size_of::<f64>());

            // Safety: `Vec::reserve` ensures `capacity >= len + additional`
            // Since we use `out_buffer` strictly as a raw FFI buffer without ever calling `.push()`,
            // its `len` is perpetually 0.
            self.out_buffer.reserve(f64_count);

            let out_ptr = self.out_buffer.as_mut_ptr().cast::<u8>();

            let actual_out_samples = sys::swr_convert(
                self.ctx,
                &raw const out_ptr,
                expected_out_samples,
                in_data,
                in_samples,
            );
            crate::fferr!(actual_out_samples);

            self.output_samples = (actual_out_samples as usize) * out_channels;

            Ok(self.output_samples > 0)
        }
    }

    /// Exposes the internally processed audio data as a typed slice.
    ///
    /// This method should only be called immediately after `process`
    /// returns `Ok(true)`. If there is no valid data, it returns an empty slice.
    #[must_use]
    pub const fn output_as<T: AudioSample>(&self) -> &[T] {
        if self.output_samples == 0 {
            return &[];
        }
        unsafe {
            std::slice::from_raw_parts(self.out_buffer.as_ptr().cast::<T>(), self.output_samples)
        }
    }
}

impl Drop for Resampler {
    fn drop(&mut self) {
        unsafe {
            if !self.ctx.is_null() {
                sys::swr_free(&raw mut self.ctx);
            }
        }
    }
}
