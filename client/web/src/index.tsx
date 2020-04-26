import * as ReactDOM from 'react-dom';
import * as React from 'react';

import * as wasm from "wasm-scriptured-client";
wasm.set_panic_hook();

import Form from './Form';
import { loadPreferences, SearchPreferences } from './Preferences';
import overtake from './overtake';
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

const _elementCache: {[key: string]: HTMLElement | null} = {};
function cachedGetElementById(id: string) { 
  const hit = _elementCache[id];
  if (hit) {
    return hit;
  }
  _elementCache[id] = document.getElementById(id);
  return _elementCache[id];
}

let BOOTSTRAP_WAIT = 5000;
const SHORTEST_SEARCH_LENGTH = 2;

interface CounterRef {
  previewPath: string;
  clicks: number;
  timeout: number;
}
const counter: CounterRef = {
  previewPath: '',
  clicks: 0,
  timeout: 0,
};

const inputTagName = 'LI'

function resetPreview() {
  if (counter.timeout) {
    window.clearTimeout(counter.timeout);
  }
  counter.previewPath = '';
  counter.clicks = 0;
  counter.timeout = 0;
}

function previewListener(e: MouseEvent) {
  let itemEl;
  if ((e as any).target.tagName === inputTagName) {
    itemEl = e.target;
  } else if (((e as any).target.parentNode && (e as any).target.parentNode.tagName) === inputTagName) {
    itemEl = (e.target as any).parentNode;
  }
  if (!itemEl) {
    return
  }
  const previewPath = itemEl.dataset.versePath;
  if (previewPath  === counter.previewPath) {
    counter.clicks += 1
  } else {
    counter.previewPath = previewPath;
    counter.clicks = 1;
  }
  if (counter.clicks >= 4) {
    overtake(wasm.get_chapter_preview(previewPath))
  };

  if (counter.timeout) {
    window.clearTimeout(counter.timeout);
  }
  const timeout = setTimeout(resetPreview, 4000)
  counter.timeout = timeout as unknown as number;
}

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
        <ul id="scriptured-results" className="results-section" onClick={previewListener as any} />
      </div>
    case Bootstrapped.Fail:
      return <div className="fail">
        Failed to bootstrap searcher.
      </div>
  }
}

ReactDOM.render(<App />, document.getElementById('scriptured-root'));
