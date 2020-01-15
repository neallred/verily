import * as ReactDOM from 'react-dom';
import * as React from 'react';

import * as wasm from "wasm-scriptured-client";
wasm.set_panic_hook();

import NoResult from './NoResult';
import Result from './Result';
import Form from './Form';
import { loadPreferences, SearchPreferences } from './Preferences';
import { debounce } from './utils';

interface AppProps {
}

enum Bootstrapped {
  Y,
  N,
  Fail
}

function reduceString(acc: string[], [k, v]: [string, boolean]): string[] {
  if (v) {
    acc.push(k);
  }
  return acc;
}

function jsPreferencesToWasmPreferences(jsPreferences: SearchPreferences): any {
  const {
    caseSensitive,
    exact,
    toSearch,
  } = jsPreferences;
  return {
    and: jsPreferences.and,
    caseSensitive,
    exact,
    includedSources: toSearch.includeSource,
    includedBooks: {
      ot: Object.entries(toSearch.ot).reduce(reduceString, []),
      nt: Object.entries(toSearch.nt).reduce(reduceString, []),
      bom: Object.entries(toSearch.bom).reduce(reduceString, []),
      pogp: Object.entries(toSearch.pogp).reduce(reduceString, []),
      dc: toSearch.dc.range
    },
  };
}

//       {results.length
//         ? results.map(x => <Result key={x} displayString={x} />)
//         : <NoResult
//           plausibleSearch={searchTerm.length >= SHORTEST_SEARCH_LENGTH}
//           searchPending={searchPending}
//           searchTerm={searchTerm}
//         />
//       }
//     </ul>
const _elementCache: {[key: string]: HTMLElement | null} = {};
function cachedGetElementById(id: string) { 
  const hit = _elementCache[id];
  if (hit) {
    return hit;
  }
  _elementCache[id] = document.getElementById(id);
  return _elementCache[id];
}

const resultList = document.getElementById('scriptured-results');

let BOOTSTRAP_WAIT = 5000;
const noResults: string[] = [];
const SHORTEST_SEARCH_LENGTH = 2;
function App({}: AppProps) {
  const [searchTerm, setSearchTerm] = React.useState("");
  const [preferences, setPreferences] = React.useState(loadPreferences());
  const [searchPending, setSearchPending] = React.useState(false);
  const [resultCount, setResultCount] = React.useState<null | number>(null);
  const [bootstrapped, setBootstrapped] = React.useState<Bootstrapped>(Bootstrapped.N);
  const bootstrapTimeoutRef = React.useRef<number>(0);

  const debouncedFullTextSearch = React.useCallback(debounce((currentSearchTerm: string, preferences: SearchPreferences) => {
    if (bootstrapTimeoutRef.current) {
      ((window as any).cancelIdleCallback || window.clearTimeout)(bootstrapTimeoutRef.current);
    }

    const shouldSearch = currentSearchTerm.length >= SHORTEST_SEARCH_LENGTH;
    const newResults = shouldSearch
      ? wasm.full_match_search(currentSearchTerm, jsPreferencesToWasmPreferences(preferences as any))
      : [];
    setResultCount(shouldSearch ? newResults.length : null);
    setSearchPending(false);

    cachedGetElementById('scriptured-results').innerHTML = newResults.join('');

  }, 350), []);
  React.useEffect(() => {
    const timeoutMethod = ((window as any).requestIdleCallback || window.setTimeout);
    bootstrapTimeoutRef.current = timeoutMethod(
      () => {
        try {
          wasm.bootstrap_searcher();
          setBootstrapped(Bootstrapped.Y);
        } catch(e) {
          alert(e);
        }
      },
      (window as any).requestIdleCallback ? { timeout: BOOTSTRAP_WAIT } : BOOTSTRAP_WAIT,
    )
  }, []);

  React.useEffect(() => {
    if (bootstrapped !== Bootstrapped.Y) {
      return;
    }
    setSearchPending(true);
    debouncedFullTextSearch(
      searchTerm,
      preferences,
    );
  }, [bootstrapped, searchTerm, preferences]);

  const boundSetSearchTerm = React.useCallback(
    newTerm => {
      setSearchPending(true);
      setSearchTerm(newTerm);
    },
    []
  );

  switch (bootstrapped) {
    case Bootstrapped.N:
      return <div className="waiting">
        <span>Building search indices...</span>
      </div>;
    case Bootstrapped.Y:
      return <div style={{
      padding: '8px',
        }}>
        <Form
          searchTerm={searchTerm}
          setSearchTerm={boundSetSearchTerm}
          preferences={preferences}
          setPreferences={setPreferences}
          resultCount={resultCount}
        />
        <ul id="scriptured-results" className="results-section" />
      </div>
    case Bootstrapped.Fail:
      return <div className="fail">
        Failed to bootstrap searcher.
      </div>
  }
}

ReactDOM.render(<App />, document.getElementById('scriptured-root'));
