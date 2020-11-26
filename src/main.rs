use std::{fs::File, io::BufReader};

use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vibin")
        .with_resizable(false)
        .with_decorations(false)
        .with_visible(true)
        .with_inner_size(LogicalSize::new(128, 128))
        .build(&event_loop)
        .unwrap();

    // Open the matroska file
    let file = File::open("C:/Users/Guillaume/Desktop/memelord_music_pack/widewalk.ogg").unwrap();

    // Ignore unused variable warning, stream should not be dropped
    let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();

    // Add a dummy source of the sake of the example.
    let source = rodio::Decoder::new_vorbis(BufReader::new(file)).unwrap();

    sink.append(source);
    //sink.append(rodio::source::SineWave::new(440));
    sink.set_volume(0.6);

    event_loop.run(|e, target, control_flow| match e {
        winit::event::Event::WindowEvent { window_id, event } => match event {
            WindowEvent => {}
            _ => {}
        },
        _ => {}
    });
}
