extern crate scripture_types;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::path::Path;

use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;
use data_bundler;

#[cfg(windows)]
pub const NPM: &'static str = "npm.cmd";

#[cfg(not(windows))]
pub const NPM: &'static str = "npm";
// enum AndOr {
//     And = 1,
//     Or = 0,
// }

fn ensure_data_source(test_path: &std::path::PathBuf) {
    if Path::new(&test_path).exists() {
        println!("\nData source exists, skipping install.");
    } else {
        println!("\nData source not found, installing...");
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("{} i", NPM))
            .status()
            .expect("Unable to install data source");
    }
}

pub fn read_file(filepath: &str) -> String {
    let file = File::open(filepath).expect("could not open file");
    let mut buffered_reader = BufReader::new(file);
    let mut contents = String::new();
    let _number_of_bytes: usize = match buffered_reader.read_to_string(&mut contents) {
        Ok(number_of_bytes) => number_of_bytes,
        Err(_err) => 0,
    };

    contents
}

pub fn copy_minified<T: serde::de::DeserializeOwned + serde::ser::Serialize>(
    src_folder: &std::path::PathBuf,
    dest_folder: &std::path::PathBuf,
    file_name: &str,
) -> T {
    println!("    {}", file_name);
    let mut src = src_folder.clone();
    src.push(file_name);

    let unparsed: String = read_file(&src.into_os_string().into_string().unwrap());
    let parsed: T = serde_json::from_str(&unparsed).unwrap();

    let mut dest = dest_folder.clone();
    dest.push(format!("{}.{}", file_name, "gz"));

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    serde_json::to_writer(&mut encoder, &parsed).unwrap();

    let mut f_gzipped = BufWriter::new(File::create(dest).unwrap());
    let gzipped = encoder.finish().unwrap();
    f_gzipped.write(&gzipped).unwrap();

    parsed
}

pub fn write_minified<T: serde::ser::Serialize>(
    data: &T,
    dest_folder: &std::path::PathBuf,
    file_name: &str,
) -> () {
    let mut dest = dest_folder.clone();
    dest.push(format!("{}.{}", file_name, "gz"));

    println!("writing {}", file_name);

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    serde_json::to_writer(&mut encoder, data).unwrap();

    let mut f_gzipped = BufWriter::new(File::create(dest).unwrap());
    let gzipped = encoder.finish().unwrap();
    f_gzipped.write(&gzipped).unwrap();
}

fn main() {
    let mut project_root = std::env::current_exe().expect("Unable to find");
    project_root.pop();
    project_root.pop();
    project_root.pop();

    let mut src_folder = project_root.clone();
    src_folder.push("node_modules");
    src_folder.push("@bencrowder");
    src_folder.push("scriptures-json");

    let mut test_path = src_folder.clone();
    test_path.push("old-testament.json");
    ensure_data_source(&test_path);

    let mut dest_folder = project_root.clone();
    dest_folder.push("data-bundler");
    dest_folder.push("data");

    println!("Minifying:");
    let ot = copy_minified::<scripture_types::OldTestament>(
        &src_folder,
        &dest_folder,
        "old-testament.json",
    );

    let nt = copy_minified::<scripture_types::NewTestament>(
        &src_folder,
        &dest_folder,
        "new-testament.json",
    );

    let bom = copy_minified::<scripture_types::BookOfMormon>(
        &src_folder,
        &dest_folder,
        "book-of-mormon.json",
    );

    let dc = copy_minified::<scripture_types::DoctrineAndCovenants>(
        &src_folder,
        &dest_folder,
        "doctrine-and-covenants.json",
    );

    let pogp = copy_minified::<scripture_types::PearlOfGreatPrice>(
        &src_folder,
        &dest_folder,
        "pearl-of-great-price.json",
    );
    println!("Minifying done!\n");

    println!("Building indices:");
    let (words_index, paths_index) = data_bundler::build_index(&ot, &nt, &bom, &dc, &pogp);
    println!("Index building done!\n");
    println!("total word stems: {}", words_index.len());
    println!("total paths: {}", paths_index.len());
    write_minified(&paths_index, &dest_folder, "paths-index.json");
    write_minified(&words_index, &dest_folder, "words-index.json");
}
