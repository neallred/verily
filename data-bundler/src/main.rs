extern crate scripture_types;
extern crate phf_codegen;
extern crate phf;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

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

fn repr_verse_path(verse_path: &scripture_types::VersePath) -> String {
    match verse_path {
        scripture_types::VersePath::PathBoM(a, b, c) => format!("VersePath::PathBoM({},{},{})", a, b, c),
        scripture_types::VersePath::PathOT(a, b, c) => format!("VersePath::PathOT({},{},{})", a, b, c),
        scripture_types::VersePath::PathNT(a, b, c) => format!("VersePath::PathNT({},{},{})", a, b, c),
        scripture_types::VersePath::PathPOGP(a, b, c) => format!("VersePath::PathPOGP({},{},{})", a, b, c),
        scripture_types::VersePath::PathDC(a, b) => format!("VersePath::PathDC({},{})", a, b),
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
    dest.push(format!("{}.bin", file_name));

    let mut f = BufWriter::new(File::create(dest).unwrap());
    f.write(&bincode::serialize(&parsed).unwrap()).unwrap();
    f.flush().unwrap();

    parsed
}

pub fn write_minified<T: serde::ser::Serialize>(
    data: &T,
    dest_folder: &std::path::PathBuf,
    file_name: &str,
) -> () {
    let mut dest = dest_folder.clone();
    dest.push(format!("{}.bin", file_name));

    println!("writing {}", file_name);

    let mut f = BufWriter::new(File::create(dest).unwrap());
    f.write(&bincode::serialize(&data).unwrap()).unwrap();
    f.flush().unwrap();
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

    let mut paths_index_codegen_file = dest_folder.clone();
    paths_index_codegen_file.push("codegen-paths-index.rs");

    println!("generating paths index codegen file...");

    let mut paths_index_phf: phf_codegen::Map<u16> = phf_codegen::Map::new();
    for (k, v) in &paths_index {
        paths_index_phf.entry(*k, &repr_verse_path(v));
    }

    println!("writing paths index codegen file...");

    let mut f_codegen = BufWriter::new(File::create(paths_index_codegen_file).unwrap());

    writeln!(
        &mut f_codegen,
        "pub static PHF_PATHS_INDEX: phf::Map<u16, scripture_types::VersePath> = \n{};\n",
        paths_index_phf.build(),
    ).unwrap();




    let mut words_index_codegen_file = dest_folder.clone();
    words_index_codegen_file.push("codegen-words-index.rs");

    println!("generating words index codegen file...");

    let mut words_index_phf: phf_codegen::Map<&str> = phf_codegen::Map::new();
    for (word, usage_map) in &words_index {
        let mut usages_phf: phf_codegen::Map<u16> = phf_codegen::Map::new();
        for (scripture_id, highlights_vec) in usage_map {
            let (i_s, l_s): (Vec<_>, Vec<_>) = highlights_vec.iter().cloned().map(|(x,y)| (x as u16, y as u8)).unzip();
            usages_phf.entry(
                *scripture_id,
                &format!(
                    "(U256 {{ 0: {:?} }},{})",
                    data_bundler::pack_indices_arr(&i_s),
                    data_bundler::pack_lengths(&l_s),
                ),
            );
        }
        let built_usages_phf = usages_phf.build();
        words_index_phf.entry(word, &built_usages_phf.to_string());
    }


    println!("writing words index codegen file...");

    let mut f_codegen_words_index = BufWriter::new(File::create(words_index_codegen_file).unwrap());

    writeln!(
        &mut f_codegen_words_index,
        "pub static PHF_WORDS_INDEX: phf::Map<&str, phf::Map<u16, (U256,u128)>> = \n{};\n",
        words_index_phf.build(),
    ).unwrap();
}
