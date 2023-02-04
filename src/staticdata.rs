use std::path::Path;
use std::slice::from_raw_parts;
use std::{fs, mem};

use object::pe::{ImageDosHeader, ImageNtHeaders64};
use object::read::pe::{ImageNtHeaders, ImageOptionalHeader};
use object::write::pe;
use object::write::pe::NtHeaders;
use object::LittleEndian as LE;

#[used]
#[link_section = ".vibin"]
pub static mut GIF_SIZE: usize = GIF_DATA.len();

// TODO remove feature and use optional env

#[cfg(feature = "bundle-gif-cat")]
#[link_section = ".vibin"]
pub static GIF_DATA: [u8; include_bytes!("../cat.gif").len()] = *include_bytes!("../cat.gif");

#[cfg(not(feature = "bundle-gif-cat"))]
#[link_section = ".vibin"]
pub static GIF_DATA: [u8; include_bytes!(env!("VIBIN_GIF")).len()] =
    *include_bytes!(env!("VIBIN_GIF"));

pub(crate) fn get_gif_data() -> &'static [u8] {
    // Prevent bounds checking with from_raw_parts
    unsafe { from_raw_parts(&GIF_DATA as *const u8, GIF_SIZE) }
}

#[used]
#[link_section = ".vibin"]
pub static mut AUDIO_SIZE: usize = AUDIO_DATA.len();

#[no_mangle]
#[used]
#[link_section = ".vibin"]
pub static AUDIO_DATA: [u8; include_bytes!(env!("VIBIN_AUDIO")).len()] =
    *include_bytes!(env!("VIBIN_AUDIO"));

/// 16 byte alignment
fn align_16(addr: usize) -> usize {
    (addr & 0xFFFF_FFFF_FFFF_FFF0) + 0x10
}

pub(crate) fn get_audio_data() -> &'static [u8] {
    // The audio data is known to be after the gif data, only the gif data pointer.
    // The gif data address won't change.
    unsafe {
        // 16 bytes align
        let ptr = align_16(&GIF_DATA as *const u8 as usize + GIF_SIZE) as *const u8;
        let audio_size = (ptr as *const usize).read();
        from_raw_parts(ptr.add(mem::size_of::<usize>()), audio_size)
    }
}

pub(crate) fn rewrite_exe(
    exe: &str,
    new_gif: Option<(String, Vec<u8>)>,
    new_audio: Option<(String, Vec<u8>)>,
) {
    // Read the original exe
    let in_exe = fs::read(exe).unwrap();

    let dos_header = ImageDosHeader::parse(&in_exe[..]).unwrap();
    let mut offset = dos_header.nt_headers_offset() as u64;
    //let rich_header = RichHeaderInfo::parse(&data, offset);
    let (nt_headers, data_dir) = ImageNtHeaders64::parse(&in_exe[..], &mut offset).unwrap();
    let in_sections = nt_headers.sections(&in_exe[..], offset).unwrap();

    let mut out_data = Vec::with_capacity(in_exe.len());
    let mut writer = pe::Writer::new(
        true,
        nt_headers.optional_header.section_alignment(),
        nt_headers.optional_header.file_alignment(),
        &mut out_data,
    );

    // Plan spaces for data
    writer.reserve_dos_header_and_stub();
    writer.reserve_nt_headers(data_dir.len());
    for (i, dir) in data_dir.iter().enumerate() {
        if dir.virtual_address.get(LE) == 0
            || i == object::pe::IMAGE_DIRECTORY_ENTRY_SECURITY
            || i == object::pe::IMAGE_DIRECTORY_ENTRY_BASERELOC
        {
            continue;
        }
        writer.set_data_directory(i, dir.virtual_address.get(LE), dir.size.get(LE));
    }

    writer.reserve_section_headers(in_sections.len() as u16 + 1);

    let mut new_section = Vec::new();

    if let Some((_, gif_data)) = new_gif {
        new_section.extend_from_slice(&gif_data.len().to_le_bytes());
        new_section.extend(&gif_data);
    } else {
        new_section.extend_from_slice(&get_gif_data().len().to_le_bytes());
        new_section.extend_from_slice(get_gif_data());
    }

    // Extra align bytes
    new_section.resize(align_16(new_section.len()), 0);

    let audio_name = if let Some((audio_name, audio_data)) = new_audio {
        new_section.extend_from_slice(&audio_data.len().to_le_bytes());
        new_section.extend(&audio_data);
        Some(audio_name)
    } else {
        new_section.extend_from_slice(&get_audio_data().len().to_le_bytes());
        new_section.extend_from_slice(get_audio_data());
        None
    };

    let mut reserved_sections = Vec::with_capacity(in_sections.len());
    for s in in_sections.iter() {
        let reserved = match s.raw_name() {
            b".vibin" => (
                writer
                    .reserve_section(
                        s.name,
                        s.characteristics.get(LE),
                        new_section.len() as u32,
                        new_section.len() as u32,
                    )
                    .file_offset,
                &new_section[..],
            ),
            _ => (
                writer
                    .reserve_section(
                        s.name,
                        s.characteristics.get(LE),
                        s.virtual_size.get(LE),
                        s.size_of_raw_data.get(LE),
                    )
                    .file_offset,
                s.pe_data(&in_exe[..]).unwrap(),
            ),
        };
        reserved_sections.push(reserved);
    }

    let mut blocks = data_dir
        .relocation_blocks(&in_exe[..], &in_sections)
        .unwrap()
        .unwrap();
    while let Some(block) = blocks.next().unwrap() {
        for reloc in block {
            writer.add_reloc(reloc.virtual_address, reloc.typ);
        }
    }
    writer.reserve_reloc_section();

    // Start writing data
    writer.write_dos_header_and_stub().unwrap();
    writer.write_nt_headers(NtHeaders {
        machine: nt_headers.file_header.machine.get(LE),
        time_date_stamp: nt_headers.file_header.time_date_stamp.get(LE),
        characteristics: nt_headers.file_header.characteristics.get(LE),
        major_linker_version: nt_headers.optional_header.major_linker_version(),
        minor_linker_version: nt_headers.optional_header.minor_linker_version(),
        address_of_entry_point: nt_headers.optional_header.address_of_entry_point(),
        image_base: nt_headers.optional_header.image_base(),
        major_operating_system_version: nt_headers.optional_header.major_operating_system_version(),
        minor_operating_system_version: nt_headers.optional_header.minor_operating_system_version(),
        major_image_version: nt_headers.optional_header.major_image_version(),
        minor_image_version: nt_headers.optional_header.minor_image_version(),
        major_subsystem_version: nt_headers.optional_header.major_subsystem_version(),
        minor_subsystem_version: nt_headers.optional_header.minor_subsystem_version(),
        subsystem: nt_headers.optional_header.subsystem(),
        dll_characteristics: nt_headers.optional_header.dll_characteristics(),
        size_of_stack_reserve: nt_headers.optional_header.size_of_stack_reserve(),
        size_of_stack_commit: nt_headers.optional_header.size_of_stack_commit(),
        size_of_heap_reserve: nt_headers.optional_header.size_of_heap_reserve(),
        size_of_heap_commit: nt_headers.optional_header.size_of_heap_commit(),
    });
    writer.write_section_headers();
    for (offset, data) in reserved_sections {
        writer.write_section(offset, data);
    }
    writer.write_reloc_section();

    fs::write(
        if let Some(audio_name) = audio_name {
            format!(
                "{}.exe",
                Path::new(&audio_name)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
            )
        } else {
            format!("{exe}_new.exe")
        },
        &out_data,
    )
    .unwrap();
}
