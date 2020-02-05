extern crate phf;
extern crate rust_stemmers;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;
use primitive_types::U256;
use regex::Regex;
use rust_stemmers::{Algorithm, Stemmer};

extern crate scripture_types;
extern crate data_bundler;

mod utils;
mod preferences;

#[macro_use]
extern crate lazy_static;

use scripture_types::{
    BookOfMormon, DoctrineAndCovenants, NewTestament, OldTestament, VersePathsIndex, PearlOfGreatPrice,
    VersePath
};
use fnv::FnvHashMap;
use fnv::FnvHashSet;
use wasm_bindgen::prelude::*;

extern crate web_sys;
use web_sys::console;
macro_rules! log {
    ( $( $t:tt )* ) => {
        console::log_1(&format!( $( $t )* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static BIN_OLD_TESTAMENT: &'static [u8] =
    include_bytes!("../../data-bundler/data/old-testament.json.bin");
static BIN_NEW_TESTAMENT: &'static [u8] =
    include_bytes!("../../data-bundler/data/new-testament.json.bin");
static BIN_BOOK_OF_MORMON: &'static [u8] =
    include_bytes!("../../data-bundler/data/book-of-mormon.json.bin");
static BIN_DOCTRINE_AND_COVENANTS: &'static [u8] =
    include_bytes!("../../data-bundler/data/doctrine-and-covenants.json.bin");
static BIN_PEARL_OF_GREAT_PRICE: &'static [u8] =
    include_bytes!("../../data-bundler/data/pearl-of-great-price.json.bin");

static BASE_URL: &'static str = "https://www.churchofjesuschrist.org/study/scriptures";
lazy_static! {
    static ref BOOK_OF_MORMON: BookOfMormon = adserde(BIN_BOOK_OF_MORMON);
    static ref OLD_TESTAMENT: OldTestament = adserde(BIN_OLD_TESTAMENT);
    static ref NEW_TESTAMENT: NewTestament = adserde(BIN_NEW_TESTAMENT);
    static ref DOCTRINE_AND_COVENANTS: DoctrineAndCovenants =
        adserde(BIN_DOCTRINE_AND_COVENANTS);
    static ref PEARL_OF_GREAT_PRICE: PearlOfGreatPrice = adserde(BIN_PEARL_OF_GREAT_PRICE);

    static ref VERSE_PATHS_INDEX: VersePathsIndex = scripture_types::paths_to_verse_paths_index(&indices::PHF_PATHS_INDEX);

    static ref STEMMER: rust_stemmers::Stemmer = Stemmer::create(Algorithm::English);
    static ref RE_VERSE_CHARS: Regex = Regex::new(r"[^A-Za-z0-9\sæ\-]").unwrap();
}

fn make_link(verse_path: &scripture_types::VersePath) -> String {
    let url_slug = match verse_path {
        VersePath::PathOT(b, c, v) => {
            let coll = &(&*OLD_TESTAMENT);

            let book = &coll.books[*b as usize];
            format!("{}/{}/{}.{}", coll.lds_slug, book.lds_slug, c + 1, v + 1)
        }
        VersePath::PathNT(b, c, v) => {
            let coll = &(&*NEW_TESTAMENT);

            let book = &coll.books[*b as usize];
            format!("{}/{}/{}.{}", coll.lds_slug, book.lds_slug, c + 1, v + 1)
        }
        VersePath::PathBoM(b, c, v) => {
            let coll = &(&*BOOK_OF_MORMON);

            let book = &coll.books[*b as usize];
            format!("{}/{}/{}.{}", coll.lds_slug, book.lds_slug, c + 1, v + 1)
        }
        VersePath::PathDC(s, v) => {
            let coll = &(&*DOCTRINE_AND_COVENANTS);

            format!("{}/{}.{}", coll.lds_slug, s + 1, v + 1)
        }
        VersePath::PathPOGP(b, c, v) => {
            let coll = &(&*PEARL_OF_GREAT_PRICE);

            let book = &coll.books[*b as usize];
            format!(
                "{}/{}/{}.{}?lang=eng",
                coll.lds_slug,
                book.lds_slug,
                c + 1,
                v + 1
            )
        }
    };
    format!("{}/{}", BASE_URL, url_slug)
}

fn extract_highlights((start_indices, lengths): &(U256, u128)) -> Vec<(u16, u8)> {
    data_bundler::unpack_indices(*start_indices).iter().cloned().zip(data_bundler::unpack_lengths(*lengths)).collect()
}

fn highlight_matches(text: &String, highlights: &Vec<(u16, u8)>) -> String {
    highlights
        .iter()
        .rev()
        .fold(text.to_string(), |mut acc, (from, to)| {
            let from_usize = *from as usize;
            let to_usize = from_usize + (*to as usize);
            // can't just use the slice because that
            // runs afoul of intended borrow checker usage.
            let word_to_replace = String::from(&acc[from_usize..to_usize]);
            acc.replace_range(
                &from_usize..&to_usize,
                &format!("<span class=\"match\">{}</span>", word_to_replace),
            );
            acc
        })
}

fn format_verse(
    p: &scripture_types::VersePath,
    v: &scripture_types::Verse,
    highlights: &Vec<(u16, u8)>,
) -> String {
    let mut sorted_highlights = highlights.clone();
    sorted_highlights.sort(); 
    format!(
        "<li><a target=\"_blank\" rel=\"noopener noreferrer\" href=\"{}\">{}</a>: {}</li>",
        make_link(p),
        &v.reference,
        highlight_matches(&v.text, &sorted_highlights),
    )
}

pub fn adserde<T: serde::de::DeserializeOwned + serde::ser::Serialize>(s: &'static [u8]) -> T {
    let t_0 = web_sys::window().unwrap().performance().unwrap().now();

    let data: T = bincode::deserialize(s).unwrap();
    let t_1 = web_sys::window().unwrap().performance().unwrap().now();
    log!("DATA LOAD: {:?}", t_1 - t_0);
    data
}

#[wasm_bindgen]
pub fn bootstrap_searcher() {
    let t_0 = web_sys::window().unwrap().performance().unwrap().now();
    let num_verse_paths = 
        (&*VERSE_PATHS_INDEX).len();
    let t_1 = web_sys::window().unwrap().performance().unwrap().now();
    log!("CALCULATING VERSE PATHS : {:?}", t_1 - t_0);

    log!(
        "words: {:?}, paths: {:?}, verse_paths: {:?}",
        (&indices::PHF_WORDS_INDEX).len(),
        (&indices::PHF_PATHS_INDEX).len(),
        num_verse_paths,
    );

    // Force the minimal amount of work to initialize all data structures
    // so that user searches are speedy.
    let empty_preferences = preferences::make_empty_preferences();
    full_match_search(
        // common words
        String::from("god and the faith"),
        JsValue::from_serde(&empty_preferences).unwrap(),
    );
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

pub fn resolve_verse_path(
    path: &VersePath,
    _preferences: &preferences::SearchPreferences,
) -> &'static scripture_types::Verse {
    match path {
        VersePath::PathOT(b, c, v) => &(&*OLD_TESTAMENT).books[*b as usize].chapters[*c as usize].verses[*v as usize],
        VersePath::PathNT(b, c, v) => &(&*NEW_TESTAMENT).books[*b as usize].chapters[*c as usize].verses[*v as usize],
        VersePath::PathBoM(b, c, v) => &(&*BOOK_OF_MORMON).books[*b as usize].chapters[*c as usize].verses[*v as usize],
        VersePath::PathDC(s, v) => &(&*DOCTRINE_AND_COVENANTS).sections[*s as usize].verses[*v as usize],
        VersePath::PathPOGP(b, c, v) => &(&*PEARL_OF_GREAT_PRICE).books[*b as usize].chapters[*c as usize].verses[*v as usize],
    }
}

fn check_collection_searchable(verse_path: &VersePath, preferences: &preferences::SearchPreferences) -> bool {
    let return_value = match verse_path {
        VersePath::PathOT(book, _, _) => (
            preferences.included_sources.ot &&
            preferences.included_books.ot.contains(&(&*OLD_TESTAMENT).books[*book as usize].book)
        ),
        VersePath::PathNT(book, _, _) => (
            preferences.included_sources.nt &&
            preferences.included_books.nt.contains(&(&*NEW_TESTAMENT).books[*book as usize].book)
        ),
        VersePath::PathBoM(book, _, _) => (
            preferences.included_sources.bom &&
            preferences.included_books.bom.contains(&(&*BOOK_OF_MORMON).books[*book as usize].book)
        ),
        VersePath::PathDC(sec, _) => (
            preferences.included_sources.dc &&
            sec >= &(preferences.included_books.dc.0) &&
            sec <= &(preferences.included_books.dc.1)
        ),
        VersePath::PathPOGP(book, _, _) => {
            let title = &(&*PEARL_OF_GREAT_PRICE).books[*book as usize].book;
            (
                preferences.included_sources.pogp &&
                preferences.included_books.pogp.contains(title)
            )
        },
    };

    return_value
}

pub type WordsIndexBorrowing = FnvHashMap<String, &'static phf::Map<u16, (U256,u128)>>;
#[wasm_bindgen]
pub fn full_match_search(search_term_raw: String, search_preferences_js: JsValue) -> JsValue {
    let t_0 = web_sys::window().unwrap().performance().unwrap().now();

    let search_preferences: preferences::SearchPreferences = search_preferences_js.into_serde().unwrap();
    if !preferences::check_can_search(&search_term_raw, &search_preferences) {
        let no_results: Vec<String> = vec![];
        return JsValue::from_serde(&no_results).unwrap();
    }

    log!("accessing paths index");
    let paths_index = &indices::PHF_PATHS_INDEX;

    let search_term = &make_splittable(&search_term_raw.to_lowercase());

    let search_stems: FnvHashSet<_> = search_term
        .split_whitespace()
        .map(|term| STEMMER.stem(term).to_string())
        .collect();

    let possible_matches: WordsIndexBorrowing =
        search_stems
            .iter()
            .fold(FnvHashMap::default(), |mut matching_index, term| {
                if let Some(v) = indices::PHF_WORDS_INDEX.get(term.as_str()) {
                    matching_index.insert(term.to_string(), v);
                }
                matching_index
            });

    let verse_paths_index = &*VERSE_PATHS_INDEX;

    log!("about to use paths index");
    let or_matches: FnvHashSet<u16> = possible_matches
        .iter()
        .flat_map(|(_k, v)| v.keys())
        .map(|x| *x)
        .filter(|x| check_collection_searchable(paths_index.get(x).unwrap(), &search_preferences))
        .collect();

    let and_matches: FnvHashSet<u16> = possible_matches.iter().fold(or_matches, |acc, (_k, v)| {
        let current_matches: FnvHashSet<u16> = v.keys().map(|x| *x).collect();
        let result: FnvHashSet<u16> = acc.intersection(&current_matches).map(|x| *x).collect();
        result
    });

    let mut verses: Vec<(u16, String)> = possible_matches
        .iter()
        .flat_map(|(_k, v)| v.entries().filter(|x| and_matches.contains(x.0)))
        .map(|(scripture_id, highlights)| {
            let verse_path = paths_index.get(scripture_id).unwrap();
            (verse_path, highlights)
        })
        .fold(
            FnvHashMap::default(),
            |mut acc: FnvHashMap<&VersePath, Vec<(u16, u8)>>, (verse_path, highlights)| {
                let mut highlights_vec = extract_highlights(&highlights);
                acc.entry(verse_path)
                    .and_modify(|existing_highlights| {
                        existing_highlights.append(&mut highlights_vec);
                    })
                    .or_insert(highlights_vec);
                acc
            },
        )
        .iter()
        .map(|(verse_path, highlights)| {
            let verse = resolve_verse_path(verse_path, &search_preferences);
            (verse_paths_index[&verse_path], format_verse(verse_path, verse, highlights))
        })
        .collect();
    verses.sort_unstable_by(|a, b| a.cmp(b));
    let sorted_verses: Vec<&String> = verses.iter().map(|(_, text)| text).collect();

    let t_1 = web_sys::window().unwrap().performance().unwrap().now();
    log!("search time: {:?}", t_1 - t_0);
    JsValue::from_serde(&sorted_verses).unwrap()
}

