use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{env, thread};

use eframe::epaint::Rgba;
use egui::{ColorImage, Pos2, TextureHandle, Vec2, Visuals};
use image::codecs::gif::GifDecoder;
use image::AnimationDecoder;
use kira::manager::backend::DefaultBackend;
use kira::manager::AudioManagerSettings;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};

static GIF_DATA: &[u8] = include_bytes!("../cat.gif");

pub struct Vibin {
    pos: Pos2,
    images: Vec<(TextureHandle, u32)>,
    current: Arc<AtomicUsize>,
}

impl Vibin {
    pub fn new(cc: &eframe::CreationContext<'_>, images: Vec<(TextureHandle, u32)>) -> Self {
        cc.egui_ctx.set_visuals(Visuals::dark());

        let current = Arc::new(AtomicUsize::new(0));
        let size = images.len();

        // UI update thread
        // Change the gif frame to display
        let delay = images[0].1;
        let ctx_clone = cc.egui_ctx.clone();
        let curr_clone = current.clone();
        thread::spawn(move || {
            let mut manager =
                kira::manager::AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                    .expect("Can't initialize audio context");

            #[cfg(feature = "bundle-audio")]
            let sound = {
                let data = include_bytes!(env!(
                    "VIBIN_BUNDLE",
                    "Feature 'bundle-audio' is set but the VIBIN_BUNDLE env var is not"
                ));
                StaticSoundData::from_cursor(
                    std::io::Cursor::new(data),
                    StaticSoundSettings::default(),
                )
                .expect("Can't decode bundled sound file")
            };

            #[cfg(not(feature = "bundle-audio"))]
            let sound = {
                StaticSoundData::from_file(
                    env::args().skip(1).collect::<Vec<String>>().join(" "),
                    StaticSoundSettings::default(),
                )
                .expect("Can't load sound file")
            };

            manager.play(sound).expect("Can't play sound");

            loop {
                thread::sleep(Duration::from_micros(delay as u64));
                curr_clone
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |c| {
                        Some((c + 1) % size)
                    })
                    .unwrap();
                ctx_clone.request_repaint();
            }
        });

        Self {
            pos: Pos2::new(100.0, 100.0),
            images,
            current,
        }
    }
}

impl eframe::App for Vibin {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        {
            let input = ctx.input();
            if input.pointer.middle_down() {
                frame.quit();
            }

            if input.pointer.primary_down() {
                if let Some(orig) = input.pointer.press_origin() {
                    if let Some(end) = input.pointer.interact_pos() {
                        self.pos += end - orig;
                        frame.set_window_pos(self.pos);
                    }
                }
            }
        }

        egui::Area::new("main_area")
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                ui.image(
                    &self.images[self.current.load(Ordering::Relaxed)].0,
                    Vec2::new(128.0, 128.0),
                )
            });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    fn clear_color(&self, _visuals: &Visuals) -> Rgba {
        Rgba::from_black_alpha(0.0)
    }
}

fn main() {
    let decoder = GifDecoder::new(GIF_DATA).unwrap();
    let frames = decoder
        .into_frames()
        .collect_frames()
        .expect("Can't decode frames");

    let width = frames[0].buffer().width();
    let height = frames[0].buffer().height();

    eframe::run_native(
        "Vibin",
        eframe::NativeOptions {
            always_on_top: true,
            decorated: false,
            initial_window_size: Some(Vec2::new(width as f32, height as f32)),
            initial_window_pos: Some(Pos2::new(100.0, 100.0)),
            vsync: true,
            resizable: false,
            transparent: true,
            ..Default::default()
        },
        Box::new(move |cc| {
            let images = frames
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let handle = cc.egui_ctx.load_texture(
                        format!("gif_frame_{i}"),
                        ColorImage::from_rgba_unmultiplied(
                            [f.buffer().width() as _, f.buffer().height() as _],
                            f.buffer(),
                        ),
                    );
                    let (num, den) = f.delay().numer_denom_ms();
                    (handle, (num as f32 * 1000.0 / den as f32).round() as u32)
                })
                .collect();
            Box::new(Vibin::new(cc, images))
        }),
    );
}
