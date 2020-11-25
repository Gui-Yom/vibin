use std::fs::File;

use av_format::buffer::AccReader;
use matroska::demuxer::MkvDemuxer;
use rustav::RustAVSource;

mod rustav;
fn main() {
    // Open the matroska file
    let reader =
        File::open("C:/Users/Guillaume/Desktop/memelord_music_pack/wide_audio.webm").unwrap();

    // Create a buffer of size 4096 B to contain matroska data
    let ar = AccReader::with_capacity(4096, reader);

    let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();

    // Add a dummy source of the sake of the example.
    let source = RustAVSource::new(Box::new(MkvDemuxer::new()), Box::new(ar))
        .expect("Can't create OpusDecoder");

    sink.append(source);
    //sink.append(rodio::source::SineWave::new(440));
    sink.set_volume(0.5);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
