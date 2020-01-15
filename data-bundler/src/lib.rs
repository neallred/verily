extern crate scripture_types;
extern crate rust_stemmers;
use rust_stemmers::{Algorithm, Stemmer};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

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

fn get_word_ranges(text: &String) -> Vec<(usize, usize)> {
    let mut results: Vec<(usize, usize)> = vec![];
    let mut open: Option<usize> = None;
    text.char_indices().for_each(|(idx, letter): (usize, char)| {
        let is_word_char = letter.is_alphanumeric() || letter == 'æ' || letter == '-';
        if is_word_char {
            if open == None {
                open = Some(idx);
            }
        } else {
            if let Some(open_idx) = open {
                results.push((open_idx, idx));
                open = None;
            }
        }
    });

    results
}

pub fn build_index(
    ot: &scripture_types::OldTestament,
    nt: &scripture_types::NewTestament,
    bom: &scripture_types::BookOfMormon,
    dc: &scripture_types::DoctrineAndCovenants,
    pogp: &scripture_types::PearlOfGreatPrice,
) -> (scripture_types::WordsIndex, scripture_types::PathsIndex) {
    let mut scripture_id: u32 = 0;

    let en_stemmer = Stemmer::create(Algorithm::English);

    let indices: (scripture_types::WordsIndex, scripture_types::PathsIndex) =
        (HashMap::new(), HashMap::new());

    let count_word_usage = |mut words_index: scripture_types::WordsIndex, verse: &String, (i_from, i_to): &(usize, usize), scripture_id: u32| {
        let f = *i_from;
        let t = *i_to;
        let word_slice = &verse[f..t].to_lowercase();
        let stemmed = en_stemmer.stem(word_slice).to_string();
        let to_insert = vec![(f, t)];

        match words_index.entry(stemmed) {
            Entry::Vacant(vacant) => {
                let mut verses_using_word = HashMap::new();
                verses_using_word.insert(scripture_id, to_insert);
                // verses_using_word

                vacant.insert(verses_using_word);
            },
            Entry::Occupied(mut verses_using_word) => {
                let verses_using_word_val = verses_using_word.get_mut();
                let verse_usage_entry = verses_using_word_val.entry(scripture_id);
                verse_usage_entry
                    .and_modify(|verse_usage| { verse_usage.push((f,t)) })
                    .or_insert(to_insert);
            },
        };

        words_index
    };

    let count_verse = |verse_text: &String, words_index: scripture_types::WordsIndex, id| {
        let index_with_verse_added = get_word_ranges(verse_text)
            .iter()
            .fold(words_index, |acc, word_indices| count_word_usage(acc, verse_text, word_indices, id));
        index_with_verse_added
    };

    // Old Testament
    let indices = prepare_book_paths(HasBooks::OT(ot)).iter().fold(
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
    let indices = prepare_book_paths(HasBooks::NT(nt)).iter().fold(
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
    let indices = prepare_book_paths(HasBooks::BOM(bom)).iter().fold(
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
    println!("    {}", dc.title);
    let with_section_nums: Vec<(usize, &scripture_types::Verse)> = (dc)
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
    let indices = prepare_book_paths(HasBooks::POGP(pogp)).iter().fold(
        indices,
        |(words_index, mut path_index), (book_num, chapter_num, verse)| {
            scripture_id += 1;
            path_index.insert(
                scripture_id,
                scripture_types::VersePath::PathPOGP(*book_num, *chapter_num, verse.verse as usize - 1),
            );
            (
                count_verse(&verse.text, words_index, scripture_id),
                path_index,
            )
        },
    );

    indices
}
