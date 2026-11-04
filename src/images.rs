use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use egui::{Color32, ColorImage, Context, TextureHandle, TextureOptions};
use reqwest::Client;
use tokio::runtime::Handle;

const USER_AGENT: &str = "rbx2016/0.1 (Linux; Wayland; local fan build)";

pub struct ImageCache {
    entries: HashMap<String, TextureHandle>,
    pending: HashSet<String>,
    failed: HashSet<String>,
    tx: Sender<ImageEvent>,
    rx: Receiver<ImageEvent>,
    runtime: Handle,
    client: Client,
    dir: PathBuf,
}

enum ImageEvent {
    Loaded { url: String, image: ColorImage },
    Failed { url: String, error: String },
}

impl ImageCache {
    pub fn new(runtime: Handle, dir: PathBuf) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(25))
            .build()
            .unwrap_or_default();
        Self {
            entries: HashMap::new(),
            pending: HashSet::new(),
            failed: HashSet::new(),
            tx,
            rx,
            runtime,
            client,
            dir,
        }
    }

    pub fn ensure(&mut self, url: &str) {
        if url.is_empty()
            || self.entries.contains_key(url)
            || self.pending.contains(url)
            || self.failed.contains(url)
        {
            return;
        }
        self.pending.insert(url.to_owned());

        let url = url.to_owned();
        let tx = self.tx.clone();
        let client = self.client.clone();
        let dir = self.dir.clone();
        self.runtime.spawn(async move {
            let path = dir.join(format!("{:016x}.img", fnv1a(url.as_bytes())));
            let bytes = match std::fs::read(&path) {
                Ok(bytes) => bytes,
                Err(_) => match download(&client, &url).await {
                    Ok(bytes) => {
                        if let Some(parent) = path.parent()
                            && let Err(error) = std::fs::create_dir_all(parent)
                        {
                            log::warn!("Could not create thumbnail cache: {error}");
                        }
                        if let Err(error) = std::fs::write(&path, &bytes) {
                            log::warn!("Could not write thumbnail cache: {error}");
                        }
                        bytes
                    }
                    Err(error) => {
                        let _ = tx.send(ImageEvent::Failed { url, error });
                        return;
                    }
                },
            };

            match tokio::task::spawn_blocking(move || decode(&bytes)).await {
                Ok(Ok(image)) => {
                    let _ = tx.send(ImageEvent::Loaded { url, image });
                }
                Ok(Err(error)) => {
                    let _ = std::fs::remove_file(&path);
                    let _ = tx.send(ImageEvent::Failed { url, error });
                }
                Err(error) => {
                    let _ = tx.send(ImageEvent::Failed {
                        url,
                        error: error.to_string(),
                    });
                }
            }
        });
    }

    pub fn poll(&mut self, ctx: &Context) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                ImageEvent::Loaded { url, image } => {
                    self.pending.remove(&url);
                    let texture =
                        ctx.load_texture(format!("thumbnail:{url}"), image, TextureOptions::LINEAR);
                    self.entries.insert(url, texture);
                    ctx.request_repaint();
                }
                ImageEvent::Failed { url, error } => {
                    self.pending.remove(&url);
                    self.failed.insert(url.clone());
                    log::warn!("Thumbnail failed ({}): {error}", short_url(&url));
                }
            }
        }
    }

    pub fn get(&self, url: &str) -> Option<&TextureHandle> {
        self.entries.get(url)
    }
}

async fn download(client: &Client, url: &str) -> Result<Vec<u8>, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status().as_u16()));
    }
    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|error| error.to_string())
}

fn decode(bytes: &[u8]) -> Result<ColorImage, String> {
    let image = image::load_from_memory(bytes).map_err(|error| error.to_string())?;
    let rgba = image.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let pixels = rgba
        .pixels()
        .map(|pixel| Color32::from_rgba_unmultiplied(pixel[0], pixel[1], pixel[2], pixel[3]))
        .collect();
    Ok(ColorImage::new(size, pixels))
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn short_url(url: &str) -> &str {
    url.split('/').next_back().unwrap_or(url)
}
