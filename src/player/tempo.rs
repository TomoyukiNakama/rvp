//! Pitch-preserving tempo change (libavfilter `atempo`) on interleaved f32.
//!
//! `atempo` stamps its output in the *output* time domain, so media-time pts
//! are derived here instead: output sample `k` after a reset corresponds to
//! media time `base + k · speed / rate`.

use ffmpeg_next as ffmpeg;
use ffmpeg::ChannelLayout;
use ffmpeg::format::{Sample, sample::Type};
use ffmpeg::{filter, frame};

pub struct Tempo {
    graph: filter::Graph,
    speed: f32,
    rate: u32,
    channels: usize,
    /// Media time of the first input sample (set on first `process`).
    base: Option<f64>,
    /// Output sample frames emitted so far.
    emitted: u64,
}

// SAFETY: the graph is owned and used by a single (decode) thread at a time.
unsafe impl Send for Tempo {}

/// `atempo` accepts 0.5..=100 per instance; chain instances for slower rates.
fn atempo_chain(speed: f32) -> String {
    let mut parts = Vec::new();
    let mut s = f64::from(speed);
    while s < 0.5 {
        parts.push("atempo=0.5".to_owned());
        s /= 0.5;
    }
    parts.push(format!("atempo={s:.6}"));
    parts.join(",")
}

fn layout_name(channels: usize) -> &'static str {
    match channels {
        1 => "mono",
        2 => "stereo",
        3 => "2.1",
        4 => "quad",
        5 => "5.0",
        6 => "5.1",
        7 => "6.1",
        8 => "7.1",
        _ => "stereo",
    }
}

/// A chunk of tempo-adjusted output.
pub struct TempoOut {
    /// Media time of the first sample.
    pub pts: f64,
    pub samples: Vec<f32>,
}

impl Tempo {
    pub fn new(speed: f32, rate: u32, channels: usize) -> Result<Self, ffmpeg::Error> {
        let layout = layout_name(channels);
        let mut graph = filter::Graph::new();
        let abuffer = filter::find("abuffer").ok_or(ffmpeg::Error::FilterNotFound)?;
        let sink = filter::find("abuffersink").ok_or(ffmpeg::Error::FilterNotFound)?;
        graph.add(
            &abuffer,
            "in",
            &format!("time_base=1/{rate}:sample_rate={rate}:sample_fmt=flt:channel_layout={layout}"),
        )?;
        graph.add(&sink, "out", "")?;
        let spec = format!("{},aformat=sample_fmts=flt:channel_layouts={layout}", atempo_chain(speed));
        graph.output("in", 0)?.input("out", 0)?.parse(&spec)?;
        graph.validate()?;
        Ok(Self { graph, speed, rate, channels, base: None, emitted: 0 })
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// Feeds interleaved samples starting at media time `pts`; returns any output ready.
    pub fn process(&mut self, pts: f64, samples: &[f32]) -> Result<Vec<TempoOut>, ffmpeg::Error> {
        let n = samples.len() / self.channels;
        self.base.get_or_insert(pts);
        if n > 0 {
            let mut f = frame::Audio::new(Sample::F32(Type::Packed), n, ChannelLayout::default(self.channels as i32));
            f.set_rate(self.rate);
            f.set_pts(Some((pts * f64::from(self.rate)).round() as i64));
            let dst = f.data_mut(0);
            dst[..samples.len() * 4].copy_from_slice(bytemuck::cast_slice(samples));
            self.graph.get("in").expect("abuffer").source().add(&f)?;
        }
        self.drain()
    }

    /// Signals end of stream and returns the remaining output.
    pub fn finish(&mut self) -> Result<Vec<TempoOut>, ffmpeg::Error> {
        self.graph.get("in").expect("abuffer").source().flush()?;
        self.drain()
    }

    fn drain(&mut self) -> Result<Vec<TempoOut>, ffmpeg::Error> {
        let rate = f64::from(self.rate);
        let ch = self.channels;
        let mut out = Vec::new();
        // Both contexts are created in `new`, so `get` can't fail.
        let mut ctx = self.graph.get("out").expect("abuffersink");
        let mut sink = ctx.sink();
        loop {
            let mut f = frame::Audio::empty();
            match sink.frame(&mut f) {
                Ok(()) => {
                    let n = f.samples() * ch;
                    let bytes = &f.data(0)[..n * 4];
                    let samples: Vec<f32> = bytemuck::pod_collect_to_vec(bytes);
                    let base = self.base.unwrap_or(0.0);
                    let pts = base + self.emitted as f64 * f64::from(self.speed) / rate;
                    self.emitted += (samples.len() / ch) as u64;
                    out.push(TempoOut { pts, samples });
                }
                Err(ffmpeg::Error::Other { errno }) if errno == ffmpeg::error::EAGAIN => break,
                Err(ffmpeg::Error::Eof) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_for_slow_speeds() {
        assert_eq!(atempo_chain(1.5), "atempo=1.500000");
        assert_eq!(atempo_chain(0.25), "atempo=0.5,atempo=0.500000");
    }

    /// Feeding N seconds at speed s yields ≈ N/s seconds, with media-time pts.
    #[test]
    fn duration_scales_and_pts_stay_in_media_time() {
        ffmpeg::init().unwrap();
        let rate = 48_000;
        for speed in [0.25f32, 0.5, 1.5, 2.0, 4.0] {
            let mut t = Tempo::new(speed, rate, 2).unwrap();
            let block = 1024;
            let total_in = rate as usize * 2; // 2 s
            let mut out_frames = 0;
            let mut pts_seen = Vec::new();
            for i in 0..total_in / block {
                let pts = (i * block) as f64 / f64::from(rate);
                let samples: Vec<f32> = (0..block * 2)
                    .map(|k| ((i * block + k / 2) as f32 * 0.05).sin() * 0.5)
                    .collect();
                for o in t.process(pts, &samples).unwrap() {
                    out_frames += o.samples.len() / 2;
                    pts_seen.push(o.pts);
                }
            }
            for o in t.finish().unwrap() {
                out_frames += o.samples.len() / 2;
                pts_seen.push(o.pts);
            }
            let out_secs = out_frames as f64 / f64::from(rate);
            let expected = 2.0 / f64::from(speed);
            assert!((out_secs - expected).abs() < 0.05 * expected, "speed {speed}: {out_secs}s, want {expected}s");
            // pts are media time: monotonic and ending near 2 s regardless of speed.
            assert!(pts_seen.windows(2).all(|w| w[1] >= w[0]), "speed {speed}: pts not monotonic");
            let last = *pts_seen.last().unwrap();
            assert!((1.8..2.05).contains(&last), "speed {speed}: last pts {last}");
        }
    }
}
