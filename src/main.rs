use std::fs::read_dir;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::exit;

use aifc::Sample;
use mp3lame_encoder::{Builder, FlushNoGap, Id3Tag, InterleavedPcm};

fn collect_aiff_files(start_folder: &Path) -> Vec<PathBuf> {
    let mut paths = vec![];

    for entry in read_dir(start_folder).expect("Could not read_dir") {
        let e = entry.expect("Could not get the entry from read_dir");
        let metadata = e.metadata().expect("Could not get metadata from entry");
        if metadata.is_file() {
            let file_path = e.path();
            if file_path.to_str().unwrap().ends_with(".aiff") {
                paths.push(file_path.into());
            }
        } else if metadata.is_dir() {
            paths.append(&mut collect_aiff_files(&e.path()));
        } else {
            println!("Invalid entry - skip");
        }
    }

    return paths;
}

fn get_samples(path: &Path) -> Vec<u16> {
    let mut stream = std::io::BufReader::new(std::fs::File::open(path).expect("Open failed"));
    let mut reader = aifc::AifcReader::new(&mut stream).expect("Can't create reader");
    // let info = reader.info();
    // println!("{:?}", info);
    let samples = reader.samples().expect("Could not get samples");

    let mut data = vec![];
    for s in samples {
        let sample = s.unwrap();
        match sample {
            Sample::U8(v) => data.push(v as u16),
            Sample::I8(v) => data.push(v as u16),

            Sample::I16(v) => data.push(v as u16),
            Sample::I24(v) => data.push(v as u16),
            Sample::I32(v) => data.push(v as u16),
            Sample::F32(v) => data.push(v as u16),
            Sample::F64(v) => data.push(v as u16),
        }
    }
    data
}

fn create_mp3_file(path: &Path, samples: Vec<u16>) {
    let id3tag = Id3Tag {
        title: path.file_name().expect("Invalid file_name").as_bytes(),
        artist: &[],
        album: b"My album",
        album_art: &[],
        year: b"Current year",
        comment: b"Created by aiff2mp3",
    };

    println!("  - Buiding mp3_encoder");
    let mut mp3_encoder = Builder::new()
        .expect("Create LAME builder")
        .with_num_channels(2)
        .expect("set channels")
        .with_sample_rate(44_100)
        .expect("set sample rate")
        .with_brate(mp3lame_encoder::Bitrate::Kbps192)
        .expect("set brate")
        .with_quality(mp3lame_encoder::Quality::Best)
        .expect("set quality")
        .with_id3_tag(id3tag)
        .expect("set tags")
        .build()
        .expect("To initialize LAME encoder");

    println!("  - InterleavedPcm input");
    let interleaved_pcm = InterleavedPcm(&samples);
    println!("  - Reserving output buffer");
    let mut mp3_out_buffer = Vec::new();
    mp3_out_buffer.reserve(mp3lame_encoder::max_required_buffer_size(samples.len() / 2));

    println!("  - Encoding...");
    let encoded_size = mp3_encoder
        .encode(interleaved_pcm, mp3_out_buffer.spare_capacity_mut())
        .expect("To encode");
    unsafe {
        mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
    }

    println!("  - Flushing...");
    let encoded_size = mp3_encoder
        .flush::<FlushNoGap>(mp3_out_buffer.spare_capacity_mut())
        .expect("to flush");
    unsafe {
        mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
    }

    let mp3_file = path.with_extension("mp3");
    println!("  - Write MP3 file to {:?}", mp3_file);
    std::fs::write(mp3_file, mp3_out_buffer).expect("Failed to write mp3 file");
}

fn usage() {
    println!("Usage: aiff2mp3 [PATH_TO_THE_FOLDER]");
    exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let _ = args.get(0).expect("Could not get the program name");

    let arg_path = args.get(1);
    if arg_path.is_none() {
        println!("The folder with aiff files was not provided.");
        usage();
    }

    let path = Path::new(arg_path.expect("Already checked if it's none"));
    if !path.exists() {
        println!(
            "{} does not exist...",
            path.to_str().expect("Could not convert path to str")
        );
        usage();
    }

    println!(
        "Searching aiff files on: {}",
        path.to_str().expect("Could not convert path to str")
    );

    let aiff_files = collect_aiff_files(path);
    for file in aiff_files {
        println!("Parsing {:?}", file);
        println!(" - Getting samples");
        let samples = get_samples(&file);
        println!(" - Creating MP3 file");
        create_mp3_file(&file, samples);
    }
}
