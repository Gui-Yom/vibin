use std::path::Path;
use std::slice::from_raw_parts;
use std::{fs, mem};

/*
#[used]
#[link_section = ".vibin"]
pub static mut GIF_SIZE: usize = GIF_DATA.len();

// TODO remove feature and use optional env

#[cfg(feature = "bundle-gif-cat")]
#[link_section = ".vibin"]
pub static GIF_DATA: [u8; include_bytes!("../gifs/cat.gif").len()] =
    *include_bytes!("../gifs/cat.gif");

#[cfg(not(feature = "bundle-gif-cat"))]
#[link_section = ".vibin"]
pub static GIF_DATA: [u8; include_bytes!(env!("VIBIN_GIF")).len()] =
    *include_bytes!(env!("VIBIN_GIF"));

#[used]
#[link_section = ".vibin"]
pub static mut AUDIO_SIZE: usize = AUDIO_DATA.len();

#[no_mangle]
#[used]
#[link_section = ".vibin"]
pub static AUDIO_DATA: [u8; include_bytes!(env!("VIBIN_AUDIO")).len()] =
    *include_bytes!(env!("VIBIN_AUDIO"));*/

mutself::mutself! {
    pub GIF = *include_bytes!("C:\\Code\\Rust\\vibin\\gifs\\cat.gif");
    pub AUDIO = *include_bytes!("C:\\Users\\Guillaume\\Desktop\\polish_cow.mp3");
}
