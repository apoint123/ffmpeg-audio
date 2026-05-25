use std::{
    collections::HashMap,
    ffi::CStr,
    ptr,
    time::Duration,
};

use crate::{
    AudioCover,
    error::{
        AudioError,
        Result,
    },
    io::IoContext,
    sys,
};

pub struct Demuxer {
    ctx: *mut sys::AVFormatContext,
    packet: *mut sys::AVPacket,
    audio_stream_idx: usize,
    _io_ctx: IoContext,
}

impl Demuxer {
    pub fn new(io_ctx: IoContext) -> Result<Self> {
        unsafe {
            let mut ctx = sys::avformat_alloc_context();
            if ctx.is_null() {
                return Err(AudioError::from_ffmpeg(sys::AVERROR_ENOMEM));
            }

            (*ctx).pb = io_ctx.ctx;

            (*ctx).flags |= sys::AVFMT_FLAG_CUSTOM_IO.cast_signed();

            let ret = sys::avformat_open_input(
                &raw mut ctx,
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
            if ret < 0 {
                return Err(AudioError::from_ffmpeg(ret));
            }

            let ret = sys::avformat_find_stream_info(ctx, ptr::null_mut());
            if ret < 0 {
                sys::avformat_close_input(&raw mut ctx);
                return Err(AudioError::from_ffmpeg(ret));
            }

            let stream_idx = sys::av_find_best_stream(
                ctx,
                sys::AVMediaType_AVMEDIA_TYPE_AUDIO,
                -1,
                -1,
                ptr::null_mut(),
                0,
            );

            if stream_idx < 0 {
                sys::avformat_close_input(&raw mut ctx);
                return Err(AudioError::from_ffmpeg(stream_idx));
            }

            let packet = sys::av_packet_alloc();
            if packet.is_null() {
                sys::avformat_close_input(&raw mut ctx);
                return Err(AudioError::from_ffmpeg(sys::AVERROR_ENOMEM));
            }

            Ok(Self {
                ctx,
                packet,
                audio_stream_idx: stream_idx as usize,
                _io_ctx: io_ctx,
            })
        }
    }

    pub fn stream_codec_params(&self) -> *mut sys::AVCodecParameters {
        unsafe {
            // Safety: 我们在 new 中已经验证了索引的合法性
            let stream_ptr = *(*self.ctx).streams.add(self.audio_stream_idx);
            (*stream_ptr).codecpar
        }
    }

    pub fn read_packet(&mut self) -> Result<Option<*mut sys::AVPacket>> {
        loop {
            unsafe {
                sys::av_packet_unref(self.packet);

                let ret = sys::av_read_frame(self.ctx, self.packet);

                if ret == crate::sys::AVERROR_EOF {
                    return Ok(None);
                } else if ret < 0 {
                    return Err(AudioError::from_ffmpeg(ret));
                }

                if (*self.packet).stream_index == self.audio_stream_idx as i32 {
                    return Ok(Some(self.packet));
                }
            }
        }
    }

    pub fn seek_to(&mut self, target: Duration) -> Result<()> {
        unsafe {
            let stream_ptr = *(*self.ctx).streams.add(self.audio_stream_idx);
            let time_base = (*stream_ptr).time_base;

            let target_us = target.as_micros() as i64;

            let bq = sys::AVRational {
                num: 1,
                den: 1_000_000,
            };

            let pts = sys::av_rescale_q(target_us, bq, time_base);

            let ret = sys::avformat_seek_file(
                self.ctx,
                self.audio_stream_idx as i32,
                i64::MIN,
                pts,
                pts,
                0,
            );

            if ret < 0 {
                return Err(AudioError::from_ffmpeg(ret));
            }

            Ok(())
        }
    }

    pub fn metadata(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        unsafe {
            extract_dict((*self.ctx).metadata, &mut map);

            let stream_ptr = *(*self.ctx).streams.add(self.audio_stream_idx);
            extract_dict((*stream_ptr).metadata, &mut map);
        }
        map
    }

    pub fn cover(&self) -> Option<AudioCover> {
        unsafe {
            for i in 0..(*self.ctx).nb_streams {
                let stream_ptr = *(*self.ctx).streams.add(i as usize);
                if (*stream_ptr).codecpar.is_null() {
                    continue;
                }

                let is_video =
                    (*(*stream_ptr).codecpar).codec_type == sys::AVMediaType_AVMEDIA_TYPE_VIDEO;
                let is_attached_pic = ((*stream_ptr).disposition
                    & (sys::AV_DISPOSITION_ATTACHED_PIC.cast_signed()))
                    != 0;

                if is_video && is_attached_pic {
                    let pkt = &(*stream_ptr).attached_pic;
                    if pkt.data.is_null() || pkt.size <= 0 {
                        return None;
                    }

                    let data = std::slice::from_raw_parts(pkt.data, pkt.size as usize).to_vec();

                    let codec_id = (*(*stream_ptr).codecpar).codec_id;
                    let desc = sys::avcodec_descriptor_get(codec_id);
                    let mime_type = if !desc.is_null() && !(*desc).mime_types.is_null() {
                        let first = *(*desc).mime_types;
                        if first.is_null() {
                            None
                        } else {
                            Some(CStr::from_ptr(first).to_string_lossy().into_owned())
                        }
                    } else {
                        None
                    };

                    return Some(AudioCover { data, mime_type });
                }
            }

            None
        }
    }

    pub fn bit_rate(&self) -> i64 {
        unsafe { (*self.ctx).bit_rate }
    }

    pub fn duration(&self) -> Option<Duration> {
        unsafe {
            let stream_ptr = *(*self.ctx).streams.add(self.audio_stream_idx);
            let stream_duration = (*stream_ptr).duration;

            if stream_duration >= 0 && stream_duration != sys::AV_NOPTS_VALUE {
                let time_base = (*stream_ptr).time_base;

                let bq = sys::AVRational {
                    num: 1,
                    den: sys::AV_TIME_BASE.cast_signed(),
                };
                let duration_us = sys::av_rescale_q(stream_duration, time_base, bq);

                if duration_us >= 0 {
                    return Some(Duration::from_micros(duration_us.cast_unsigned()));
                }
            }

            let ctx_duration = (*self.ctx).duration;
            if ctx_duration >= 0 && ctx_duration != sys::AV_NOPTS_VALUE {
                return Some(Duration::from_micros(ctx_duration.cast_unsigned()));
            }

            None
        }
    }
}

unsafe fn extract_dict(dict: *mut sys::AVDictionary, map: &mut HashMap<String, String>) {
    if dict.is_null() {
        return;
    }

    let mut entry = ptr::null_mut();
    loop {
        entry = unsafe {
            sys::av_dict_get(
                dict,
                c"".as_ptr().cast::<i8>(),
                entry,
                sys::AV_DICT_IGNORE_SUFFIX.cast_signed(),
            )
        };

        if entry.is_null() {
            break;
        }

        let key = unsafe { CStr::from_ptr((*entry).key).to_string_lossy().into_owned() };
        let value = unsafe {
            CStr::from_ptr((*entry).value)
                .to_string_lossy()
                .into_owned()
        };

        map.insert(key, value);
    }
}

impl Drop for Demuxer {
    fn drop(&mut self) {
        unsafe {
            if !self.packet.is_null() {
                sys::av_packet_free(&raw mut self.packet);
            }
            if !self.ctx.is_null() {
                sys::avformat_close_input(&raw mut self.ctx);
            }
        }
    }
}
