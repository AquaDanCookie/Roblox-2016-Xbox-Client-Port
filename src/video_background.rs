use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, TryRecvError, sync_channel};

use egui::{Color32, ColorImage, Context, TextureHandle, TextureOptions};

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const FRAME_BYTES: usize = WIDTH * HEIGHT * 4;

pub struct VideoBackground {
    frames: Receiver<Vec<u8>>,
    child: Child,
    texture: Option<TextureHandle>,
    path: PathBuf,
    ended: bool,
    luma: f32,
}

impl VideoBackground {
    pub fn start(path: &Path) -> Result<Self, String> {
        let mut child = Command::new("ffmpeg")
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-nostdin",
                "-stream_loop",
                "-1",
                "-re",
                "-i",
            ])
            .arg(path)
            .args([
                "-an",
                "-vf",
                "scale=1920:1080:force_original_aspect_ratio=increase:flags=fast_bilinear,crop=1920:1080",
                "-r",
                "30",
                "-pix_fmt",
                "rgba",
                "-f",
                "rawvideo",
                "-",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("Could not start ffmpeg: {error}"))?;

        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| "ffmpeg did not expose its output stream".to_owned())?;
        let (tx, frames) = sync_channel(1);
        std::thread::spawn(move || {
            loop {
                let mut frame = vec![0u8; FRAME_BYTES];
                if stdout.read_exact(&mut frame).is_err() {
                    break;
                }
                if tx.send(frame).is_err() {
                    break;
                }
            }
        });

        Ok(Self {
            frames,
            child,
            texture: None,
            path: path.to_path_buf(),
            ended: false,
            luma: 0.0,
        })
    }

    pub fn poll(&mut self, ctx: &Context) -> Option<TextureHandle> {
        let mut latest = None;
        loop {
            match self.frames.try_recv() {
                Ok(frame) => latest = Some(frame),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.ended = true;
                    break;
                }
            }
        }

        if let Some(frame) = latest {
            let mut sum = 0.0f32;
            let mut count = 0.0f32;
            let mut pixels = Vec::with_capacity(WIDTH * HEIGHT);
            for (index, pixel) in frame.chunks_exact(4).enumerate() {
                if index % 64 == 0 {
                    sum += 0.2126 * pixel[0] as f32
                        + 0.7152 * pixel[1] as f32
                        + 0.0722 * pixel[2] as f32;
                    count += 1.0;
                }
                pixels.push(Color32::from_rgb(pixel[0], pixel[1], pixel[2]));
            }
            if count > 0.0 {
                self.luma = (sum / count / 255.0).clamp(0.0, 1.0);
            }
            let image = ColorImage::new([WIDTH, HEIGHT], pixels);
            match &mut self.texture {
                Some(texture) => texture.set(image, TextureOptions::LINEAR),
                None => {
                    self.texture = Some(ctx.load_texture(
                        "menu-background-video",
                        image,
                        TextureOptions::LINEAR,
                    ));
                }
            }
        }

        self.texture.clone()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn has_frame(&self) -> bool {
        self.texture.is_some()
    }

    pub fn luma(&self) -> f32 {
        self.luma
    }

    pub fn ended(&self) -> bool {
        self.ended
    }
}

impl Drop for VideoBackground {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
