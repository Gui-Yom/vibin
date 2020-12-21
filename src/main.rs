use std::{fs::File, io::BufReader};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use winit::{
    dpi::LogicalSize, event::ModifiersState, event::MouseScrollDelta, event::WindowEvent,
    event_loop::ControlFlow, event_loop::EventLoop, platform::windows::WindowBuilderExtWindows,
    window::WindowBuilder,
};

mod gui;

struct AudioController {
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink,
}

impl AudioController {
    fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        AudioController {
            stream,
            stream_handle,
            sink,
        }
    }
}

fn main() {
    // Open the matroska file
    let file = File::open("C:/Users/Guillaume/Desktop/memelord_music_pack/widewalk.ogg").unwrap();

    let audio_controller = AudioController::new();

    // Add a dummy source of the sake of the example.
    let source = Decoder::new_vorbis(BufReader::new(file)).unwrap();
    audio_controller.sink.append(source);

    init_window(audio_controller);

    /*
    for _ in 0..5 {
        sleep(Duration::from_millis(1000))
    }
    */
}

fn init_window(audio_controller: AudioController) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vibin")
        .with_resizable(false)
        .with_decorations(true)
        .with_visible(true)
        .with_always_on_top(true)
        .with_inner_size(LogicalSize::new(128, 128))
        .with_drag_and_drop(false)
        .with_transparent(true)
        .build(&event_loop)
        .unwrap();
        

    let mut modifiers_state: ModifiersState = ModifiersState::default();
    event_loop.run(move |e, target, control_flow| {
        // Wait for OS events
        *control_flow = ControlFlow::Wait;

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
                    audio_controller
                        .sink
                        .set_volume(audio_controller.sink.volume() + 0.05 * vertical);
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            _ => {}
        }
    });
}
