use std::fs::File;

use av_format::buffer::AccReader;
use matroska::demuxer::MkvDemuxer;
use rustav::RustAVSource;

mod rustav;
fn main() {
    // Open the matroska file
    let reader =
        File::open("C:/Users/Guillaume/Desktop/memelord_music_pack/widewalk_mono.webm").unwrap();

    // Create a buffer of size 4096 B to contain matroska data
    let ar = AccReader::with_capacity(8192, reader);

    // Ignore unused variable warning, stream should not be dropped
    let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();

    // Add a dummy source of the sake of the example.
    let source = RustAVSource::new(Box::new(MkvDemuxer::new()), Box::new(ar))
        .expect("Can't create OpusDecoder");

    sink.append(source);
    //sink.append(rodio::source::SineWave::new(440));
    sink.set_volume(0.6);

    // Stop with ctrl+c
    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
