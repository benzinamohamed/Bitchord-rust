import { Search, X } from 'lucide-react';
import { useCallback, useEffect, useId, useRef, useState } from 'react';
import { useNavigate } from 'react-router';
import { useSuggestions } from '@/data/music/queries';
import type { MusicItem } from '@/data/music/types';
import { Artwork } from './Artwork';
import { hrefFor } from './links';
import { playTrack, toTrack } from './play';
import { TrackMenu } from './TrackMenu';

type Option = { kind: 'query'; text: string; typed: string; rest: string } | { kind: 'item'; item: MusicItem };

/**
 * The centred search field and its suggestions (report §6): a pill field, and
 * a glass dropdown of completions, then artist, album and song suggestions.
 * Arrow keys move through it; Enter picks; Escape closes.
 */
export function SearchField({ initial, onSearch }: { initial: string; onSearch: (query: string) => void }) {
  const [value, setValue] = useState(initial);
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(-1);
  const input = useRef<HTMLInputElement>(null);
  const listId = useId();
  const navigate = useNavigate();
  const suggestions = useSuggestions(open ? value : '');
  // A suggestion's track menu takes focus when it opens (Menu focuses its
  // first item), which would otherwise read as the field blurring and close
  // the dropdown under the open menu. While a menu is open the blur close is
  // held off; when the menu closes, its pick is done, so the dropdown
  // closes with it.
  const menuOpen = useRef(false);
  const onMenuOpenChange = useCallback((isOpen: boolean) => {
    const wasOpen = menuOpen.current;
    menuOpen.current = isOpen;
    if (!isOpen && wasOpen) setOpen(false);
  }, []);

  useEffect(() => setValue(initial), [initial]);

  const options: Option[] = [
    ...(suggestions.data?.queries ?? []).map((q) => ({ kind: 'query' as const, ...q })),
    ...(suggestions.data?.items ?? []).map((item) => ({ kind: 'item' as const, item })),
  ];
  const showList = open && value.trim().length > 0 && options.length > 0;

  const pick = (option: Option | undefined) => {
    if (menuOpen.current) return;
    setOpen(false);
    setActive(-1);
    if (!option) return onSearch(value.trim());
    if (option.kind === 'query') {
      setValue(option.text);
      return onSearch(option.text);
    }
    const href = hrefFor(option.item);
    if (href) navigate(href);
    else playTrack(option.item);
  };

  return (
    <div className="relative w-full max-w-[680px]">
      <form
        className="relative flex items-center w-full"
        role="search"
        onSubmit={(e) => {
          e.preventDefault();
          pick(active >= 0 ? options[active] : undefined);
        }}
      >
        <Search size={16} strokeWidth={2.4} className="pointer-events-none absolute left-3.5 text-secondary" aria-hidden="true" />
        <input
          ref={input}
          className="h-[44px] w-full rounded-pill border border-search-border bg-page pt-1.5 pr-[36px] pb-[5px] pl-10 type-body text-primary placeholder:text-tertiary [&::-webkit-search-cancel-button]:hidden shadow-sm transition-shadow focus:shadow-md"
          type="search"
          value={value}
          placeholder="Artists, Songs, Lyrics, and More"
          aria-label="Search"
          role="combobox"
          aria-expanded={showList}
          aria-controls={listId}
          aria-activedescendant={active >= 0 ? `${listId}-${active}` : undefined}
          autoComplete="off"
          spellCheck={false}
          onChange={(e) => {
            setValue(e.target.value);
            setOpen(true);
            setActive(-1);
          }}
          onFocus={() => setOpen(true)}
          onBlur={() => setTimeout(() => { if (!menuOpen.current) setOpen(false); }, 120)}
          onKeyDown={(e) => {
            if (e.key === 'ArrowDown') {
              e.preventDefault();
              setActive((i) => Math.min(i + 1, options.length - 1));
            } else if (e.key === 'ArrowUp') {
              e.preventDefault();
              setActive((i) => Math.max(i - 1, -1));
            } else if (e.key === 'Enter') {
              e.preventDefault();
              pick(active >= 0 ? options[active] : undefined);
            } else if (e.key === 'Escape') {
              setOpen(false);
            }
          }}
        />
        {value && (
          <button
            type="button"
            className="absolute right-[12px] grid size-5 place-items-center rounded-full bg-tertiary text-page hover:bg-secondary"
            aria-label="Clear"
            onClick={() => {
              setValue('');
              input.current?.focus();
            }}
          >
            <X size={13} strokeWidth={3} />
          </button>
        )}
      </form>
      {showList && (
        // The dropdown: exact full width of the search field, 6px under the field (Spotify-style).
        <ul
          id={listId}
          className="absolute top-[calc(100%+6px)] inset-x-0 z-(--z-contextual-menus) max-h-[min(650px,65vh)] w-full overflow-y-auto rounded-[14px] border border-search-border bg-thick-material py-2 shadow-[0_16px_40px_rgba(0,0,0,0.65)] [backdrop-filter:blur(60px)_saturate(2.2)]"
          role="listbox"
        >
          {options.map((option, i) => {
            const item = option.kind === 'item' ? option.item : null;
            const playable = item !== null && (item.kind === 'song' || item.kind === 'video');
            return (
            <li
              key={option.kind === 'query' ? `q-${option.text}` : `i-${option.item.id}`}
              id={`${listId}-${i}`}
              role="option"
              aria-selected={i === active}
              className={option.kind === 'query' ? hint : `${lockup} ${playable ? lockupMenuCols : lockupCols}`}
              data-active={i === active || undefined}
              onMouseDown={(e) => e.preventDefault()}
              onMouseEnter={() => setActive(i)}
              onClick={() => pick(option)}
            >
              {option.kind === 'query' ? (
                <>
                  <Search size={14} strokeWidth={2.4} aria-hidden="true" className="shrink-0 text-secondary" />
                  <span className="truncate">
                    <span className="text-primary group-data-active/option:text-white font-medium">{option.typed}</span>
                    <span>{option.rest}</span>
                  </span>
                </>
              ) : (
                <>
                  <div className="w-[46px] shrink-0">
                    <Artwork
                      src={option.item.thumbnail}
                      displayWidth={46}
                      radius={option.item.kind === 'artist' ? '50%' : 'var(--radius-artwork-small)'}
                    />
                  </div>
                  <div className="flex min-w-0 flex-col">
                    <span className="truncate type-body font-medium">{option.item.title}</span>
                    <span className="mt-0.5 truncate type-callout text-secondary group-data-active/option:text-white">{option.item.subtitle || kindLabel(option.item.kind)}</span>
                  </div>
                  {playable && item && (
                    <TrackMenu
                      track={toTrack(item)}
                      onOpenChange={onMenuOpenChange}
                      className="grid size-8 place-items-center rounded-full text-secondary opacity-0 transition-opacity duration-100 ease-in group-hover/option:opacity-100 group-data-active/option:text-white focus-visible:opacity-100 hover:bg-quinary group-data-active/option:hover:bg-white/20 data-open:opacity-100"
                    />
                  )}
                </>
              )}
            </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}

// A suggestion row; hover or keyboard selection paints it in the key colour, white text.
const optionRow = 'group/option mx-2 cursor-pointer rounded-[8px] transition-colors duration-100 hover:bg-quaternary data-active:bg-key data-active:text-white';
const hint = `${optionRow} flex items-center gap-3 px-3 py-2.5 type-body text-secondary hover:text-primary`;
const lockup = `${optionRow} my-1 grid items-center gap-x-3.5 px-3 py-2`;
const lockupCols = 'grid-cols-[46px_minmax(0,1fr)]';
const lockupMenuCols = 'grid-cols-[46px_minmax(0,1fr)_auto]';

function kindLabel(kind: MusicItem['kind']): string {
  return { song: 'Song', video: 'Video', album: 'Album', playlist: 'Playlist', 'spotify-playlist': 'Spotify Playlist', artist: 'Artist', episode: 'Episode', podcast: 'Podcast', unknown: '' }[kind];
}
