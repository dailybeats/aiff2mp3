/* Copyright (C) 2026 Tiago Duarte - All Rights Reserved */

use std::fs::{self, read_dir};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use aifc::Sample;
use mp3lame_encoder::{Builder, FlushNoGap, Id3Tag, InterleavedPcm};

use crate::mp3tag::{Mp3Tag, parse_metadata};

#[derive(Default)]
struct MusicFolder {
    name: String,
    path: PathBuf,
    tag: Option<Mp3Tag>,
    files: Vec<PathBuf>,
}

fn collect_aiff_files(start_folder: &Path) -> Vec<MusicFolder> {
    let mut paths = vec![];

    let mut current_folder = MusicFolder::default();
    current_folder.name = start_folder
        .file_name()
        .expect("Could not get file_name")
        .to_str()
        .unwrap()
        .into();
    current_folder.path = start_folder.to_path_buf();

    for dir in read_dir(start_folder).expect("Could not read_dir") {
        let dir = dir.expect("Could not get the dir from read_dir");
        let metadata = dir.metadata().expect("Could not get metadata from entry");

        if metadata.is_file() {
            let file_path = dir.path();
            let file_name = file_path.file_name().expect("Could not get the file_name");
            let file_path = file_path
                .to_str()
                .expect("Could not get the file_path as str");

            if file_path.ends_with(".aiff") {
                current_folder.files.push(file_path.into());
            } else if file_name == "mp3tag.txt" {
                current_folder.tag = Some(parse_metadata(file_path));
            }
        } else if metadata.is_dir() {
            paths.append(&mut collect_aiff_files(&dir.path()));
        } else {
            println!("Invalid entry - skip");
        }
    }

    if !current_folder.files.is_empty() {
        paths.push(current_folder);
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

fn create_mp3_file(path: &Path, tag: &Option<Mp3Tag>, samples: Vec<u16>) {
    let file_name = path.file_name().expect("Invalid file_name");
    let mut artist = String::new();
    let mut album = String::new();
    let mut year = String::new();

    if let Some(t) = tag {
        artist = t.artist.clone().or(Some("".into())).unwrap();
        album = t.album.clone().or(Some("".into())).unwrap().to_string();
        year = t.year.clone().or(Some("".into())).unwrap().to_string();
    }
    println!("  - artist: {}, album: {}, year: {}", artist, album, year);

    let id3tag = Id3Tag {
        title: file_name.as_bytes(),
        artist: artist.as_bytes(),
        album: album.as_bytes(),
        album_art: &[],
        year: year.as_bytes(),
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

    let mut mp3_file = path.parent().expect("Could not get parent").to_path_buf();
    mp3_file.push("aiff2mp3");
    mp3_file.push(format!(
        "{}.mp3",
        path.file_prefix().unwrap().to_str().unwrap()
    ));
    println!("  - Write MP3 file to {:?}", mp3_file);
    std::fs::write(mp3_file, mp3_out_buffer).expect("Failed to write mp3 file");
}

pub fn convert_aiff_file_on_path(path: &Path) {
    let music_folders = collect_aiff_files(path);
    for folder in music_folders {
        println!("Parsing folder {} ({:?})", folder.name, folder.path);

        let mut mp3_folder = folder.path.to_path_buf();
        mp3_folder.push("aiff2mp3");
        println!("Creating aiff2mp3 folder: {:?}", mp3_folder);
        fs::create_dir_all(mp3_folder).expect("Could not create aiff2mp3 foldder");

        for file in folder.files {
            println!(" - Parsing {:?}", file);
            println!(" - Getting samples");
            let samples = get_samples(&file);
            println!(" - Creating MP3 file");
            create_mp3_file(&file, &folder.tag, samples);
        }
    }
}
