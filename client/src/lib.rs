mod utils;

extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate scripture_types;

#[macro_use]
extern crate lazy_static;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use flate2::read::GzDecoder;
use std::io::prelude::*;
use std::collections::HashSet;
use std::collections::HashMap;
// use std::error::Error;
extern crate rust_stemmers;
use rust_stemmers::{Algorithm, Stemmer};


extern crate web_sys;
use web_sys::console;
use regex::Regex;
macro_rules! log {
    ( $( $t:tt )* ) => {
        console::log_1(&format!( $( $t )* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchPreferences {
  pub and: bool,
  #[serde(rename = "caseSensitive")]
  pub case_sensitive: bool,
  pub exact: bool,
  #[serde(rename = "includedSources")]
  pub included_sources: IncludedSources,
  #[serde(rename = "includedBooks")]
  pub included_books: IncludedBooks,
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct IncludedSources {
    pub ot: bool,
    pub nt: bool,
    pub bom: bool,
    pub dc: bool,
    pub pogp: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IncludedBooks {
    // TODO: Represent these as `HashSet`s
    pub ot: Vec<String>,
    pub nt: Vec<String>,
    pub bom: Vec<String>,
    pub dc: (u64, u64),
    pub pogp: Vec<String>,
}

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static BYTES_OLD_TESTAMENT: &'static [u8] = include_bytes!("../../data-bundler/data/old-testament.json.gz");
static BYTES_NEW_TESTAMENT: &'static [u8] = include_bytes!("../../data-bundler/data/new-testament.json.gz");
static BYTES_BOOK_OF_MORMON: &'static [u8] = include_bytes!("../../data-bundler/data/book-of-mormon.json.gz");
static BYTES_DOCTRINE_AND_COVENANTS: &'static [u8] = include_bytes!("../../data-bundler/data/doctrine-and-covenants.json.gz");
static BYTES_PEARL_OF_GREAT_PRICE: &'static [u8] = include_bytes!("../../data-bundler/data/pearl-of-great-price.json.gz");

// TODO: Figure out to do this one, at compile time.
lazy_static! {
    static ref BOOK_OF_MORMON: scripture_types::BookOfMormon = parse_gzip(&BYTES_BOOK_OF_MORMON);
    static ref OLD_TESTAMENT: scripture_types::OldTestament = parse_gzip(&BYTES_OLD_TESTAMENT);
    static ref NEW_TESTAMENT: scripture_types::NewTestament = parse_gzip(&BYTES_NEW_TESTAMENT);
    static ref PEARL_OF_GREAT_PRICE: scripture_types::PearlOfGreatPrice = parse_gzip(&BYTES_PEARL_OF_GREAT_PRICE);
    static ref DOCTRINE_AND_COVENANTS: scripture_types::DoctrineAndCovenants = parse_gzip(&BYTES_DOCTRINE_AND_COVENANTS);
    static ref IDX: (HashMap<String, HashSet<u32>>, HashMap<u32, VersePath>) = build_index();
}

type WordsIndex = HashMap<String, HashSet<u32>>;
type PathsIndex = HashMap<u32, VersePath>;

pub enum HasBooks {
    OT(&'static scripture_types::OldTestament),
    NT(&'static scripture_types::NewTestament),
    BOM(&'static scripture_types::BookOfMormon),
    POGP(&'static scripture_types::PearlOfGreatPrice),
}

fn prepare_book_paths(coll: HasBooks) -> Vec<(&'static String, u8, &'static scripture_types::Verse)> {
    let books = match coll {
        HasBooks::OT(x) => &x.books,
        HasBooks::NT(x) => &x.books,
        HasBooks::BOM(x) => &x.books,
        HasBooks::POGP(x) => &x.books,
    };
    let with_books: Vec<(&String, &scripture_types::Chapter)> = books.iter()
        .flat_map(|book| {
            let with_books: Vec<(&String, &scripture_types::Chapter)> = book.chapters.iter().map(|cs| (&book.book, cs)).collect();

            with_books
        })
        .collect();

    let with_chapters = with_books.iter()
        .flat_map(|(title, chapter)| {
            let with_verses: Vec<(&String, u8, &scripture_types::Verse)> = chapter.verses.iter().map(|v| (*title, chapter.chapter as u8, v)).collect();

            with_verses
        })
        .collect();


    with_chapters
}

fn build_index() -> (WordsIndex, PathsIndex) {
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

    let indices: (WordsIndex, PathsIndex) = (HashMap::new(), HashMap::new());

    let count_word_usage = |mut words_index: WordsIndex, word: &str, id| {
        let stemmed = en_stemmer.stem(word);

        words_index.insert(
            stemmed.to_string(),
            match words_index.get(&stemmed.to_string()) {
                Some(x) => {
                    let mut verses_using_word = x.clone();
                    verses_using_word.insert(id);
                    verses_using_word
                },
                None => {
                    let mut verses_using_word = HashSet::new();
                    verses_using_word.insert(id);
                    verses_using_word
                },
            }
        );
        words_index
    };

    let count_verse = |verse_text: &String, words_index: WordsIndex, id| {
        let index_with_verse_added = make_splittable(verse_text)
            .split_whitespace()
            .fold(
                words_index,
                |acc, word| count_word_usage(acc, word, id) 
            );
        index_with_verse_added
    };

    // Doctrine and Covenants
    let with_section_nums: Vec<(u8, &scripture_types::Verse)> = (&*DOCTRINE_AND_COVENANTS).sections.iter()
        .flat_map(|section| {
            let with_section_nums: Vec<(u8, &scripture_types::Verse)> = section.verses.iter().map(|v| (section.section as u8, v)).collect();

            with_section_nums
        })
        .collect();

    let indices = with_section_nums.iter()
        .fold(
            indices,
            |(words_index, mut path_index), (section_num, verse)| {
                scripture_id += 1;
                path_index.insert(scripture_id, VersePath::PathDC(*section_num, verse.verse as u8));
                (count_verse(&verse.text, words_index, scripture_id), path_index)
            }
        );

    // Old Testament
    let indices = prepare_book_paths(HasBooks::OT(&*OLD_TESTAMENT)).iter()
        .fold(
            indices,
            |(words_index, mut path_index), (title, chapter_num, verse)| {
                scripture_id += 1;
                path_index.insert(scripture_id, VersePath::PathOT(title.to_string(), *chapter_num, verse.verse as u8));
                (count_verse(&verse.text, words_index, scripture_id), path_index)
            }
        );

    // New Testament
    let indices = prepare_book_paths(HasBooks::NT(&*NEW_TESTAMENT)).iter()
        .fold(
            indices,
            |(words_index, mut path_index), (title, chapter_num, verse)| {
                scripture_id += 1;
                path_index.insert(scripture_id, VersePath::PathNT(title.to_string(), *chapter_num, verse.verse as u8));
                (count_verse(&verse.text, words_index, scripture_id), path_index)
            }
        );

    // Book of Mormon
    let indices = prepare_book_paths(HasBooks::BOM(&*BOOK_OF_MORMON)).iter()
        .fold(
            indices,
            |(words_index, mut path_index), (title, chapter_num, verse)| {
                scripture_id += 1;
                path_index.insert(scripture_id, VersePath::PathBoM(title.to_string(), *chapter_num, verse.verse as u8));
                (count_verse(&verse.text, words_index, scripture_id), path_index)
            }
        );

    // Pearl of Great Price
    let indices = prepare_book_paths(HasBooks::POGP(&*PEARL_OF_GREAT_PRICE)).iter()
        .fold(
            indices,
            |(words_index, mut path_index), (title, chapter_num, verse)| {
                scripture_id += 1;
                path_index.insert(scripture_id, VersePath::PathPOGP(title.to_string(), *chapter_num, verse.verse as u8));
                (count_verse(&verse.text, words_index, scripture_id), path_index)
            }
        );

    indices
}

// enum AndOr {
//     And = 1,
//     Or = 0,
// }
#[derive(Debug)]
pub enum VersePath {
    PathBoM(String, u8, u8),
    PathOT(String, u8, u8),
    PathNT(String, u8, u8),
    PathPOGP(String, u8, u8),
    PathDC(u8, u8), // section verse
}

fn format_verse(v: &scripture_types::Verse) -> String {
    format!("{}: {}", &v.reference, &v.text)
}

fn inclusive_contains(x: u64, bounds: (u64, u64)) -> bool {
    x >= bounds.0 && x <= bounds.1
}

pub fn parse_gzip<T: serde::de::DeserializeOwned + serde::ser::Serialize>(
    gzipped: &[u8]
) -> T {

    let mut d = GzDecoder::new(gzipped);
    let mut s = String::new();
    d.read_to_string(&mut s).unwrap();

    let data: T = serde_json::from_str(&s).unwrap();
    data
}

#[wasm_bindgen]
pub fn bootstrap_searcher() {
    // Force the minimal amount of work to initialize all data structures
    // so that user searches are speedy.
    let empty_preferences = SearchPreferences {
        and: false,
        case_sensitive: true,
        exact: false,
        included_sources: IncludedSources {
            ot: true,
            nt: true,
            bom: true,
            dc: true,
            pogp: true,
        },
        included_books: IncludedBooks {
            ot: vec![],
            nt: vec![],
            bom: vec![],
            dc: (1, 1),
            pogp: vec![],
        },
    };
    full_match_search(
        String::from("BOOSTRAP SEARCHER BOOSTRAP SEARCHER BOOSTRAP SEARCHER"),
        JsValue::from_serde(&empty_preferences).unwrap(),
    );
    log!("words: {:?}", IDX.0.len());
    log!("paths: {:?}", IDX.1.len());
}

#[wasm_bindgen]
pub fn full_match_search(search_term_raw: String, search_preferences_js: JsValue) -> JsValue {
    let search_preferences: SearchPreferences = search_preferences_js.into_serde().unwrap();
    let search_term = &search_term_raw.to_lowercase();
    let case_sensitive_match = |verse: &&scripture_types::Verse| verse.text.contains(&search_term_raw);
    let case_insensitive_match = |verse: &&scripture_types::Verse| verse.text.to_lowercase().contains(search_term);

    let verse_search: Box<dyn Fn(&&scripture_types::Verse) -> bool> = if search_preferences.case_sensitive {
        Box::new(case_sensitive_match)
    } else {
        Box::new(case_insensitive_match)
    };

    let mut results: Vec<String> = vec![];

    if search_preferences.included_sources.ot {
        let mut ot_results: Vec<String> = (&*OLD_TESTAMENT).books.iter()
            .filter(|book| search_preferences.included_books.ot.contains(&book.book))
            .flat_map(|book| &book.chapters)
            .flat_map(|chapter| &chapter.verses)
            .filter(&verse_search)
            .map(format_verse).collect();

        results.append(&mut ot_results);
    }

    if search_preferences.included_sources.nt {
        let mut nt_results: Vec<String> = (&*NEW_TESTAMENT).books.iter()
            .filter(|book| search_preferences.included_books.nt.contains(&book.book))
            .flat_map(|book| &book.chapters)
            .flat_map(|chapter| &chapter.verses)
            .filter(&verse_search)
            .map(format_verse).collect();
        results.append(&mut nt_results);
    }

    if search_preferences.included_sources.bom {
        let mut bom_results: Vec<String> = (&*BOOK_OF_MORMON).books.iter()
            .filter(|book| search_preferences.included_books.bom.contains(&book.book))
            .flat_map(|book| &book.chapters)
            .flat_map(|chapter| &chapter.verses)
            .filter(&verse_search)
            .map(format_verse).collect();
        results.append(&mut bom_results);
    }

    if search_preferences.included_sources.dc {
        let mut dc_results: Vec<String> = (&*DOCTRINE_AND_COVENANTS).sections.iter()
            .filter(|section| inclusive_contains(section.section, search_preferences.included_books.dc))
            .flat_map(|section| &section.verses)
            .filter(&verse_search)
            .map(format_verse).collect();
        results.append(&mut dc_results);
    }

    if search_preferences.included_sources.pogp {
        let mut pogp_results: Vec<String> = (&*PEARL_OF_GREAT_PRICE).books.iter()
            .filter(|book| search_preferences.included_books.pogp.contains(&book.book))
            .flat_map(|book| &book.chapters)
            .flat_map(|chapter| &chapter.verses)
            .filter(&verse_search)
            .map(format_verse).collect();
        results.append(&mut pogp_results);
    }

    JsValue::from_serde(&results).unwrap()
}
