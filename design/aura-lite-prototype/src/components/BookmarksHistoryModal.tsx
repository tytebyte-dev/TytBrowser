import React, { useState } from 'react';
import {
  Bookmark as BookmarkIcon,
  Clock,
  Search,
  Trash2,
  X,
  ExternalLink,
  Plus,
} from 'lucide-react';
import { Bookmark, HistoryItem, ThemeMode } from '../types/browser';

interface BookmarksHistoryModalProps {
  isOpen: boolean;
  initialTab?: 'bookmarks' | 'history';
  theme: ThemeMode;
  bookmarks: Bookmark[];
  history: HistoryItem[];
  onClose: () => void;
  onSelectUrl: (url: string) => void;
  onDeleteBookmark: (id: string) => void;
  onClearHistory: () => void;
  onDeleteHistoryItem: (id: string) => void;
}

export const BookmarksHistoryModal: React.FC<BookmarksHistoryModalProps> = ({
  isOpen,
  initialTab = 'bookmarks',
  theme,
  bookmarks,
  history,
  onClose,
  onSelectUrl,
  onDeleteBookmark,
  onClearHistory,
  onDeleteHistoryItem,
}) => {
  const [activeSegment, setActiveSegment] = useState<'bookmarks' | 'history'>(initialTab);
  const [searchFilter, setSearchFilter] = useState('');

  if (!isOpen) return null;

  const isDark = theme === 'dark' || theme === 'amoled';

  const modalBg =
    theme === 'amoled' && isDark
      ? 'bg-black border-neutral-800'
      : isDark
      ? 'bg-neutral-900 border-neutral-700'
      : 'bg-white border-neutral-200';

  const textColor = isDark ? 'text-neutral-100' : 'text-neutral-900';
  const rowHover = isDark ? 'hover:bg-neutral-800/60' : 'hover:bg-neutral-100';

  const filteredBookmarks = bookmarks.filter(
    (b) =>
      b.title.toLowerCase().includes(searchFilter.toLowerCase()) ||
      b.url.toLowerCase().includes(searchFilter.toLowerCase())
  );

  const filteredHistory = history.filter(
    (h) =>
      h.title.toLowerCase().includes(searchFilter.toLowerCase()) ||
      h.url.toLowerCase().includes(searchFilter.toLowerCase())
  );

  return (
    <div className="fixed inset-0 z-50 bg-black/60 flex items-center justify-center p-3 sm:p-5">
      <div
        className={`w-full max-w-md h-[80vh] max-h-[600px] rounded-3xl border shadow-2xl flex flex-col overflow-hidden ${modalBg} ${textColor}`}
      >
        {/* Header with Segmented tabs */}
        <div className="px-4 py-3.5 border-b border-neutral-800/40 flex items-center justify-between">
          <div className={`p-1 rounded-xl flex items-center gap-1 ${isDark ? 'bg-neutral-800' : 'bg-neutral-200'}`}>
            <button
              onClick={() => setActiveSegment('bookmarks')}
              className={`flex items-center gap-1.5 px-3 py-1 text-xs font-semibold rounded-lg transition-colors ${
                activeSegment === 'bookmarks'
                  ? isDark
                    ? 'bg-neutral-700 text-white shadow-xs'
                    : 'bg-white text-neutral-900 shadow-xs'
                  : 'text-neutral-400 hover:text-neutral-200'
              }`}
            >
              <BookmarkIcon size={13} />
              <span>Закладки ({bookmarks.length})</span>
            </button>

            <button
              onClick={() => setActiveSegment('history')}
              className={`flex items-center gap-1.5 px-3 py-1 text-xs font-semibold rounded-lg transition-colors ${
                activeSegment === 'history'
                  ? isDark
                    ? 'bg-neutral-700 text-white shadow-xs'
                    : 'bg-white text-neutral-900 shadow-xs'
                  : 'text-neutral-400 hover:text-neutral-200'
              }`}
            >
              <Clock size={13} />
              <span>История ({history.length})</span>
            </button>
          </div>

          <button
            onClick={onClose}
            className="p-1 rounded-full text-neutral-400 hover:text-neutral-200"
            title="Закрыть"
          >
            <X size={18} />
          </button>
        </div>

        {/* Search filter row */}
        <div className="px-4 py-2.5 border-b border-neutral-800/40">
          <div
            className={`flex items-center gap-2 px-3 py-1.5 rounded-xl text-xs ${
              isDark ? 'bg-neutral-800/80 text-white' : 'bg-neutral-100 text-black'
            }`}
          >
            <Search size={14} className="text-neutral-400" />
            <input
              type="text"
              value={searchFilter}
              onChange={(e) => setSearchFilter(e.target.value)}
              placeholder="Фильтр по названию или ссылке..."
              className="w-full bg-transparent focus:outline-none"
            />
            {searchFilter && (
              <button onClick={() => setSearchFilter('')} className="text-neutral-400 hover:text-white">
                <X size={13} />
              </button>
            )}
          </div>
        </div>

        {/* Content List */}
        <div className="flex-1 overflow-y-auto divide-y divide-neutral-800/30">
          {activeSegment === 'bookmarks' ? (
            filteredBookmarks.length === 0 ? (
              <div className="p-8 text-center text-neutral-400 text-xs">
                Нет сохранённых закладок
              </div>
            ) : (
              filteredBookmarks.map((bm) => (
                <div
                  key={bm.id}
                  onClick={() => {
                    onSelectUrl(bm.url);
                    onClose();
                  }}
                  className={`px-4 py-3 flex items-center justify-between cursor-pointer transition-colors ${rowHover}`}
                >
                  <div className="min-w-0 flex-1 pr-3">
                    <h4 className="text-xs font-semibold truncate">{bm.title}</h4>
                    <p className="text-[11px] text-neutral-400 font-mono truncate">{bm.url}</p>
                  </div>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onDeleteBookmark(bm.id);
                    }}
                    className="p-1.5 rounded-lg text-neutral-400 hover:text-red-400 hover:bg-neutral-800"
                    title="Удалить из закладок"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              ))
            )
          ) : filteredHistory.length === 0 ? (
            <div className="p-8 text-center text-neutral-400 text-xs">
              История посещений пуста
            </div>
          ) : (
            filteredHistory.map((item) => (
              <div
                key={item.id}
                onClick={() => {
                  onSelectUrl(item.url);
                  onClose();
                }}
                className={`px-4 py-3 flex items-center justify-between cursor-pointer transition-colors ${rowHover}`}
              >
                <div className="min-w-0 flex-1 pr-3">
                  <div className="flex items-center gap-2">
                    <h4 className="text-xs font-semibold truncate">{item.title}</h4>
                    <span className="text-[10px] text-neutral-400 shrink-0 font-mono">
                      {item.visitedAt}
                    </span>
                  </div>
                  <p className="text-[11px] text-neutral-400 font-mono truncate">{item.url}</p>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onDeleteHistoryItem(item.id);
                  }}
                  className="p-1.5 rounded-lg text-neutral-400 hover:text-red-400 hover:bg-neutral-800"
                  title="Удалить запись"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            ))
          )}
        </div>

        {/* Footer Actions */}
        {activeSegment === 'history' && history.length > 0 && (
          <div className="p-3 border-t border-neutral-800/40 flex justify-between items-center text-xs">
            <span className="text-neutral-400 text-[11px]">Локальное хранилище Android SQLite</span>
            <button
              onClick={onClearHistory}
              className="flex items-center gap-1.5 text-xs font-semibold text-red-400 hover:text-red-300 px-3 py-1.5 rounded-lg hover:bg-red-500/10 transition-colors"
            >
              <Trash2 size={13} />
              <span>Очистить историю</span>
            </button>
          </div>
        )}
      </div>
    </div>
  );
};
