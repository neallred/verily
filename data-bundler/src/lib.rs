extern crate scripture_types;
extern crate rust_stemmers;
use rust_stemmers::{Algorithm, Stemmer};
use fnv::FnvHashMap;
use std::collections::hash_map::Entry;
use scripture_types::{
    OldTestament,
    NewTestament,
    BookOfMormon,
    DoctrineAndCovenants,
    PearlOfGreatPrice,
    Verse,
    Chapter,
    WordsIndex,
    PathsIndex,
    VersePath,
};

pub enum HasBooks<'a> {
    OT(&'a OldTestament),
    NT(&'a NewTestament),
    BOM(&'a BookOfMormon),
    POGP(&'a PearlOfGreatPrice),
}
use primitive_types::U256;

fn prepare_book_paths<'a>(coll: HasBooks<'a>) -> Vec<(u8, u8, &'a Verse)> {
    let (books, title) = match coll {
        HasBooks::OT(x) => (&x.books, &x.title),
        HasBooks::NT(x) => (&x.books, &x.title),
        HasBooks::BOM(x) => (&x.books, &x.title),
        HasBooks::POGP(x) => (&x.books, &x.title),
    };
    println!("    {}", title);
    let with_books: Vec<(u8, &Chapter)> = books
        .iter()
        .enumerate()
        .flat_map(|(book_num, book)| {
            let with_books: Vec<(u8, &Chapter)> =
                book.chapters.iter().map(|cs| (book_num as u8, cs)).collect();

            with_books
        })
        .collect();

    let with_chapters = with_books
        .iter()
        .flat_map(|(book_num, chapter)| {
            let with_verses: Vec<(u8, u8, &Verse)> = chapter
                .verses
                .iter()
                .map(|v| (*book_num, chapter.chapter - 1, v))
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

// unsafe
// each highlight length (u8) must be no bigger than 5 bits
pub fn pack_lengths(lengths: &Vec<u8>) -> u128 {
    let mut result: u128 = 0;
    for i in lengths {
        if result > 0 {
            result = result << 5;
        }
        result = result + *i as u128
    }
    result
}

pub fn unpack_lengths(packed: u128) -> Vec<u8> {
    let mut tmp = packed.clone();
    let mut result: Vec<u8> = vec![];

    while tmp > 0 {
        let diff = (tmp >> 5) << 5;
        result.push((tmp - diff) as u8);
        tmp = diff >> 5;
    }
    result.iter().map(|x| *x).rev().collect()
}

// unsafe
// u16s must be no bigger than 11 bits - 1 (need each chunk to have a min value of 1) for efficient
// unpacking
pub fn pack_indices(indices: &Vec<u16>) -> U256 {
    // offset all by one.
    let mut result: U256 = U256::from(0);
    for i in indices {
        if !result.is_zero() {
            result = result << 11;
        }
        result = result + U256::from(*i + 1);
    }
    result
}

pub fn pack_indices_arr(indices: &Vec<u16>) -> [u64;4] {
    // offset all by one.
    let mut num: U256 = U256::from(0);
    for i in indices {
        if !num.is_zero() {
            num = num << 11;
        }
        num = num + U256::from(*i + 1);
    }

    let mut num_arr: [u64;4] = [0,0,0,0];

    let new_num = num >> 64 << 64;
    num_arr[0] = (num - new_num).as_u64();
    num = new_num;

    let new_num = num >> 64 << 64;
    num_arr[1] = (num - new_num).as_u64();
    num = new_num;

    let new_num = num >> 64 << 64;
    num_arr[2] = (num - new_num).as_u64();
    num = new_num;

    let new_num = num >> 64 << 64;
    num_arr[3] = (num - new_num).as_u64();

    num_arr
}

pub fn unpack_indices(packed: U256) -> Vec<u16> {
    // offset all by one.
    let mut tmp = packed.clone();
    let mut result: Vec<u16> = vec![];

    while !tmp.is_zero() {
        let diff = (tmp >> 11) << 11;
        result.push(((tmp - diff) - 1).as_u32() as u16);
        tmp = diff >> 11;
    }
    result.iter().map(|x| *x).rev().collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    fn test_round_trip(lenghts: Vec<u8>) {
        assert_eq!(lenghts, unpack_lengths(pack_lengths(&lenghts)));
    }

    fn test_round_trip_indices(indices: Vec<u16>) {
        assert_eq!(indices, unpack_indices(pack_indices(&indices)));
    }

    #[test]
    fn packs_and_unpacks_highlight_lengths_less_than_32_up_to_22_elements() {
        test_round_trip(vec![]);
        test_round_trip((1..22).collect());
        test_round_trip((10..31).collect());
    }

    #[test]
    fn packs_and_unpacks_highlight_indices_less_than_2048_up_to_22_elements() {
        test_round_trip_indices(vec![]);
        test_round_trip_indices(vec![1234,2046,0,1,2,3,4,5,88]);
        test_round_trip_indices((0..21).collect());
        test_round_trip_indices((2025..2046).collect());
    }
}

pub fn build_index(
    ot: &OldTestament,
    nt: &NewTestament,
    bom: &BookOfMormon,
    dc: &DoctrineAndCovenants,
    pogp: &PearlOfGreatPrice,
) -> (WordsIndex, PathsIndex) {
    let mut scripture_id: u16 = 0;

    let en_stemmer = Stemmer::create(Algorithm::English);

    let indices: (WordsIndex, PathsIndex) =
        (FnvHashMap::default(), FnvHashMap::default());

    let count_word_usage = |mut words_index: WordsIndex, verse: &String, (i_from, i_to): &(usize, usize), scripture_id: u16| {
        let f = *i_from;
        let t = *i_to;

        let i = *i_from;
        let l = t - i;

        let word_slice = &verse[f..t].to_lowercase();
        let stemmed = en_stemmer.stem(word_slice).to_string();
        let to_insert: Vec<(usize, usize)> = vec![(i, l)];

        match words_index.entry(stemmed) {
            Entry::Vacant(vacant) => {
                let mut verses_using_word = FnvHashMap::default();
                verses_using_word.insert(scripture_id, to_insert);

                vacant.insert(verses_using_word);
            },
            Entry::Occupied(mut verses_using_word) => {
                let verses_using_word_val = verses_using_word.get_mut();
                let verse_usage_entry = verses_using_word_val.entry(scripture_id);
                verse_usage_entry
                    .and_modify(|verse_usage| { verse_usage.push((i,l)); })
                    .or_insert(to_insert);
            },
        };

        words_index
    };

    let count_verse = |verse_text: &String, words_index: WordsIndex, id| {
        let index_with_verse_added = get_word_ranges(verse_text)
            .iter()
            .fold(words_index, |acc, word_indices| count_word_usage(acc, verse_text, word_indices, id));
        index_with_verse_added
    };

    // Old Testament
    let indices = prepare_book_paths(HasBooks::OT(ot)).iter().fold(
        indices,
        |(words_indices, mut path_index), (book_num, chapter_num, verse)| {
            scripture_id += 1;
            path_index.insert(
                scripture_id,
                VersePath::PathOT(
                    *book_num,
                    *chapter_num,
                    verse.verse - 1,
                ),
            );
            (
                count_verse(&verse.text, words_indices, scripture_id),
                path_index,
            )
        },
    );

    // New Testament
    let indices = prepare_book_paths(HasBooks::NT(nt)).iter().fold(
        indices,
        |(words_indices, mut path_index), (book_num, chapter_num, verse)| {
            scripture_id += 1;
            path_index.insert(
                scripture_id,
                VersePath::PathNT(
                    *book_num,
                    *chapter_num,
                    verse.verse - 1,
                ),
            );
            (
                count_verse(&verse.text, words_indices, scripture_id),
                path_index,
            )
        },
    );

    // Book of Mormon
    let indices = prepare_book_paths(HasBooks::BOM(bom)).iter().fold(
        indices,
        |(words_indices, mut path_index), (book_num, chapter_num, verse)| {
            scripture_id += 1;
            path_index.insert(
                scripture_id,
                VersePath::PathBoM(
                    *book_num,
                    *chapter_num,
                    verse.verse - 1,
                ),
            );
            (
                count_verse(&verse.text, words_indices, scripture_id),
                path_index,
            )
        },
    );

    // Doctrine and Covenants
    println!("    {}", dc.title);
    let with_section_nums: Vec<(usize, &Verse)> = (dc)
        .sections
        .iter()
        .flat_map(|section| {
            let with_section_nums: Vec<(usize, &Verse)> = section
                .verses
                .iter()
                .map(|v| (section.section as usize - 1, v))
                .collect();

            with_section_nums
        })
        .collect();

    let indices = with_section_nums.iter().fold(
        indices,
        |(words_indices, mut path_index), (section_num, verse)| {
            scripture_id += 1;
            path_index.insert(
                scripture_id,
                VersePath::PathDC(*section_num as u8, verse.verse - 1),
            );
            (
                count_verse(&verse.text, words_indices, scripture_id),
                path_index,
            )
        },
    );

    // Pearl of Great Price
    let indices = prepare_book_paths(HasBooks::POGP(pogp)).iter().fold(
        indices,
        |(words_indices, mut path_index), (book_num, chapter_num, verse)| {
            scripture_id += 1;
            path_index.insert(
                scripture_id,
                VersePath::PathPOGP(*book_num, *chapter_num, verse.verse - 1),
            );
            (
                count_verse(&verse.text, words_indices, scripture_id),
                path_index,
            )
        },
    );

    (indices.0, indices.1)
}
