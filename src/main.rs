#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Cursor;
use std::time::{Duration, Instant};
use std::{env, fs, slice};

use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, ImageDecoder};
use kira::manager::backend::DefaultBackend;
use kira::manager::{AudioManager, AudioManagerSettings, MainPlaybackState};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::tween::{Easing, Tween};
use kira::{LoopBehavior, StartTime};
use softbuffer::GraphicsContext;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{WindowBuilder, WindowLevel};

mod staticdata;

fn main() {
    let mut args = env::args();
    if args.len() > 1 {
        // Self modification time !
        let exe = args.next().unwrap();

        let mut new_gif = None;
        let mut new_audio = None;

        for file in args {
            let content = fs::read(&file).unwrap();
            if GifDecoder::new(&content[..]).is_ok() {
                new_gif = Some(content);
            } else {
                let sound = StaticSoundData::from_cursor(
                    Cursor::new(content.clone()),
                    StaticSoundSettings::default(),
                );
                if sound.is_ok() {
                    new_audio = Some(content);
                } else {
                    // Can't interpret that first file as a gif or an audio file
                    panic!("Invalid file : {file}");
                }
            }
        }

        staticdata::mutself(&exe, new_gif.as_deref(), new_audio.as_deref());
        return;
    }

    let decoder = GifDecoder::new(Cursor::new(&*staticdata::GIF)).unwrap();
    let (width, height) = decoder.dimensions();
    let frames: Vec<_> = decoder
        .into_frames()
        .map(|f| {
            let mut f = f.unwrap();
            for p in f.buffer_mut().chunks_exact_mut(4) {
                p.copy_from_slice(&[p[2], p[1], p[0], p[3]]);
            }
            f
        })
        .collect();

    let event_loop = EventLoop::new();
    let mut builder = WindowBuilder::new()
        .with_decorations(false)
        .with_title("Vibin")
        .with_transparent(true)
        // .with_taskbar_icon(Some(
        //     Icon::from_rgba(frames[0].buffer().to_vec(), width, height).unwrap(),
        // ))
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_resizable(false)
        .with_inner_size(LogicalSize::new(width, height));

    #[cfg(windows)]
    {
        use winit::platform::windows::WindowBuilderExtWindows;
        builder = builder
            .with_skip_taskbar(false)
            .with_undecorated_shadow(false);
    }

    let window = builder.build(&event_loop).unwrap();
    let mut gctx = unsafe { GraphicsContext::new(&window, &window).unwrap() };

    let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
        .expect("Can't initialize audio context");

    let settings = StaticSoundSettings::new()
        .volume(0.8)
        .loop_behavior(LoopBehavior {
            start_position: 0.0,
        });

    let audio = StaticSoundData::from_cursor(Cursor::new(&*staticdata::AUDIO), settings)
        .expect("Can't decode bundled sound file");

    manager.play(audio).expect("Can't play sound");
    let mut volume = 0.2;
    manager
        .main_track()
        .set_volume(volume, Tween::default())
        .unwrap();

    let mut last_redraw = Instant::now();
    let mut i = 0;

    let mut drag = false;
    let mut prev_pos = window.outer_position().unwrap();
    let (numer, denom) = frames[i].delay().numer_denom_ms();
    let mut delay = Duration::from_secs_f32(numer as f32 / denom as f32 / 1000.0);

    event_loop.run(move |e, _, control_flow| {
        match e {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    if drag {
                        let wpos = window.outer_position().unwrap();
                        window.set_outer_position(PhysicalPosition {
                            x: wpos.x + position.x as i32 - prev_pos.x,
                            y: wpos.y + position.y as i32 - prev_pos.y,
                        });
                    } else {
                        prev_pos = position.cast::<i32>();
                    }
                }
                WindowEvent::MouseInput { button, state, .. } => match button {
                    MouseButton::Middle => {
                        control_flow.set_exit();
                        return;
                    }
                    MouseButton::Right => {
                        if state == ElementState::Pressed {
                            match manager.state() {
                                MainPlaybackState::Paused | MainPlaybackState::Pausing => manager
                                    .resume(Tween {
                                        start_time: StartTime::Immediate,
                                        duration: Duration::from_millis(350),
                                        easing: Easing::InPowf(1.0),
                                    })
                                    .expect("Can't resume audio"),
                                MainPlaybackState::Playing => manager
                                    .pause(Tween {
                                        start_time: StartTime::Immediate,
                                        duration: Duration::from_millis(350),
                                        easing: Easing::OutPowf(1.0),
                                    })
                                    .expect("Can't pause audio"),
                            }
                        }
                    }
                    MouseButton::Left => match state {
                        ElementState::Pressed => {
                            drag = true;
                        }
                        ElementState::Released => {
                            drag = false;
                        }
                    },
                    _ => {}
                },
                WindowEvent::MouseWheel { delta, .. } => {
                    let amount = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y as f64,
                        MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => y,
                    };
                    volume = (volume + amount / 60.0).clamp(0.0, 1.0);
                    manager
                        .main_track()
                        .set_volume(volume, Tween::default())
                        .expect("Can't set volume");
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                // No way I'm making a copy on each redraw
                let slice = frames[i].buffer().as_raw().as_slice();
                let (ptr, len) = (slice.as_ptr(), slice.len());
                gctx.set_buffer(
                    unsafe { slice::from_raw_parts(ptr as *const u32, len / 4) },
                    width as u16,
                    height as u16,
                );
            }
            _ => {}
        }
        if last_redraw.elapsed() >= delay {
            i = (i + 1) % frames.len();
            last_redraw = Instant::now();
            let (numer, denom) = frames[i].delay().numer_denom_ms();
            delay = Duration::from_secs_f32(numer as f32 / denom as f32 / 1000.0);
            control_flow.set_wait_until(last_redraw + delay);
            window.request_redraw();
        }
    });
}
