import { useState } from 'react';
import { useSearchParams } from 'react-router';
import { Play } from 'lucide-react';
import { PrefetchLink } from '@/components/PrefetchLink';
import { Artwork } from '@/components/music/Artwork';
import { CategoryTiles } from '@/components/music/CategoryTiles';
import { hrefFor } from '@/components/music/links';
import { playTrack, toTrack } from '@/components/music/play';
import { SearchField } from '@/components/music/SearchField';
import { Shelves } from '@/components/music/ShelfView';
import { TrackMenu } from '@/components/music/TrackMenu';
import { EmptyState, LoadError, NeedsExtension } from '@/components/music/States';
import { useCategories, useSearch, useYouTubeReady } from '@/data/music/queries';
import type { MusicItem, SearchResults, Shelf } from '@/data/music/types';
import { SkeletonBlock, SkeletonLine } from '@/components/Skeleton';
import { ShelfSkeleton } from '@/skeletons/ShelfSkeleton';

type SearchFilter = 'All' | 'Songs' | 'Videos' | 'Albums' | 'Artists' | 'Playlists';
const FILTERS: SearchFilter[] = ['All', 'Songs', 'Videos', 'Albums', 'Artists', 'Playlists'];

const KIND_LABEL: Record<MusicItem['kind'], string> = {
  song: 'Song',
  video: 'Video',
  album: 'Album',
  playlist: 'Playlist',
  'spotify-playlist': 'Spotify Playlist',
  artist: 'Artist',
  episode: 'Episode',
  podcast: 'Podcast',
  unknown: '',
};

/** Apple's result shelves, in its order, from results grouped by kind. */
function shelvesFor(results: SearchResults, filter: SearchFilter): Shelf[] {
  const shelf = (title: string, items: MusicItem[], layout: Shelf['layout']): Shelf | null =>
    items.length ? { title, strapline: null, layout, items, seeAll: null } : null;

  const all = [
    shelf('Songs', results.songs, 'tracks'),
    shelf('Artists', results.artists, 'circle'),
    shelf('Albums', results.albums, 'square'),
    shelf('Playlists', results.playlists, 'square'),
    shelf('Videos', results.videos, 'video'),
  ].filter((s): s is Shelf => !!s);

  if (filter === 'All') return all;
  if (filter === 'Songs') return all.filter((s) => s.title === 'Songs');
  if (filter === 'Artists') return all.filter((s) => s.title === 'Artists');
  if (filter === 'Albums') return all.filter((s) => s.title === 'Albums');
  if (filter === 'Playlists') return all.filter((s) => s.title === 'Playlists');
  if (filter === 'Videos') return all.filter((s) => s.title === 'Videos');
  return all;
}

const sectionTitle = 'mx-(--body-gutter) mt-4 mb-2 type-title-2-emphasized';

/** TopResultCard matching Kotlin's TopResultCard design */
function TopResultCard({ item }: { item: MusicItem }) {
  const href = hrefFor(item);
  const playable = item.kind === 'song' || item.kind === 'video';

  return (
    <div className="mx-(--body-gutter) mb-6">
      <h2 className="mb-2.5 type-headline text-secondary">Top result</h2>
      <div className="group/top relative flex items-center justify-between rounded-[12px] bg-quinary p-3.5 transition-colors hover:bg-quaternary">
        <div className="flex min-w-0 items-center gap-4">
          <div className="size-[72px] shrink-0 overflow-hidden rounded-[10px] bg-joe">
            <Artwork
              src={item.thumbnail}
              displayWidth={72}
              radius={item.kind === 'artist' ? '50%' : '10px'}
            />
          </div>
          <div className="flex min-w-0 flex-col gap-1">
            {href ? (
              <PrefetchLink to={href} className="truncate type-title-1 hover:underline">
                {item.title}
              </PrefetchLink>
            ) : (
              <span className="truncate type-title-1">{item.title}</span>
            )}
            <span className="truncate type-callout text-secondary">
              {[KIND_LABEL[item.kind], item.kind === 'artist' ? null : item.artists.map((a) => a.name).join(', ')].filter(Boolean).join(' · ')}
            </span>
          </div>
        </div>

        <div className="flex shrink-0 items-center gap-2 pl-4">
          {playable && (
            <button
              type="button"
              className="flex h-9 items-center gap-1.5 rounded-full bg-key px-4 text-white font-medium type-callout shadow-sm transition-transform hover:scale-105 active:scale-95"
              onClick={() => playTrack(item)}
            >
              <Play size={15} fill="currentColor" />
              <span>Play</span>
            </button>
          )}
          {playable && <TrackMenu track={toTrack(item)} className="grid size-9 place-items-center rounded-full text-secondary hover:bg-tertiary/20 hover:text-primary" />}
        </div>
      </div>
    </div>
  );
}

function ResultsSkeleton() {
  return (
    <div aria-busy="true">
      <div className={sectionTitle}>
        <SkeletonLine lineHeight={22} fontSize={17} width={130} />
      </div>
      <div className="mx-(--body-gutter) mt-3 mb-6">
        <SkeletonBlock height={100} radius="12px" />
      </div>
      <ShelfSkeleton kind="circle" />
      <ShelfSkeleton kind="square" />
    </div>
  );
}

function Results({ query, filter }: { query: string; filter: SearchFilter }) {
  const search = useSearch(query);
  if (search.isError) return <LoadError error={search.error} onRetry={() => void search.refetch()} />;
  if (!search.data) return <ResultsSkeleton />;
  const results = search.data;
  const shelves = shelvesFor(results, filter);
  if (!results.top.length && !shelves.length) {
    return <EmptyState title="No Results" detail={`Try a new search for something other than “${query}”.`} />;
  }

  const primaryTop = results.top[0] ?? null;

  return (
    <>
      {filter === 'All' && primaryTop && <TopResultCard item={primaryTop} />}
      <Shelves shelves={shelves} />
    </>
  );
}

function Landing() {
  const categories = useCategories();
  if (categories.isError) return <LoadError error={categories.error} onRetry={() => void categories.refetch()} />;
  if (!categories.data) {
    return (
      <div className="mx-(--body-gutter) mt-[46px] mb-8 grid grid-cols-4 gap-5" aria-busy="true">
        {Array.from({ length: 12 }, (_, i) => (
          <SkeletonBlock key={i} aspectRatio="16 / 9" radius="var(--radius-artwork-large)" />
        ))}
      </div>
    );
  }
  return <CategoryTiles groups={categories.data.map((g) => ({ ...g, title: g.title || 'Browse Categories' }))} />;
}

/** Search screen with Kotlin parity: sticky bar, category filter pills, TopResultCard, and shelves */
export default function SearchPage() {
  const { missing } = useYouTubeReady();
  const [params, setParams] = useSearchParams();
  const query = params.get('q') ?? '';
  const [filter, setFilter] = useState<SearchFilter>('All');

  return (
    <>
      <div className="sticky top-0 z-[calc(var(--z-default)+5)] flex flex-col items-center px-(--body-gutter) pt-3 pb-2.5 gap-2.5 before:absolute before:inset-0 before:-z-1 before:bg-header-material before:[backdrop-filter:saturate(180%)_blur(20px)]">
        <SearchField initial={query} onSearch={(q) => { setParams(q ? { q } : {}); setFilter('All'); }} />
        {query && (
          <div className="flex flex-wrap items-center justify-center gap-1.5" role="tablist" aria-label="Search categories">
            {FILTERS.map((f) => (
              <button
                key={f}
                type="button"
                role="tab"
                aria-selected={filter === f}
                onClick={() => setFilter(f)}
                className={`h-7 px-3.5 rounded-full text-[13px] font-medium transition-all duration-150 ${
                  filter === f
                    ? 'bg-key text-white shadow-sm'
                    : 'bg-quinary text-secondary hover:bg-quaternary hover:text-primary'
                }`}
              >
                {f}
              </button>
            ))}
          </div>
        )}
      </div>
      {missing ? <NeedsExtension what="and search YouTube Music" /> : query ? <Results query={query} filter={filter} /> : <Landing />}
    </>
  );
}
