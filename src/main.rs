use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::exit;

// use symphonia::core::codecs::CodecRegistry;

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
        println!("{file:?}");
    }
}
