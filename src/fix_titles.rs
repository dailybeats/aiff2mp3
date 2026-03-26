/* Copyright (C) 2026 Tiago Duarte - All Rights Reserved */

use std::fs::read_dir;
use std::path::Path;

use id3::{Tag, TagLike, Version};

pub fn fix_mp3_titles(folder: &Path) {
    for entry in read_dir(folder).expect("Could not read_dir") {
        let entry = entry.expect("Could not read entry");
        let path = entry.path();
        let metadata = entry.metadata().expect("Could not get metadata");

        if metadata.is_dir() {
            fix_mp3_titles(&path);
        } else if path.extension().map(|e| e == "mp3").unwrap_or(false) {
            fix_title(&path);
        }
    }
}

fn fix_title(path: &Path) {
    let correct_title = path
        .file_stem()
        .expect("Could not get file stem")
        .to_string_lossy()
        .into_owned();

    let mut tag = Tag::read_from_path(path).unwrap_or_else(|_| Tag::new());
    let current_title = tag.title().unwrap_or("").to_string();

    if current_title == correct_title {
        return;
    }

    println!(
        "  {:?}: {:?} -> {:?}",
        path.file_name().unwrap(),
        current_title,
        correct_title
    );
    tag.set_title(correct_title);
    tag.write_to_path(path, Version::Id3v23)
        .expect("Could not write ID3 tag");
}
