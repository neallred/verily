import * as React from 'react';
import { debounce } from './utils';
import BookPreferences, {
  Bookronym,
  FIRST_DC_SECTION,
  LAST_DC_SECTION,
  SectionNumbers,
} from './BookPreferences';

const bookOrders = {
  ot: [
    "Genesis",
    "Exodus",
    "Leviticus",
    "Numbers",
    "Deuteronomy",
    "Joshua",
    "Judges",
    "Ruth",
    "1 Samuel",
    "2 Samuel",
    "1 Kings",
    "2 Kings",
    "1 Chronicles",
    "2 Chronicles",
    "Ezra",
    "Nehemiah",
    "Esther",
    "Job",
    "Psalms",
    "Proverbs",
    "Ecclesiastes",
    "Solomon's Song",
    "Isaiah",
    "Jeremiah",
    "Lamentations",
    "Ezekiel",
    "Daniel",
    "Hosea",
    "Joel",
    "Amos",
    "Obadiah",
    "Jonah",
    "Micah",
    "Nahum",
    "Habakkuk",
    "Zephaniah",
    "Haggai",
    "Zechariah",
    "Malachi",
  ],
  nt: [
    "Matthew",
    "Mark",
    "Luke",
    "John",
    "Acts",
    "Romans",
    "1 Corinthians",
    "2 Corinthians",
    "Galatians",
    "Ephesians",
    "Philippians",
    "Colossians",
    "1 Thessalonians",
    "2 Thessalonians",
    "1 Timothy",
    "2 Timothy",
    "Titus",
    "Philemon",
    "Hebrews",
    "James",
    "1 Peter",
    "2 Peter",
    "1 John",
    "2 John",
    "3 John",
    "Jude",
    "Revelation",
  ],
  bom: [
    "1 Nephi",
    "2 Nephi",
    "Jacob",
    "Enos",
    "Jarom",
    "Omni",
    "Words of Mormon",
    "Mosiah",
    "Alma",
    "Helaman",
    "3 Nephi",
    "4 Nephi",
    "Mormon",
    "Ether",
    "Moroni",
  ],
  pogp: [
    "Moses",
    "Abraham",
    "Joseph Smith—Matthew",
    "Joseph Smith—History",
    "Articles of Faith",
  ],
}

const defaultRange: [SectionNumbers, SectionNumbers] = [FIRST_DC_SECTION, LAST_DC_SECTION];

export interface SearchMaterials {
  includeSource: {
    ot: boolean;
    nt: boolean;
    bom: boolean;
    dc: boolean;
    pogp: boolean;
  }
  ot: {
    "Genesis": boolean;
    "Exodus": boolean;
    "Leviticus": boolean;
    "Numbers": boolean;
    "Deuteronomy": boolean;
    "Joshua": boolean;
    "Judges": boolean;
    "Ruth": boolean;
    "1 Samuel": boolean;
    "2 Samuel": boolean;
    "1 Kings": boolean;
    "2 Kings": boolean;
    "1 Chronicles": boolean;
    "2 Chronicles": boolean;
    "Ezra": boolean;
    "Nehemiah": boolean;
    "Esther": boolean;
    "Job": boolean;
    "Psalms": boolean;
    "Proverbs": boolean;
    "Ecclesiastes": boolean;
    "Solomon's Song": boolean;
    "Isaiah": boolean;
    "Jeremiah": boolean;
    "Lamentations": boolean;
    "Ezekiel": boolean;
    "Daniel": boolean;
    "Hosea": boolean;
    "Joel": boolean;
    "Amos": boolean;
    "Obadiah": boolean;
    "Jonah": boolean;
    "Micah": boolean;
    "Nahum": boolean;
    "Habakkuk": boolean;
    "Zephaniah": boolean;
    "Haggai": boolean;
    "Zechariah": boolean;
    "Malachi": boolean;
  }
  nt: {
    "Matthew": boolean;
    "Mark": boolean;
    "Luke": boolean;
    "John": boolean;
    "Acts": boolean;
    "Romans": boolean;
    "1 Corinthians": boolean;
    "2 Corinthians": boolean;
    "Galatians": boolean;
    "Ephesians": boolean;
    "Philippians": boolean;
    "Colossians": boolean;
    "1 Thessalonians": boolean;
    "2 Thessalonians": boolean;
    "1 Timothy": boolean;
    "2 Timothy": boolean;
    "Titus": boolean;
    "Philemon": boolean;
    "Hebrews": boolean;
    "James": boolean;
    "1 Peter": boolean;
    "2 Peter": boolean;
    "1 John": boolean;
    "2 John": boolean;
    "3 John": boolean;
    "Jude": boolean;
    "Revelation": boolean;
  }
  bom: {
    "1 Nephi": boolean;
    "2 Nephi": boolean;
    "Jacob": boolean;
    "Enos": boolean;
    "Jarom": boolean;
    "Omni": boolean;
    "Words of Mormon": boolean;
    "Mosiah": boolean;
    "Alma": boolean;
    "Helaman": boolean;
    "3 Nephi": boolean;
    "4 Nephi": boolean;
    "Mormon": boolean;
    "Ether": boolean;
    "Moroni": boolean;
  }
  dc: {
    range: [SectionNumbers, SectionNumbers];
  }
  pogp: {
    "Moses": boolean;
    "Abraham": boolean;
    "Joseph Smith—Matthew": boolean;
    "Joseph Smith—History": boolean;
    "Articles of Faith": boolean;
  }
};

export interface SearchPreferences {
  and: boolean;
  or: boolean;
  caseSensitive: boolean;
  exact: boolean;
  toSearch: SearchMaterials;
}

const defaultSearchMaterial = {
  includeSource: {
    ot: true,
    nt: true,
    bom: true,
    dc: true,
    pogp: true,
  },
  ot: {
    "Genesis": true,
    "Exodus": true,
    "Leviticus": true,
    "Numbers": true,
    "Deuteronomy": true,
    "Joshua": true,
    "Judges": true,
    "Ruth": true,
    "1 Samuel": true,
    "2 Samuel": true,
    "1 Kings": true,
    "2 Kings": true,
    "1 Chronicles": true,
    "2 Chronicles": true,
    "Ezra": true,
    "Nehemiah": true,
    "Esther": true,
    "Job": true,
    "Psalms": true,
    "Proverbs": true,
    "Ecclesiastes": true,
    "Solomon's Song": true,
    "Isaiah": true,
    "Jeremiah": true,
    "Lamentations": true,
    "Ezekiel": true,
    "Daniel": true,
    "Hosea": true,
    "Joel": true,
    "Amos": true,
    "Obadiah": true,
    "Jonah": true,
    "Micah": true,
    "Nahum": true,
    "Habakkuk": true,
    "Zephaniah": true,
    "Haggai": true,
    "Zechariah": true,
    "Malachi": true,
  },
  nt: {
    "Matthew": true,
    "Mark": true,
    "Luke": true,
    "John": true,
    "Acts": true,
    "Romans": true,
    "1 Corinthians": true,
    "2 Corinthians": true,
    "Galatians": true,
    "Ephesians": true,
    "Philippians": true,
    "Colossians": true,
    "1 Thessalonians": true,
    "2 Thessalonians": true,
    "1 Timothy": true,
    "2 Timothy": true,
    "Titus": true,
    "Philemon": true,
    "Hebrews": true,
    "James": true,
    "1 Peter": true,
    "2 Peter": true,
    "1 John": true,
    "2 John": true,
    "3 John": true,
    "Jude": true,
    "Revelation": true,
  },
  bom: {
    "1 Nephi": true,
    "2 Nephi": true,
    "Jacob": true,
    "Enos": true,
    "Jarom": true,
    "Omni": true,
    "Words of Mormon": true,
    "Mosiah": true,
    "Alma": true,
    "Helaman": true,
    "3 Nephi": true,
    "4 Nephi": true,
    "Mormon": true,
    "Ether": true,
    "Moroni": true,
  },
  dc: {
    range: defaultRange,
  },
  pogp: {
    "Moses": true,
    "Abraham": true,
    "Joseph Smith—Matthew": true,
    "Joseph Smith—History": true,
    "Articles of Faith": true,
  },
};

const defaultPreferences: SearchPreferences = {
  and: true,
  or: false,
  caseSensitive: false,
  exact: true,
  toSearch: defaultSearchMaterial,
};

interface PreferencesProps {
  preferences: SearchPreferences;
  setPreferences: (preferences: SearchPreferences) => void;
  hidePreferences: () => void;
}

function mergeConfigs<T>(a: T, b: {}, c: any = {}): T {
  return Object.entries(a).reduce((acc, [k, v]) => {
    if (
      !Object.prototype.hasOwnProperty.call(b, k) ||
      (typeof (a as any)[k] !== typeof (b as any)[k])
    ) {
      c[k] = (a as any)[k];
    } else if (typeof v === 'object') {
      c[k] = Object.prototype.hasOwnProperty.call(v, 'length')
        ? (b as any)[k]
        : mergeConfigs(v, (b as any)[k]);
    } else {
      // strings, booleans, numbers
      c[k] = (b as any)[k];
    }

    return acc;
  }, c) as T
}

export function loadPreferences(): SearchPreferences {
  const savedPreferences = JSON.parse(localStorage.getItem('scripturedPreferences')) as SearchPreferences;

  if (savedPreferences) {
    const mergedConfigs = mergeConfigs(defaultPreferences, savedPreferences);
    return mergedConfigs
  } else {
    return defaultPreferences
  }
}

function savePreferences(preferences: SearchPreferences) {
  localStorage.setItem('scripturedPreferences', JSON.stringify(preferences));
}

const debouncedSavePreferences = debounce(savePreferences);

function deepSet(path: string[], value: any, obj: any, merge: boolean = false): any {
  const k = path[0];
  if (!k) {
    return obj;
  }
  return {
    ...obj,
    [k]: (path.length === 1)
        ? (merge ? ({ ...obj[k], ...value }): value)
        : deepSet(path.slice(1), value, obj[k], merge)
  };
}

export default function Preferences({
  preferences,
  setPreferences,
  hidePreferences,
}: PreferencesProps) {
  React.useEffect(() => {
    debouncedSavePreferences(preferences);
  }, [preferences]);

  const setPathValue = React.useCallback((path: string[], value: boolean | string | number) => {
    setPreferences(deepSet(['toSearch'].concat(path), value, preferences));
  }, [preferences]);

  const setAll = React.useCallback((bookronym: Bookronym, allValue: boolean, min?: number, max?: number) => {
    const retVal: any = {};
    setPreferences({
      ...preferences,
      toSearch: {
        ...preferences.toSearch,
        [bookronym]: (
          bookronym === 'dc'
            ? { range: [min, (allValue ? max : min)] }
            : Object.keys(preferences.toSearch[bookronym]).reduce((acc, curr) => { acc[curr] = allValue; return acc; }, retVal)
        ),
        includeSource: {
          ...preferences.toSearch.includeSource,
        },
      },
    });
  }, [preferences]);
  return <div
    onClick={hidePreferences}
    style={{
      position: 'fixed',
      top: '0',
      bottom: '0',
      left: '0',
      right: '0',
      backgroundColor: 'rgba(0,0,0,0.2)',
      overflowY: 'auto',
    }}
  >
    <div
      onClick={e => e?.stopPropagation()}
      style={{
        width: '320px',
        backgroundColor: 'white',
        position: 'relative',
        margin: '10px auto 0',
        padding: '10px',
        fontSize: '18px',
      }}
    >
      {/* <div>
        And search: <input type="checkbox" checked={preferences.and} onChange={e => setPreferences({...preferences, and: e.target.checked})} />
      </div>
      <div>
        Case sensitive search: <input type="checkbox" checked={preferences.caseSensitive} onChange={e => setPreferences({...preferences, caseSensitive: e.target.checked})} />
      </div>*/}
      <BookPreferences
        bookronym="ot"
        title="Old Testament"
        includeSource={preferences.toSearch.includeSource.ot}
        bookOrder={bookOrders.ot}
        booksIncluded={preferences.toSearch.ot}
        numberRange={undefined}
        setPathValue={setPathValue}
        setAll={setAll}
      />
      <BookPreferences
        bookronym="nt"
        title="New Testament"
        includeSource={preferences.toSearch.includeSource.nt}
        bookOrder={bookOrders.nt}
        booksIncluded={preferences.toSearch.nt}
        numberRange={undefined}
        setPathValue={setPathValue}
        setAll={setAll}
      />
      <BookPreferences
        bookronym="bom"
        title="Book of Mormon"
        includeSource={preferences.toSearch.includeSource.bom}
        bookOrder={bookOrders.bom}
        booksIncluded={preferences.toSearch.bom}
        numberRange={undefined}
        setPathValue={setPathValue}
        setAll={setAll}
      />
      <BookPreferences
        bookronym="dc"
        title="Doctrine and Covenants"
        includeSource={preferences.toSearch.includeSource.dc}
        numberRange={preferences.toSearch.dc.range}
        min={FIRST_DC_SECTION}
        max={LAST_DC_SECTION}
        setPathValue={setPathValue}
        setAll={setAll}
      />
      <BookPreferences
        bookronym="pogp"
        title="Pearl of Great Price"
        includeSource={preferences.toSearch.includeSource.pogp}
        bookOrder={bookOrders.pogp}
        booksIncluded={preferences.toSearch.pogp}
        numberRange={undefined}
        setPathValue={setPathValue}
        setAll={setAll}
      />
      <button 
        style={{
          position: 'absolute',
          top: '-9px',
          right: '-9px',
          padding: '4px 7px',
          borderRadius: '100%',
          fontWeight: 800,
        }}
        onClick={hidePreferences}
      >&#10799;</button>
    </div>
  </div>
}
