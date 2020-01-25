extern crate serde;
extern crate serde_derive;
extern crate serde_json;
extern crate phf;

use serde::{Deserialize, Serialize};
use fnv::FnvHashMap;

#[derive(Serialize, Deserialize, Debug, Hash, Clone)]
pub enum ArrWrap {
    A1([(u16, u8); 1]),
    A2([(u16, u8); 2]),
    A3([(u16, u8); 3]),
    A4([(u16, u8); 4]),
    A5([(u16, u8); 5]),
    A6([(u16, u8); 6]),
    A7([(u16, u8); 7]),
    A8([(u16, u8); 8]),
    A9([(u16, u8); 9]),
    A10([(u16, u8); 10]),
    A11([(u16, u8); 11]),
    A12([(u16, u8); 12]),
    A13([(u16, u8); 13]),
    A14([(u16, u8); 14]),
    A15([(u16, u8); 15]),
    A16([(u16, u8); 16]),
    A17([(u16, u8); 17]),
    A18([(u16, u8); 18]),
    A19([(u16, u8); 19]),
    A20([(u16, u8); 20]),
    A21([(u16, u8); 21]),
    A22([(u16, u8); 22]),
//     The longest number of highlights is A22.
//     The biggest number of highlight index is 1168.
//     Exclude all other possibilities to take up the least amount of memory possible
//     (Need this to run on low end devices).
//     A23([(u16, u8); 23]),
//     A24([(u16, u8); 24]),
//     A25([(u16, u8); 25]),
//     A26([(u16, u8); 26]),
//     A27([(u16, u8); 27]),
//     A28([(u16, u8); 28]),
//     A29([(u16, u8); 29]),
//     A30([(u16, u8); 30]),
//     A31([(u16, u8); 31]),
//     A32([(u16, u8); 32]),
}

pub type WordsIndex = FnvHashMap<String, FnvHashMap<u16, FnvHashMap<usize, usize>>>;
pub type PathsIndex = FnvHashMap<u16, VersePath>;
pub type PhfPathsIndex = phf::Map<u16, VersePath>;
pub type VersePathsIndex = FnvHashMap<VersePath, u16>;

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
