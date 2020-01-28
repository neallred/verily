import * as React from 'react';

import Preferences, { SearchPreferences } from './Preferences';
import GearSvg from './GearSvg';

interface FormProps {
  searchTerm: string;
  setSearchTerm: (searchTerm: string) => void;
  preferences: SearchPreferences;
  setPreferences: (preferences: SearchPreferences) => void;
  resultCount: null | number,
};

export default function Form({
  searchTerm,
  setSearchTerm,
  preferences,
  setPreferences,
  resultCount,
}: FormProps) {
  const [preferencesOpen, setPreferencesOpen] = React.useState(false);
  const showPreferences = React.useCallback(() => setPreferencesOpen(true), []);
  const hidePreferences = React.useCallback(() => setPreferencesOpen(false), []);
  const inputRef = React.useRef(null);
  React.useEffect(() => {
    setTimeout(() => {
      if (inputRef.current) {
        inputRef.current.focus();
      }
    });
  }, []);

  return <div>
    <div style={{
      display: 'flex',
      alignItems: 'center',
      margin: '0 auto 20px',
      maxWidth: '520px',
      position: 'relative'
    }}>
      <input
        placeholder="Search scriptures"
        value={searchTerm}
        onChange={e => setSearchTerm(e.target.value)}
        ref={inputRef}
        style={{
          flex: '1 1 200px',
          maxWidth: 'calc(100% - 47px)',
        }}
      />
      <GearSvg
        size={30}
        onClick={showPreferences}
        style={{
          marginLeft: '5px',
          padding: '2px',
          flex: '0 0 auto',
        }}
      />
      { false && typeof resultCount === 'number' && <div style={{
        // position: 'absolute',
        // left: '100%',
        // whiteSpace: 'nowrap',
        // alignSelf: 'center',
      }}>
        {resultCount} results
      </div>}
    </div>
      { typeof resultCount === 'number' && <div style={{
        // position: 'absolute',
        // left: '100%',
        // whiteSpace: 'nowrap',
        // alignSelf: 'center',
      }}>
        {resultCount} results
      </div>}
    {preferencesOpen &&
      <Preferences
        preferences={preferences}
        setPreferences={setPreferences}
        hidePreferences={hidePreferences}
      />
    }
  </div>
}
