//! Audio output (cpal) and the sample queue shared with the audio decoder.
//!
//! The decoder pushes interleaved f32 chunks tagged with their pts; the device
//! callback consumes them and publishes an [`AudioClockSample`] describing
//! which pts is reaching the speaker and when. That sample drives the master
//! clock.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use anyhow::{Context as _, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub(super) struct AudioChunk {
    pub serial: u64,
    /// pts of the first sample frame in `samples`.
    pub pts: f64,
    /// Interleaved samples at the device rate/channel count.
    pub samples: Vec<f32>,
    pub offset: usize,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct AudioClockSample {
    pub serial: u64,
    /// pts of the first sample of the last delivered device buffer…
    pub pts: f64,
    /// …and the instant it is predicted to be audible.
    pub at: Instant,
    /// pts just past the last delivered sample.
    pub end_pts: f64,
}

pub(super) struct AudioQueue {
    pub chunks: VecDeque<AudioChunk>,
    /// Sample *frames* (per-channel samples) currently queued.
    pub queued_frames: usize,
    pub serial: u64,
    pub paused: bool,
    /// Output silence without consuming until the first video frame of the
    /// current serial is on screen (keeps A/V aligned after open/seek).
    pub hold: bool,
    pub gain: f32,
    pub clock: Option<AudioClockSample>,
}

pub(super) struct AudioShared {
    queue: Mutex<AudioQueue>,
    pub rate: u32,
    pub channels: u16,
}

impl AudioShared {
    pub fn lock(&self) -> MutexGuard<'_, AudioQueue> {
        // A panic while holding the lock can't leave the queue logically broken.
        self.queue.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Current audible position, if the device is actively playing `serial`.
    pub fn audible_time(&self, serial: u64) -> Option<f64> {
        let q = self.lock();
        if q.paused || q.hold {
            return None;
        }
        let c = q.clock?;
        if c.serial != serial {
            return None;
        }
        let now = Instant::now();
        // Stale = the callback hasn't delivered real samples recently (underrun / EOF).
        if now.saturating_duration_since(c.at) > Duration::from_millis(250) {
            return None;
        }
        let t = match now.checked_duration_since(c.at) {
            Some(after) => c.pts + after.as_secs_f64(),
            None => c.pts - c.at.duration_since(now).as_secs_f64(),
        };
        Some(t.min(c.end_pts))
    }
}

/// The audio output device. Opened once per app and shared by successive
/// players: (re)opening a device costs a few hundred ms of start-up silence.
/// Dropping it stops audio.
pub struct AudioDevice {
    _stream: cpal::Stream,
    pub(super) shared: Arc<AudioShared>,
}

impl AudioDevice {
    pub fn open() -> anyhow::Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow!("音声出力デバイスがありません"))?;
        let supported = device
            .default_output_config()
            .context("音声出力の設定を取得できません")?;
        let config = supported.config();
        let shared = Arc::new(AudioShared {
            queue: Mutex::new(AudioQueue {
                chunks: VecDeque::new(),
                queued_frames: 0,
                serial: 0,
                paused: true,
                hold: true,
                gain: 1.0,
                clock: None,
            }),
            rate: config.sample_rate,
            channels: config.channels,
        });

        let stream = match supported.sample_format() {
            cpal::SampleFormat::F32 => build::<f32>(&device, config, shared.clone()),
            cpal::SampleFormat::I16 => build::<i16>(&device, config, shared.clone()),
            cpal::SampleFormat::I32 => build::<i32>(&device, config, shared.clone()),
            cpal::SampleFormat::U16 => build::<u16>(&device, config, shared.clone()),
            other => Err(anyhow!("未対応のサンプル形式: {other:?}")),
        }?;
        stream.play().context("音声出力を開始できません")?;
        log::info!(
            "audio out: {} Hz, {} ch, {:?}",
            shared.rate,
            shared.channels,
            supported.sample_format()
        );
        Ok(Self {
            _stream: stream,
            shared,
        })
    }
}

fn build<T>(device: &cpal::Device, config: cpal::StreamConfig, shared: Arc<AudioShared>) -> anyhow::Result<cpal::Stream>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let mut scratch: Vec<f32> = Vec::new();
    let stream = device.build_output_stream::<T, _, _>(
        config,
        move |data: &mut [T], info: &cpal::OutputCallbackInfo| {
            scratch.clear();
            scratch.resize(data.len(), 0.0);
            let ts = info.timestamp();
            let delay = ts.playback.checked_duration_since(ts.callback).unwrap_or_default();
            fill(&shared, &mut scratch, delay);
            for (d, s) in data.iter_mut().zip(&scratch) {
                *d = T::from_sample(*s);
            }
        },
        |e| log::error!("audio stream error: {e}"),
        None,
    )?;
    Ok(stream)
}

/// Device callback body: copies queued samples into `out` (pre-zeroed).
fn fill(shared: &AudioShared, out: &mut [f32], delay: Duration) {
    let ch = usize::from(shared.channels);
    let rate = f64::from(shared.rate);
    let mut q = shared.lock();
    if q.paused || q.hold {
        return;
    }

    let gain = q.gain;
    let serial = q.serial;
    let mut written = 0;
    let mut first_pts = None;
    while written < out.len() {
        let Some(chunk) = q.chunks.front_mut() else { break };
        if chunk.serial != serial || chunk.offset >= chunk.samples.len() {
            let frames = (chunk.samples.len() - chunk.offset.min(chunk.samples.len())) / ch;
            q.chunks.pop_front();
            q.queued_frames = q.queued_frames.saturating_sub(frames);
            continue;
        }
        if first_pts.is_none() {
            first_pts = Some(chunk.pts + (chunk.offset / ch) as f64 / rate);
        }
        let n = (out.len() - written).min(chunk.samples.len() - chunk.offset);
        for (o, s) in out[written..written + n]
            .iter_mut()
            .zip(&chunk.samples[chunk.offset..chunk.offset + n])
        {
            *o = (s * gain).clamp(-1.0, 1.0);
        }
        written += n;
        chunk.offset += n;
        let done = chunk.offset >= chunk.samples.len();
        q.queued_frames = q.queued_frames.saturating_sub(n / ch);
        if done {
            q.chunks.pop_front();
        }
    }

    if let Some(pts) = first_pts {
        q.clock = Some(AudioClockSample {
            serial,
            pts,
            at: Instant::now() + delay,
            end_pts: pts + (written / ch) as f64 / rate,
        });
    }
}
