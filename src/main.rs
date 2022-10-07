use std::env;
use std::time::{Duration, Instant};

use eframe::epaint::Rgba;
use egui::{ColorImage, Pos2, TextureFilter, TextureHandle, Vec2, Visuals};
use image::codecs::gif::GifDecoder;
use image::AnimationDecoder;
use kira::manager::backend::DefaultBackend;
use kira::manager::{AudioManager, AudioManagerSettings, MainPlaybackState};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::tween::{Easing, Tween};
use kira::StartTime;

pub struct Vibin {
    pos: Pos2,
    images: Vec<(TextureHandle, u32)>,
    current: usize,
    last_change: Instant,
    audio: AudioManager,
    volume: f64,
}

impl Vibin {
    pub fn new(cc: &eframe::CreationContext<'_>, images: Vec<(TextureHandle, u32)>) -> Self {
        cc.egui_ctx.set_visuals(Visuals::dark());

        let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .expect("Can't initialize audio context");

        let settings = StaticSoundSettings::new().volume(0.8);

        #[cfg(feature = "bundle-audio")]
        let sound = {
            let data = include_bytes!(env!(
                "VIBIN_BUNDLE",
                "Feature 'bundle-audio' is set but the VIBIN_BUNDLE env var is not"
            ));
            StaticSoundData::from_cursor(std::io::Cursor::new(data), settings)
                .expect("Can't decode bundled sound file")
        };

        #[cfg(not(feature = "bundle-audio"))]
        let sound = {
            StaticSoundData::from_file(
                env::args().skip(1).collect::<Vec<String>>().join(" "),
                settings,
            )
            .expect("Can't load sound file")
        };

        manager.play(sound).expect("Can't play sound");

        Self {
            pos: Pos2::new(100.0, 100.0),
            images,
            current: 0,
            last_change: Instant::now(),
            audio: manager,
            volume: 1.0,
        }
    }
}

impl eframe::App for Vibin {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        {
            let input = ctx.input();
            if input.pointer.middle_down() {
                frame.close();
            }

            if input.pointer.primary_down() {
                if let Some(orig) = input.pointer.press_origin() {
                    if let Some(end) = input.pointer.interact_pos() {
                        self.pos += end - orig;
                        frame.set_window_pos(self.pos);
                    }
                }
            }

            if input.pointer.secondary_down() && input.pointer.any_pressed() {
                match self.audio.state() {
                    MainPlaybackState::Paused | MainPlaybackState::Pausing => self
                        .audio
                        .resume(Tween {
                            start_time: StartTime::Immediate,
                            duration: Duration::from_millis(350),
                            easing: Easing::InPowf(1.0),
                        })
                        .expect("Can't resume audio"),
                    MainPlaybackState::Playing => self
                        .audio
                        .pause(Tween {
                            start_time: StartTime::Immediate,
                            duration: Duration::from_millis(350),
                            easing: Easing::OutPowf(1.0),
                        })
                        .expect("Can't pause audio"),
                }
            }

            self.volume = (self.volume + input.scroll_delta.y as f64 / 1000.0).clamp(0.0, 1.0);
            self.audio
                .main_track()
                .set_volume(self.volume, Tween::default())
                .expect("Can't set volume");
        }

        let delta =
            self.images[self.current].1 as i32 - self.last_change.elapsed().as_micros() as i32;
        if delta <= 0 {
            self.current = (self.current + 1) % self.images.len();
            self.last_change = Instant::now();
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(Duration::from_micros(delta as u64));
        }

        egui::Area::new("main_area")
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                ui.image(&self.images[self.current].0, Vec2::new(128.0, 128.0))
            });
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    fn clear_color(&self, _visuals: &Visuals) -> Rgba {
        Rgba::from_black_alpha(0.0)
    }
}

#[cfg(feature = "bundle-gif-cat")]
static GIF_DATA: &[u8] = include_bytes!("../cat.gif");

#[cfg(not(feature = "bundle-gif-cat"))]
static GIF_DATA: &[u8] = include_bytes!(env!("VIBIN_GIF"));

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
                        TextureFilter::Linear,
                    );
                    let (num, den) = f.delay().numer_denom_ms();
                    (handle, (num as f32 * 1000.0 / den as f32).round() as u32)
                })
                .collect();
            Box::new(Vibin::new(cc, images))
        }),
    );
}
