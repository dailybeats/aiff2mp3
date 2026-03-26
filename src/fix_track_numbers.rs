/* Copyright (C) 2026 Tiago Duarte - All Rights Reserved */

use std::fs::read_dir;
use std::path::Path;

use id3::{Tag, TagLike, Version};

pub fn fix_track_numbers(folder: &Path) {
    for entry in read_dir(folder).expect("Could not read_dir") {
        let entry = entry.expect("Could not read entry");
        let path = entry.path();
        let metadata = entry.metadata().expect("Could not get metadata");

        if metadata.is_dir() {
            fix_track_numbers(&path);
        } else if path.extension().map(|e| e == "mp3").unwrap_or(false) {
            fix_track_number(&path);
        }
    }
}

fn fix_track_number(path: &Path) {
    let stem = path
        .file_stem()
        .expect("Could not get file stem")
        .to_string_lossy()
        .into_owned();

    let track_number = match stem.split_once(' ') {
        Some((prefix, _)) if prefix.chars().all(|c| c.is_ascii_digit()) && !prefix.is_empty() => {
            match prefix.parse::<u32>() {
                Ok(n) => n,
                Err(_) => return,
            }
        }
        _ => return,
    };

    let mut tag = Tag::read_from_path(path).unwrap_or_else(|_| Tag::new());
    let current_track = tag.track();

    if current_track == Some(track_number) {
        return;
    }

    println!(
        "  {:?}: {:?} -> {}",
        path.file_name().unwrap(),
        current_track,
        track_number
    );
    tag.set_track(track_number);
    tag.write_to_path(path, Version::Id3v23)
        .expect("Could not write ID3 tag");
}
