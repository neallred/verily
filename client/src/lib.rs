mod utils;

extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate scripture_types;
extern crate rust_stemmers;
use rust_stemmers::{Algorithm, Stemmer};
use regex::Regex;

#[macro_use]
extern crate lazy_static;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use flate2::read::GzDecoder;
use std::io::prelude::*;
use std::collections::HashSet;
// use std::collections::HashMap;
// use std::error::Error;

extern crate web_sys;
use web_sys::console;
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

static BYTES_WORDS_INDEX: &'static [u8] = include_bytes!("../../data-bundler/data/words-index.json.gz");
static BYTES_PATHS_INDEX: &'static [u8] = include_bytes!("../../data-bundler/data/paths-index.json.gz");

// TODO: Figure out to do this one, at compile time.
lazy_static! {
    static ref BOOK_OF_MORMON: scripture_types::BookOfMormon = parse_gzip(&BYTES_BOOK_OF_MORMON);
    static ref OLD_TESTAMENT: scripture_types::OldTestament = parse_gzip(&BYTES_OLD_TESTAMENT);
    static ref NEW_TESTAMENT: scripture_types::NewTestament = parse_gzip(&BYTES_NEW_TESTAMENT);
    static ref PEARL_OF_GREAT_PRICE: scripture_types::PearlOfGreatPrice = parse_gzip(&BYTES_PEARL_OF_GREAT_PRICE);
    static ref DOCTRINE_AND_COVENANTS: scripture_types::DoctrineAndCovenants = parse_gzip(&BYTES_DOCTRINE_AND_COVENANTS);
    static ref WORDS_INDEX: scripture_types::WordsIndex = parse_gzip(&BYTES_WORDS_INDEX);
    static ref PATHS_INDEX: scripture_types::PathsIndex = parse_gzip(&BYTES_PATHS_INDEX);
    static ref STEMMER: rust_stemmers::Stemmer = Stemmer::create(Algorithm::English);
    static ref RE_VERSE_CHARS: Regex = Regex::new(r"[^A-Za-z0-9\sæ\-]").unwrap();
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
    log!("words: {:?}", WORDS_INDEX.len());
    log!("paths: {:?}", PATHS_INDEX.len());
}

fn make_splittable(text: &String) -> String {
    let with_substitutions = text
        .replace("–", " ")
        .replace("—", " ")
        .replace("—", " ")
        .replace("'s", "")
        .to_lowercase();
    let splittable = RE_VERSE_CHARS.replace_all(&with_substitutions, "");
    splittable.to_string() 
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
    let empty = HashSet::new();
    let empty_forever = HashSet::new();

    let stemmed_search = STEMMER.stem(search_term);

    let bob = || {
        let result = make_splittable(search_term).split_whitespace().fold(
            empty,
            |mut acc, word| {
                let index_results = match (&*WORDS_INDEX).get(word) {
                    Some(x) => x,
                    None => &empty_forever,
                };
                index_results
            }
        );
        result
    };
    let index_results = bob();

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
