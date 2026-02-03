/* Copyright (C) 2026 Tiago Duarte - All Rights Reserved */

use std::{
    fs::{self, read_dir},
    path::Path,
};

#[derive(Debug, Default)]
pub struct Mp3Tag {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<String>,
}

impl Mp3Tag {
    pub fn from_path(path: &Path) -> Self {
        let album = path.file_name().expect("Could not get the file_name");
        Mp3Tag {
            artist: Some("".into()),
            album: Some(album.to_str().unwrap().into()),
            year: Some("".into()),
        }
    }

    pub fn to_string(&self) -> String {
        let mut content = String::new();
        if let Some(artist) = &self.artist {
            content.push_str(&format!("artist: {}\n", artist));
        }
        if let Some(album) = &self.album {
            content.push_str(&format!("album: {}\n", album));
        }
        if let Some(year) = &self.year {
            content.push_str(&format!("year: {}\n", year));
        }
        content
    }
}

pub fn create_mp3tag_files(path: &Path) {
    let mut path_has_audio_files = false;
    let mut contains_mp3_tag = false;

    for dir in read_dir(path).expect("Could not read_dir") {
        let dir = dir.expect("Could not get the dir from read_dir");
        let metadata = dir.metadata().expect("Could not get metadata from entry");

        if metadata.is_file() {
            let file_path = dir.path();
            let file_name = file_path.file_name().expect("Could not get the file_name");
            let file_path = file_path
                .to_str()
                .expect("Could not get the file_path as str");

            if file_path.ends_with(".aiff") {
                path_has_audio_files = true;
            } else if file_name == "mp3tag.txt" {
                contains_mp3_tag = true;
            }
        } else if metadata.is_dir() {
            create_mp3tag_files(&dir.path());
        } else {
            println!("Invalid entry - skip");
        }
    }

    if path_has_audio_files && !contains_mp3_tag {
        let mut mp3_tag_path = path.to_path_buf();
        mp3_tag_path.push("mp3tag.txt");

        fs::write(mp3_tag_path, Mp3Tag::from_path(path).to_string())
            .expect("Could not write empty mp3tag.txt");
    }
}

pub fn parse_metadata(file: &str) -> Mp3Tag {
    let mp3_tag_content = std::fs::read(file).expect("Could not get Mp3Tag content");
    let mp3_tag_content =
        String::from_utf8(mp3_tag_content).expect("Could not convert Mp3Tag data to String");

    let mut metadata = Mp3Tag::default();

    for line in mp3_tag_content.lines() {
        let (key, value) = match line.split_once(':') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue, // skip malformed lines
        };

        match key {
            "artist" => metadata.artist = Some(value.to_string()),
            "album" => metadata.album = Some(value.to_string()),
            "year" => metadata.year = Some(value.to_string()),
            _ => {} // ignore unknown keys
        }
    }

    metadata
}
