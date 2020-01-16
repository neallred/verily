import * as React from 'react';

import GearSvg from './GearSvg';

export type Bookronym = "ot" | "nt" | "bom" | "dc" | "pogp";

export type SectionNumbers = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 80 | 81 | 82 | 83 | 84 | 85 | 86 | 87 | 88 | 89 | 90 | 91 | 92 | 93 | 94 | 95 | 96 | 97 | 98 | 99 | 100 | 101 | 102 | 103 | 104 | 105 | 106 | 107 | 108 | 109 | 110 | 111 | 112 | 113 | 114 | 115 | 116 | 117 | 118 | 119 | 120 | 121 | 122 | 123 | 124 | 125 | 126 | 127 | 128 | 129 | 130 | 131 | 132 | 133 | 134 | 135 | 136 | 137 | 138;

export const FIRST_DC_SECTION: SectionNumbers = 1;
export const LAST_DC_SECTION: SectionNumbers = 138;

interface BookPreferencesProps {
  bookOrder?: string[];
  bookronym: Bookronym;
  booksIncluded?: {[key: string]: boolean};
  includeSource: boolean;
  max?: number,
  min?: number,
  numberRange?: [number, number];
  setAll: (bookronym: Bookronym, includeAll: boolean, min?: number, max?: number) => void,
  setPathValue: Function;
  title: string;
}

function getSelectedCount(
  includeSource: boolean,
  allIncluded: boolean,
  booksIncluded: {[key: string]: boolean} | null,
  min: number,
  max: number,
): string {
  if (!includeSource) {
    return '';
  }

  if (allIncluded) {
    return '(all)'
  }

  return booksIncluded
    ? (`(${Object.values(booksIncluded).filter(x => x).length})` || '')
    : `(${min} - ${max})`;
}

export default function BookPreferences({
  bookOrder,
  bookronym,
  booksIncluded,
  includeSource,
  max,
  min,
  numberRange,
  setAll,
  setPathValue,
  title,
}: BookPreferencesProps) {
  const [open, setOpen] = React.useState(false);
  const [currentMin, currentMax] = numberRange || [FIRST_DC_SECTION, LAST_DC_SECTION];
  const allIncluded = (
    (bookOrder && bookOrder.every(x => booksIncluded[x])) ||
    (numberRange && numberRange[0] === min && numberRange[1] === max)
  );
  const setSlider = React.useCallback((e) => {
    const updatingMin = e.target.id.endsWith('-min');
    const updatedValue = parseInt(e.target.value);
    const newMin = updatingMin ? updatedValue : Math.min(currentMin, updatedValue);
    const newMax = updatingMin ? Math.max(currentMax, updatedValue) : updatedValue;
    setPathValue(['dc', 'range'], [newMin, newMax])
  }, [currentMin, currentMax, setPathValue]);
  const selectedCount = React.useMemo(
    () => getSelectedCount(includeSource, allIncluded, booksIncluded, currentMin, currentMax),
    [includeSource, allIncluded, booksIncluded, currentMin, currentMax],
  );

  const pivot = Math.floor((currentMax + currentMin) / 2);

  return <div style={{
      paddingTop: '20px',
    }}>
    <div style={{
      display: 'flex',
      alignItems: 'center',
    }}>
      <span style={{
        textDecoration: includeSource ? 'none' : 'line-through',
        color: includeSource ? '#000000' : '#888888',
        width: '222px',
      }}>{title} {selectedCount}</span>
      <GearSvg size={30} onClick={() => setOpen(!open)} style={{
        padding: '0 5px',
        marginRight: '9px',
      }}/>
      <button
        onClick={() => setPathValue(['includeSource', bookronym], !includeSource)}
        style={{
          color: '#000000',
          textDecoration: 'none',
          fontSize: '18px',
          width: '50px',
          padding: '1px 2px',
          backgroundColor: includeSource ? 'white' : '#cefcce',
        }}
      >{includeSource ? 'Hide' : 'Use'}</button>
    </div>
    {open && <div>
      <button onClick={() => setAll(bookronym, !allIncluded, min, max)}>{allIncluded ? `Exclude all` : `Use all`}</button>
      {bookOrder && booksIncluded
        ?  bookOrder.map(x => {
          return <div key={x} 
            style={{
              marginTop: '8px',
            }}
          >
            <label style={{
              display: 'flex',
              alignItems: 'center',
            }}>
              <input
                type="checkbox"
                checked={booksIncluded[x]}
                onChange={e => setPathValue([bookronym, x], e.target.checked)}
                style={{
                  margin: '0',
                  marginRight: '8px',
                }}
              />
              {x}
            </label>


          </div>
        })
        : null
      }
      { numberRange
          ?
            <div className="range-container"
              style={{
                marginTop: '8px',
              }}
            >

              <input
                type="range"
                id={`${bookronym}-min`}
                min={min}
                max={Math.min(max, Math.max(pivot, currentMin + 1))}
                step={1}
                style={{
                  flexGrow: Math.max(1, pivot - min), 
                }}
                value={currentMin}
                onChange={setSlider}
              />
              <input
                type="range"
                id={`${bookronym}-max`}
                min={Math.max(min, Math.min(pivot, currentMax - 1))}
                max={max}
                step={1}
                style={{
                  flexGrow: Math.max(1, max - pivot), 
                }}
                value={currentMax}
                onChange={setSlider}
              />
            </div>
          : null
      }
    </div>
    }
  </div>
}
