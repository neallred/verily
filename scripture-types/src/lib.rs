extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

pub type WordsIndex = HashMap<String, HashSet<u32>>;
pub type PathsIndex = HashMap<u32, VersePath>;

#[derive(Serialize, Deserialize, Debug)]
pub enum VersePath {
    PathBoM(usize, usize, usize),
    PathOT(usize, usize, usize),
    PathNT(usize, usize, usize),
    PathPOGP(usize, usize, usize),
    PathDC(usize, usize), // section verse
}

#[derive(Serialize, Deserialize)]
pub struct Verse {
    pub heading: Option<String>,
    pub pilcrow: Option<bool>,
    pub reference: String,
    pub subheading: Option<String>,
    pub text: String,
    pub verse: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Chapter {
    pub chapter: u64,
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
    pub section: u64,
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
    pub version: u64,
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
    pub version: u64,
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
    pub version: u64,
}

#[derive(Serialize, Deserialize)]
pub struct OldTestament {
    pub books: Vec<Book>,
    pub last_modified: String,
    pub lds_slug: String,
    pub the_end: String,
    pub title: String,
    pub version: u64,
}

#[derive(Serialize, Deserialize)]
pub struct PearlOfGreatPrice {
    pub books: Vec<Book>,
    pub last_modified: String,
    pub subtitle: String,
    pub title: String,
    pub version: u64,
}
