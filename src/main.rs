/* Copyright (C) 2026 Tiago Duarte - All Rights Reserved */
use std::path::PathBuf;
use std::process::exit;

use clap::{Parser, Subcommand};

use crate::convert::convert_aiff_file_on_path;
use crate::fix_titles::fix_mp3_titles;
use crate::mp3tag::create_mp3tag_files;

mod convert;
mod fix_titles;
mod mp3tag;

#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Path to the folder containing AIFF files
    path: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create mp3tag.txt files recursively in all subfolders
    Init,
    /// Convert .aiff files to .mp3 in each subfolder's aiff2mp3/ directory
    Convert,
    /// Fix ID3 title tags on existing MP3 files (sets title to the file name)
    FixTitles,
}

fn main() {
    let cli = Cli::parse();

    if !cli.path.exists() {
        eprintln!("{} does not exist...", cli.path.display());
        exit(1);
    }

    match cli.command {
        Commands::Init => {
            println!(
                "Creating mp3tag.txt files on {} subfolders",
                cli.path.display()
            );
            create_mp3tag_files(&cli.path);
        }
        Commands::Convert => {
            println!(
                "Converting aiff to mp3 on {} subfolders",
                cli.path.display()
            );
            convert_aiff_file_on_path(&cli.path);
        }
        Commands::FixTitles => {
            println!("Fixing MP3 titles under {}", cli.path.display());
            fix_mp3_titles(&cli.path);
        }
    }
}
