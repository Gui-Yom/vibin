use std::{time::Duration, vec::IntoIter};

use audiopus::{coder::Decoder, TryFrom};
use av_data::{
    packet::Packet,
    params::{AudioInfo, MediaKind},
};
use av_format::{
    buffer::Buffered,
    demuxer::{Context, Demuxer, Event},
};

pub struct RustAVSource {
    demuxer: Context,
    decoder: Decoder,
    audio_info: AudioInfo,
    current_data: IntoIter<i16>,
    ptr: usize,
    size: usize
}

impl RustAVSource {
    pub fn new<R>(demuxer: Box<dyn Demuxer>, reader: Box<R>) -> Result<Self, String>
    where
        R: Buffered + 'static,
    {
        let mut demuxer = Context::new(demuxer, reader);
        let audio_info = Self::find_opus_stream(&mut demuxer);
        if audio_info.is_none() {
            return Err("Can't find an opus stream".into());
        }
        let audio_info = audio_info.unwrap();
        let decoder = Decoder::new(
            audiopus::SampleRate::try_from(audio_info.rate as i32).unwrap(),
            audiopus::Channels::Stereo,
        )
        .unwrap();
        Ok(RustAVSource {
            demuxer: demuxer,
            decoder,
            audio_info,
            current_data: Vec::with_capacity(0).into_iter(),
            ptr: 0,
            size: 0
        })
    }

    fn find_opus_stream(demuxer: &mut Context) -> Option<AudioInfo> {
        // Read matroska headers
        demuxer
            .read_headers()
            .expect("Cannot parse the file header");

        demuxer
            .info
            .streams
            .iter()
            .find_map(|stream| match stream.params.kind.clone() {
                Some(MediaKind::Audio(info)) => {
                    if let Some(id) = &stream.params.codec_id {
                        if id == "opus" {
                            return Some(info);
                        }
                    }
                    None
                }
                _ => None,
            })
    }

    fn next_data_packet(&mut self) -> Result<Packet, String> {
        match self.demuxer.read_event() {
            Ok(event) => match event {
                Event::NewPacket(pkt) => Ok(pkt),
                Event::Eof => Err("EOF".into()),
                _ => Err("not a packet".into()),
            },
            Err(err) => Err(format!("Malformed stream : {}", err)),
        }
    }

    fn next_samples(&mut self) -> Result<(Vec<i16>, usize), String> {
        let pkt = self.next_data_packet()?;
        println!("len: {}", pkt.data.len());
        let mut output = Vec::with_capacity(57600);
        let size = self
            .decoder
            .decode(Some(&pkt.data), &mut output, false)
            .map_err(|e| e.to_string())?;
        println!("decoded size : {}", size);
        Ok((output, size))
    }
}

impl rodio::Source for RustAVSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.size)
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        self.audio_info.rate as u32
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.demuxer.info.duration.map(|x| Duration::from_millis(x))
    }
}

impl Iterator for RustAVSource {
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        println!("Asking for next item ptr: {} size: {} datalen: {}", self.ptr, self.size, self.current_data.len());
        if self.ptr >= self.size || self.current_data.len() == 0 {
            let result =  self.next_samples();
            if result.is_err() {
                println!("{}", result.err().unwrap());
                return None;
            }
            if let Some((data, size)) = result.ok() {
                self.size = size;
                self.current_data = data.into_iter();
            }
            self.ptr = 0;
        }
        self.ptr += 1;
        self.current_data.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, None)
    }
}
