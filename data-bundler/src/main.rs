extern crate rust_stemmers;
extern crate scripture_types;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::path::Path;

use flate2::write::GzEncoder;
use flate2::Compression;
use regex::Regex;
use rust_stemmers::{Algorithm, Stemmer};
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::prelude::*;

#[cfg(windows)]
pub const NPM: &'static str = "npm.cmd";

#[cfg(not(windows))]
pub const NPM: &'static str = "npm";

pub enum HasBooks<'a> {
    OT(&'a scripture_types::OldTestament),
    NT(&'a scripture_types::NewTestament),
    BOM(&'a scripture_types::BookOfMormon),
    POGP(&'a scripture_types::PearlOfGreatPrice),
}

fn prepare_book_paths<'a>(coll: HasBooks<'a>) -> Vec<(usize, usize, &'a scripture_types::Verse)> {
    let (books, title) = match coll {
        HasBooks::OT(x) => (&x.books, &x.title),
        HasBooks::NT(x) => (&x.books, &x.title),
        HasBooks::BOM(x) => (&x.books, &x.title),
        HasBooks::POGP(x) => (&x.books, &x.title),
    };
    println!("    {}", title);
    let with_books: Vec<(usize, &scripture_types::Chapter)> = books
        .iter()
        .enumerate()
        .flat_map(|(book_num, book)| {
            let with_books: Vec<(usize, &scripture_types::Chapter)> =
                book.chapters.iter().map(|cs| (book_num, cs)).collect();

            with_books
        })
        .collect();

    let with_chapters = with_books
        .iter()
        .flat_map(|(book_num, chapter)| {
            let with_verses: Vec<(usize, usize, &scripture_types::Verse)> = chapter
                .verses
                .iter()
                .map(|v| (*book_num, chapter.chapter as usize - 1, v))
                .collect();

            with_verses
        })
        .collect();

    with_chapters
}

fn build_index(
    ot: scripture_types::OldTestament,
    nt: scripture_types::NewTestament,
    bom: scripture_types::BookOfMormon,
    dc: scripture_types::DoctrineAndCovenants,
    pogp: scripture_types::PearlOfGreatPrice,
) -> (scripture_types::WordsIndex, scripture_types::PathsIndex) {
    let mut scripture_id: u32 = 0;

    let re_verse_chars: Regex = Regex::new(r"[^A-Za-z0-9\sæ\-]").unwrap();
    let en_stemmer = Stemmer::create(Algorithm::English);

    let make_splittable = |text: &String| -> String {
        let with_substitutions = text
            .replace("–", " ")
            .replace("—", " ")
            .replace("—", " ")
            .replace("'s", "")
            .to_lowercase();
        let splittable = re_verse_chars.replace_all(&with_substitutions, "");
        splittable.to_string()
    };

    let indices: (scripture_types::WordsIndex, scripture_types::PathsIndex) =
        (HashMap::new(), HashMap::new());

    let count_word_usage = |mut words_index: scripture_types::WordsIndex, word: &str, id| {
        let stemmed = en_stemmer.stem(word);

        words_index.insert(
            stemmed.to_string(),
            match words_index.get(&stemmed.to_string()) {
                Some(x) => {
                    let mut verses_using_word = x.clone();
                    verses_using_word.insert(id);
                    verses_using_word
                }
                None => {
                    let mut verses_using_word = HashSet::new();
                    verses_using_word.insert(id);
                    verses_using_word
                }
            },
        );
        words_index
    };

    let count_verse = |verse_text: &String, words_index: scripture_types::WordsIndex, id| {
        let index_with_verse_added = make_splittable(verse_text)
            .split_whitespace()
            .fold(words_index, |acc, word| count_word_usage(acc, word, id));
        index_with_verse_added
    };

    // Old Testament
    let indices = prepare_book_paths(HasBooks::OT(&ot)).iter().fold(
        indices,
        |(words_index, mut path_index), (book_num, chapter_num, verse)| {
            scripture_id += 1;
            path_index.insert(
                scripture_id,
                scripture_types::VersePath::PathOT(
                    *book_num,
                    *chapter_num,
                    verse.verse as usize - 1,
                ),
            );
            (
                count_verse(&verse.text, words_index, scripture_id),
                path_index,
            )
        },
    );

    // New Testament
    let indices = prepare_book_paths(HasBooks::NT(&nt)).iter().fold(
        indices,
        |(words_index, mut path_index), (book_num, chapter_num, verse)| {
            scripture_id += 1;
            path_index.insert(
                scripture_id,
                scripture_types::VersePath::PathNT(
                    *book_num,
                    *chapter_num,
                    verse.verse as usize - 1,
                ),
            );
            (
                count_verse(&verse.text, words_index, scripture_id),
                path_index,
            )
        },
    );

    // Book of Mormon
    let indices = prepare_book_paths(HasBooks::BOM(&bom)).iter().fold(
        indices,
        |(words_index, mut path_index), (book_num, chapter_num, verse)| {
            scripture_id += 1;
            path_index.insert(
                scripture_id,
                scripture_types::VersePath::PathBoM(
                    *book_num,
                    *chapter_num,
                    verse.verse as usize - 1,
                ),
            );
            (
                count_verse(&verse.text, words_index, scripture_id),
                path_index,
            )
        },
    );

    // Doctrine and Covenants
    println!("    {}", &dc.title);
    let with_section_nums: Vec<(usize, &scripture_types::Verse)> = (&dc)
        .sections
        .iter()
        .flat_map(|section| {
            let with_section_nums: Vec<(usize, &scripture_types::Verse)> = section
                .verses
                .iter()
                .map(|v| (section.section as usize - 1, v))
                .collect();

            with_section_nums
        })
        .collect();

    let indices = with_section_nums.iter().fold(
        indices,
        |(words_index, mut path_index), (section_num, verse)| {
            scripture_id += 1;
            path_index.insert(
                scripture_id,
                scripture_types::VersePath::PathDC(*section_num, verse.verse as usize - 1),
            );
            (
                count_verse(&verse.text, words_index, scripture_id),
                path_index,
            )
        },
    );

    // Pearl of Great Price
    let indices = prepare_book_paths(HasBooks::POGP(&pogp)).iter().fold(
        indices,
        |(words_index, mut path_index), (book_num, chapter_num, verse)| {
            scripture_id += 1;
            path_index.insert(
                scripture_id,
                scripture_types::VersePath::PathPOGP(*book_num, *chapter_num, verse.verse as usize),
            );
            (
                count_verse(&verse.text, words_index, scripture_id),
                path_index,
            )
        },
    );

    indices
}

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

    // TODO: is there a way to have a list of concrete values and types,
    // and iterate on each of them?
    // AFAIK, you can't mix term and type levels like that in Haskell.
    // let sources = vec![
    //     ("old-testament.json", scripture_types::OldTestament),
    // ];
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
    let (words_index, paths_index) = build_index(ot, nt, bom, dc, pogp);
    println!("Index building done!\n");
    println!("total word stems: {}", words_index.len());
    println!("total paths: {}", paths_index.len());
    write_minified(&words_index, &dest_folder, "words-index.json");
    write_minified(&paths_index, &dest_folder, "paths-index.json");
}
