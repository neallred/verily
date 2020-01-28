extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate phf;
use primitive_types::U256;

use serde::{Deserialize, Serialize};
use fnv::FnvHashMap;

pub type WordsIndex = FnvHashMap<String, FnvHashMap<u16, Vec<(usize, usize)>>>;
pub type PathsIndex = FnvHashMap<u16, VersePath>;
pub type VersePathsIndex = FnvHashMap<VersePath, u16>;
pub type PhfPathsIndex = phf::Map<u16, VersePath>;
pub type PhfWordsIndex = phf::Map<&'static str, phf::Map<u16, (U256, u128)>>;

pub fn paths_to_verse_paths_index(paths: &PhfPathsIndex) -> VersePathsIndex {
    paths
        .entries()
        .fold(
            FnvHashMap::default(),
            |mut acc, (k, v)| {
                acc.insert(v.clone(), *k);
                acc
            }
        )
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Clone)]
pub enum VersePath {
    PathBoM(u8, u8, u16),
    PathOT(u8, u8, u16),
    PathNT(u8, u8, u16),
    PathPOGP(u8, u8, u16),
    PathDC(u8, u16), // section verse
}

#[derive(Serialize, Deserialize)]
pub struct Verse {
    pub heading: Option<String>,
    pub pilcrow: Option<bool>,
    pub reference: String,
    pub subheading: Option<String>,
    pub text: String,
    pub verse: u16,
}

#[derive(Serialize, Deserialize)]
pub struct Chapter {
    pub chapter: u8,
    pub heading: Option<String>,
    pub note: Option<String>,
    pub reference: String,
    pub verses: Vec<Verse>,
}

#[derive(Serialize, Deserialize)]
pub struct Facsimile {
    explanations: Vec<String>,
    image_url: String,
    lds_slug: String,
    note: Option<String>,
    number: u64,
    title: String,
}

#[derive(Serialize, Deserialize)]
pub struct Book {
    pub book: String,
    pub chapters: Vec<Chapter>,
    pub facsimiles: Option<Vec<Facsimile>>,
    pub full_subtitle: Option<String>,
    pub full_title: String,
    pub heading: Option<String>,
    pub lds_slug: String,
    pub note: Option<String>,
}

// structs
#[derive(Serialize, Deserialize)]
pub struct Section {
    pub section: u8,
    pub reference: String,
    pub verses: Vec<Verse>,
    pub signature: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct BookOfMormon {
    pub books: Vec<Book>,
    pub last_modified: String,
    pub lds_slug: String,
    pub subtitle: String,
    pub testimonies: Vec<Testimony>,
    pub title: String,
    pub title_page: TitlePage,
    pub version: u8,
}

#[derive(Serialize, Deserialize)]
pub struct TitlePage {
    pub subtitle: String,
    pub text: Vec<String>,
    pub title: String,
    pub translated_by: String,
}

#[derive(Serialize, Deserialize)]
pub struct Testimony {
    text: String,
    title: String,
    witnesses: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DoctrineAndCovenants {
    pub last_modified: String,
    pub lds_slug: String,
    pub sections: Vec<Section>,
    pub subsubtitle: String,
    pub subtitle: String,
    pub title: String,
    pub version: u8,
}

#[derive(Serialize, Deserialize)]
pub struct NewTestamentTitlePage {
    subtitle: String,
    text: String,
    title: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewTestament {
    pub books: Vec<Book>,
    pub last_modified: String,
    pub lds_slug: String,
    pub title: String,
    pub title_page: NewTestamentTitlePage,
    pub version: u8,
}

#[derive(Serialize, Deserialize)]
pub struct OldTestament {
    pub books: Vec<Book>,
    pub last_modified: String,
    pub lds_slug: String,
    pub the_end: String,
    pub title: String,
    pub version: u8,
}

#[derive(Serialize, Deserialize)]
pub struct PearlOfGreatPrice {
    pub books: Vec<Book>,
    pub last_modified: String,
    pub lds_slug: String,
    pub subtitle: String,
    pub title: String,
    pub version: u8,
}
