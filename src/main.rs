use std::time::{Duration, Instant};

use libmpv::{events::Event, FileState, Format, Mpv};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winit::{
    dpi::LogicalSize,
    event::{ModifiersState, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::{run_return::EventLoopExtRunReturn, windows::WindowBuilderExtWindows},
    window::{Window, WindowBuilder},
};

fn main() -> Result<(), libmpv::Error> {
    // Create the window
    let (mut event_loop, window) = init_window();

    // Retrieve its id
    let hwnd = match window.raw_window_handle() {
        RawWindowHandle::Windows(handle) => handle.hwnd,
        _ => panic!("Unsupported platform !"),
    };

    // Create a mpv instance, tell it to paint video in our window.
    let mpv = Mpv::with_initializer(|init| {
        init.set_property("vo", "gpu")?;
        init.set_property("gpu-context", "d3d11")?;
        init.set_property("hwdec", "auto-safe")?;
        init.set_property("wid", hwnd as i64)?;
        init.set_property("volume", 100)?;
        Ok(())
    })?;

    let mut mpv_event_ctx = mpv.create_event_context();
    mpv_event_ctx.disable_deprecated_events()?;
    mpv_event_ctx.observe_property("volume", Format::Int64, 0)?;
    mpv_event_ctx.observe_property("hwdec", Format::String, 0)?;

    mpv.playlist_load_files(&[(
        // Warning, click at you own risk
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        FileState::AppendPlay,
        None,
    )])?;

    // Open the matroska file
    //let file = File::open("C:/Users/Guillaume/Desktop/memelord_music_pack/widewalk.ogg").unwrap();

    let mut modifiers_state: ModifiersState = ModifiersState::default();
    let mut volume = 80;

    // The event loop
    event_loop.run_return(|e, _target, control_flow| {
        // Wait for OS events
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(100));

        match e {
            winit::event::Event::WindowEvent { window_id, event } => match event {
                WindowEvent::ModifiersChanged(state) => {
                    modifiers_state = state;
                }
                WindowEvent::MouseWheel {
                    device_id,
                    delta: MouseScrollDelta::LineDelta(_, vertical),
                    phase,
                    ..
                } => {
                    volume = volume + 5 * vertical as i64;
                    mpv.set_property("volume", volume).unwrap();
                }
                WindowEvent::MouseInput {
                    device_id, button, ..
                } => match button {
                    winit::event::MouseButton::Middle => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                },
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            _ => {}
        }
        if *control_flow == ControlFlow::Exit {
            return;
        }

        let e = mpv_event_ctx.wait_event(0.0);
        match e {
            Some(Ok(Event::EndFile(r))) => {
                println!("Stopping ! reason : {}", r);
            }
            Some(Ok(Event::PropertyChange { name, change, .. })) => {
                let display = match change {
                    libmpv::events::PropertyData::Str(val) => val.to_string(),
                    libmpv::events::PropertyData::OsdStr(val) => val.to_string(),
                    libmpv::events::PropertyData::Flag(val) => val.to_string(),
                    libmpv::events::PropertyData::Int64(val) => val.to_string(),
                    libmpv::events::PropertyData::Double(val) => val.to_string(),
                    libmpv::events::PropertyData::Node(val) => format!("{:?}", val),
                };
                println!("Property change : {} -> {}", name, display);
            }
            _ => {}
        }
    });
    Ok(())
}

fn init_window() -> (EventLoop<()>, Window) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vibin")
        .with_resizable(false)
        .with_decorations(false)
        .with_visible(true)
        .with_always_on_top(true)
        .with_inner_size(LogicalSize::new(128, 128))
        .with_drag_and_drop(false)
        .with_transparent(true)
        .build(&event_loop)
        .unwrap();
    return (event_loop, window);
}
