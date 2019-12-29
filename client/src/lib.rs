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

fn build_index() -> (HashMap<String, HashSet<u32>>, HashMap<u32, VersePath>) {
    let zero: u32 = 0;
    let mut scripture_id: u32 = 0;

    let re_verse_chars = Regex::new(r"[^A-Za-z0-9\sæ\-]").unwrap();
    let en_stemmer = Stemmer::create(Algorithm::English);

    let dc_results: HashMap<String, HashSet<u32>> = (&*DOCTRINE_AND_COVENANTS).sections.iter()
        .flat_map(|section| &section.verses)
        .fold(
            HashMap::new(),
            |acc, verse| {
                scripture_id += 1;
                let replaced_text = verse
                    .text
                    .replace("–", " ")
                    .replace("—", " ")
                    .replace("—", " ")
                    .replace("'s", "")
                    .to_lowercase();

                let regged_text = re_verse_chars.replace_all(&replaced_text, "");


                regged_text 
                    .split_whitespace()
                    .fold(
                        acc,
                        |mut acc_inner, word| {
                            let stemmed = en_stemmer.stem(word);

                            acc_inner.insert(
                                stemmed.to_string(),
                                match acc_inner.get(&stemmed.to_string()) {
                                    Some(x) => {
                                        let mut bob = x.clone();
                                        bob.insert(scripture_id);
                                        bob
                                        // x
                                    }, // x + 1,
                                    None => {
                                        let mut bob = HashSet::new();
                                        bob.insert(scripture_id);
                                        bob
                                    },// .insert() zero + 1
                                }
                            );
                            acc_inner
                        }
                    )
            }
        );

    (dc_results, HashMap::new())

}

// enum AndOr {
//     And = 1,
//     Or = 0,
// }
enum VersePath {
    PathBoM,
    PathOT,
    PathNT,
    PathPOGP,
    PathDC(u16, u16), // section verse
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
    // log!("{:?}", (IDX.0));
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
