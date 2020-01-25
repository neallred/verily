extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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
    pub dc: (u8, u8),
    pub pogp: Vec<String>,
}

pub fn make_empty_preferences() -> SearchPreferences {
    SearchPreferences {
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
            ot: vec![String::from("Genesis")],
            nt: vec![String::from("Matthew")],
            bom: vec![String::from("1 Nephi")],
            dc: (1, 20),
            pogp: vec![String::from("Abraham")],
        },
    }
}

pub fn check_can_search(search: &String, preferences: &SearchPreferences) -> bool {
    if search.is_empty() {
        return false;
    }

    let included_sources = preferences.included_sources;
    if !included_sources.ot
        && !included_sources.nt
        && !included_sources.bom
        && !included_sources.dc
        && !included_sources.pogp
    {
        return false;
    }

    let included_books = &preferences.included_books;

    ((included_sources.ot && !included_books.ot.is_empty())
        || (included_sources.nt && !included_books.nt.is_empty())
        || (included_sources.bom && !included_books.bom.is_empty())
        || (included_sources.dc && included_books.dc.1 > included_books.dc.0)
        || (included_sources.pogp && !included_books.pogp.is_empty()))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_search_empty_blank_string_is_false() {
        let preferences = SearchPreferences {
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
                bom: vec![String::from("Alma")],
                dc: (1, 1),
                pogp: vec![],
            },
        };
        let search = String::from("");
        assert_eq!(false, check_can_search(&search, &preferences));
    }

    #[test]
    fn check_search_empty_no_sources_is_false() {
        let preferences = SearchPreferences {
            and: false,
            case_sensitive: true,
            exact: false,
            included_sources: IncludedSources {
                ot: false,
                nt: false,
                bom: false,
                dc: false,
                pogp: false,
            },
            included_books: IncludedBooks {
                ot: vec![],
                nt: vec![],
                bom: vec![String::from("Alma")],
                dc: (1, 1),
                pogp: vec![],
            },
        };
        let search = String::from("asdf");
        assert_eq!(false, check_can_search(&search, &preferences));
    }

    #[test]
    fn check_search_empty_no_books_is_false() {
        let preferences = SearchPreferences {
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
        let search = String::from("asdf");
        assert_eq!(false, check_can_search(&search, &preferences));
    }

    #[test]
    fn check_search_empty_with_search_sources_and_books_is_true() {
        let preferences = SearchPreferences {
            and: false,
            case_sensitive: true,
            exact: false,
            included_sources: IncludedSources {
                ot: false,
                nt: false,
                bom: true,
                dc: true,
                pogp: false,
            },
            included_books: IncludedBooks {
                ot: vec![],
                nt: vec![],
                bom: vec![String::from("Alma")],
                dc: (1, 1),
                pogp: vec![],
            },
        };
        let search = String::from("asdf");
        assert_eq!(true, check_can_search(&search, &preferences));
    }
}
