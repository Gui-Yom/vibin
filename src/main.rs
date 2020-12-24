use libmpv::{events::Event, FileState, Format, Mpv};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition},
    event::{ElementState, ModifiersState, MouseScrollDelta, WindowEvent},
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

    /*
    mpv.playlist_load_files(&[(
        // Warning, click at you own risk
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        FileState::AppendPlay,
        None,
    )])?;
    */

    let mut modifiers_state: ModifiersState = ModifiersState::default();
    let mut volume = 80;
    let mut first_run = true;
    let mut left_pressed = false;
    let mut right_pressed = false;
    let mut last_position: Option<PhysicalPosition<f64>> = None;

    // The crossbeam scope allow us to spawn a thread without a 'static lifetime :)
    crossbeam_utils::thread::scope(|scope| {
        // A thread to handle mpv events.
        // Separating the window and mpv event loop allow us to
        // make the event poll wait free (I mean without cpu-eating wait-loop)
        scope.spawn(|_| {
            let e = mpv_event_ctx.wait_event(1000.0);
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
                _ => println!("Received event : {:?}", e),
            }
        });

        event_loop.run_return(|e, _target, control_flow| {
            if first_run {
                // Wait for OS events
                *control_flow = ControlFlow::Wait;
                first_run = false;
            }

            match e {
                winit::event::Event::WindowEvent {
                    window_id: _,
                    event,
                } => match event {
                    WindowEvent::ModifiersChanged(state) => {
                        modifiers_state = state;
                    }

                    WindowEvent::MouseWheel {
                        device_id: _,
                        delta: MouseScrollDelta::LineDelta(_, vertical),
                        ..
                    } => {
                        volume += 5 * vertical as i64;
                        mpv.set_property("volume", volume).unwrap();
                    }

                    WindowEvent::MouseInput {
                        device_id: _,
                        button,
                        state,
                        ..
                    } => match button {
                        winit::event::MouseButton::Middle => {
                            if state == ElementState::Pressed {
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                        winit::event::MouseButton::Left => {
                            match state {
                                ElementState::Pressed => {
                                    left_pressed = true;
                                }
                                ElementState::Released => {
                                    left_pressed = false;
                                }
                            }
                            // drag the window
                            println!(
                                "Left mouse button ({})",
                                if left_pressed { "pressed" } else { "released" }
                            );
                        }
                        winit::event::MouseButton::Right => {
                            match state {
                                ElementState::Pressed => {
                                    right_pressed = true;
                                }
                                ElementState::Released => {
                                    right_pressed = false;
                                }
                            }
                            // Play next item
                            println!(
                                "Right mouse button ({})",
                                if right_pressed { "pressed" } else { "released" }
                            );
                        }
                        _ => {}
                    },

                    WindowEvent::CursorMoved {
                        device_id: _,
                        position,
                        ..
                    } => {
                        if left_pressed {
                            if let Some(last_pos) = last_position {
                                let initial_window_pos = window.outer_position().unwrap();
                                let new_pos = PhysicalPosition::new(
                                    initial_window_pos.x + (position.x - last_pos.x) as i32,
                                    initial_window_pos.y + (position.y - last_pos.y) as i32,
                                );
                                window.set_outer_position(new_pos);
                                /*
                                println!(
                                    "outerPos: {:?}; lastPos: {:?}; pos: {:?}; newPos: {:?}",
                                    initial_window_pos, last_pos, position, new_pos
                                );*/
                            }
                            last_position = Some(position);
                        }
                    }

                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                },
                _ => {}
            }
        });
    })
    .expect("Problem with the scoped thread I guess.");

    Ok(())
}

/// Builds the window
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
