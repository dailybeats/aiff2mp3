/* Copyright (C) 2026 Tiago Duarte - All Rights Reserved */
use std::path::Path;
use std::process::exit;

use crate::convert::convert_aiff_file_on_path;
use crate::mp3tag::create_mp3tag_files;

mod convert;
mod mp3tag;

fn usage() {
    println!("Usage: aiff2mp3 [PATH_TO_THE_FOLDER] init|convert");
    exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let _ = args.get(0).expect("Could not get the program name");

    if args.len() != 3 {
        println!("Invalid arguments");
        usage();
    }

    let arg_path = args.get(1);
    if arg_path.is_none() {
        println!("The folder with aiff files was not provided");
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

    match args.get(2) {
        Some(option) => match option.as_str() {
            "init" => {
                println!(
                    "Creating mp3tag.txt files on {} subfolders",
                    path.to_str().expect("Could not convert path to str")
                );
                create_mp3tag_files(path)
            }
            "convert" => {
                println!(
                    "Converting aiff to mp3 on {} subfolders",
                    path.to_str().expect("Could not convert path to str")
                );
                convert_aiff_file_on_path(path)
            }
            _ => usage(),
        },
        None => usage(),
    }
}
