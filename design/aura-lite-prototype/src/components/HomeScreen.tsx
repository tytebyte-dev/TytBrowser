import React, { useState } from 'react';
import {
  Search,
  Plus,
  BookOpen,
  Code,
  Smartphone,
  GitBranch,
  CloudSun,
  MessageSquare,
  MapPin,
  Archive,
  Globe,
  Trash2,
  Cpu,
  Shield,
  Zap,
  Bookmark as BookmarkIcon,
  Clock,
  Download,
  Settings,
  Flame,
} from 'lucide-react';
import { Bookmark, BrowserSettings, SearchEngine, ThemeMode } from '../types/browser';

interface HomeScreenProps {
  theme: ThemeMode;
  isPrivate?: boolean;
  bookmarks: Bookmark[];
  settings: BrowserSettings;
  onNavigate: (url: string) => void;
  onOpenBookmarks: () => void;
  onOpenDownloads: () => void;
  onOpenHistory: () => void;
  onOpenSettings: () => void;
  onAddBookmark: (title: string, url: string) => void;
  onDeleteBookmark: (id: string) => void;
  onSearch: (query: string) => void;
  onSelectSearchEngine: (engine: SearchEngine) => void;
}

const ICON_MAP: Record<string, React.ElementType> = {
  BookOpen,
  Code,
  Smartphone,
  GitBranch,
  CloudSun,
  MessageSquare,
  MapPin,
  Archive,
  Globe,
};

export const HomeScreen: React.FC<HomeScreenProps> = ({
  theme,
  isPrivate = false,
  bookmarks,
  settings,
  onNavigate,
  onOpenBookmarks,
  onOpenDownloads,
  onOpenHistory,
  onOpenSettings,
  onAddBookmark,
  onDeleteBookmark,
  onSearch,
  onSelectSearchEngine,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [showAddModal, setShowAddModal] = useState(false);
  const [newTitle, setNewTitle] = useState('');
  const [newUrl, setNewUrl] = useState('');
  const [showEnginePicker, setShowEnginePicker] = useState(false);

  const isDark = theme === 'dark' || theme === 'amoled' || isPrivate;

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (searchQuery.trim()) {
      onSearch(searchQuery.trim());
    }
  };

  const handleAddSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (newTitle.trim() && newUrl.trim()) {
      let finalUrl = newUrl.trim();
      if (!finalUrl.startsWith('http://') && !finalUrl.startsWith('https://')) {
        finalUrl = 'https://' + finalUrl;
      }
      onAddBookmark(newTitle.trim(), finalUrl);
      setNewTitle('');
      setNewUrl('');
      setShowAddModal(false);
    }
  };

  const searchEngineNames: Record<SearchEngine, string> = {
    duckduckgo: 'DuckDuckGo',
    google: 'Google',
    yandex: 'Яндекс',
    bing: 'Bing',
  };

  const cardBg =
    theme === 'amoled' && isDark
      ? 'bg-neutral-950 border border-neutral-900'
      : isDark
      ? 'bg-neutral-900/90 border border-neutral-800'
      : 'bg-white border border-neutral-200/80 shadow-xs';

  const textColor = isDark ? 'text-neutral-100' : 'text-neutral-900';
  const mutedText = isDark ? 'text-neutral-400' : 'text-neutral-500';

  return (
    <div className="flex-1 w-full overflow-y-auto px-4 py-6 flex flex-col items-center">
      {/* Brand Header */}
      <div className="mb-6 text-center select-none">
        <div className="inline-flex items-center gap-2 mb-1">
          <div className="w-8 h-8 rounded-xl bg-gradient-to-br from-blue-600 to-indigo-600 flex items-center justify-center text-white font-bold shadow-md shadow-blue-500/20">
            <Zap size={18} className="fill-current text-white" />
          </div>
          <span className={`text-2xl font-bold tracking-tight ${textColor}`}>
            Aura <span className="font-light text-blue-500">Lite</span>
          </span>
        </div>
        <p className="text-xs text-neutral-400 font-medium">
          {isPrivate ? 'Приватная сессия без следов' : 'Быстрый легковесный браузер'}
        </p>
      </div>

      {/* Main Search Box with Engine Switcher */}
      <div className="w-full max-w-md mb-5">
        <form onSubmit={handleSearchSubmit} className="relative">
          <div
            className={`flex items-center gap-2 px-3.5 py-2.5 rounded-2xl transition-all ${
              isDark
                ? 'bg-neutral-900 border border-neutral-800 focus-within:border-blue-500'
                : 'bg-neutral-100 border border-neutral-300/80 focus-within:border-blue-500 shadow-xs'
            }`}
          >
            {/* Search Engine Picker Button */}
            <div className="relative">
              <button
                type="button"
                onClick={() => setShowEnginePicker(!showEnginePicker)}
                className={`flex items-center gap-1 text-xs font-semibold px-2 py-1 rounded-md transition-colors ${
                  isDark
                    ? 'bg-neutral-800 hover:bg-neutral-700 text-neutral-200'
                    : 'bg-white hover:bg-neutral-200 text-neutral-800 shadow-xs'
                }`}
                title="Сменить поисковую систему"
              >
                <span>{searchEngineNames[settings.searchEngine]}</span>
                <span className="text-[10px] text-neutral-400">▾</span>
              </button>

              {showEnginePicker && (
                <div
                  className={`absolute top-8 left-0 z-40 py-1 w-32 rounded-lg shadow-lg border text-xs font-medium ${
                    isDark
                      ? 'bg-neutral-900 border-neutral-700 text-neutral-200'
                      : 'bg-white border-neutral-200 text-neutral-800'
                  }`}
                >
                  {(['duckduckgo', 'google', 'yandex', 'bing'] as SearchEngine[]).map(
                    (engine) => (
                      <button
                        key={engine}
                        type="button"
                        onClick={() => {
                          onSelectSearchEngine(engine);
                          setShowEnginePicker(false);
                        }}
                        className={`w-full text-left px-3 py-1.5 hover:bg-blue-500/10 transition-colors ${
                          settings.searchEngine === engine
                            ? 'text-blue-500 font-bold'
                            : ''
                        }`}
                      >
                        {searchEngineNames[engine]}
                      </button>
                    )
                  )}
                </div>
              )}
            </div>

            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Поиск или веб-адрес..."
              className={`w-full bg-transparent text-sm focus:outline-none ${textColor} placeholder-neutral-400`}
            />

            <button
              type="submit"
              aria-label="Искать"
              className="p-1 rounded-full text-blue-500 hover:text-blue-600 active:scale-95 transition-transform"
            >
              <Search size={18} />
            </button>
          </div>
        </form>
      </div>

      {/* Lightweight Memory & Efficiency Indicator (Zero-Pill clean typography) */}
      <div className="w-full max-w-md mb-6 px-1">
        <div
          className={`px-3 py-2 rounded-xl text-xs flex items-center justify-between gap-2 ${
            isDark ? 'bg-neutral-900/60 text-neutral-400 border border-neutral-800/80' : 'bg-neutral-100 text-neutral-600 border border-neutral-200'
          }`}
        >
          <div className="flex items-center gap-1.5 font-mono text-[11px]">
            <Cpu size={13} className="text-emerald-500" />
            <span>ОЗУ: 26 МБ</span>
          </div>
          <span className="text-neutral-500">·</span>
          <div className="flex items-center gap-1.5 font-mono text-[11px]">
            <Shield size={13} className="text-blue-500" />
            <span>Трекеры: 0</span>
          </div>
          <span className="text-neutral-500">·</span>
          <div className="flex items-center gap-1.5 font-mono text-[11px]">
            <Zap size={13} className="text-amber-500" />
            <span>APK: 2.1 МБ</span>
          </div>
        </div>
      </div>

      {/* Speed Dial Grid (Быстрый доступ) */}
      <div className="w-full max-w-md mb-6">
        <div className="flex items-center justify-between mb-3 px-1">
          <span className={`text-xs font-bold uppercase tracking-wider ${mutedText}`}>
            Быстрый доступ
          </span>
          <button
            onClick={() => setShowAddModal(true)}
            className="flex items-center gap-1 text-xs text-blue-500 hover:text-blue-400 font-medium"
          >
            <Plus size={13} />
            <span>Добавить</span>
          </button>
        </div>

        <div className="grid grid-cols-4 gap-3">
          {bookmarks.map((bm) => {
            const IconComponent = ICON_MAP[bm.iconName] || Globe;
            return (
              <div
                key={bm.id}
                className="group relative flex flex-col items-center select-none"
              >
                <button
                  onClick={() => onNavigate(bm.url)}
                  className={`w-13 h-13 rounded-2xl flex items-center justify-center text-white shadow-xs transition-transform active:scale-95 hover:scale-105 mb-1.5 ${
                    bm.color || 'bg-neutral-700'
                  }`}
                  title={`${bm.title} (${bm.url})`}
                >
                  <IconComponent size={20} strokeWidth={2.2} />
                </button>
                <span
                  className={`text-[11px] font-medium text-center truncate w-full px-1 ${textColor}`}
                >
                  {bm.title}
                </span>

                {/* Remove button on hover */}
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onDeleteBookmark(bm.id);
                  }}
                  title="Удалить ярлык"
                  className="absolute -top-1 -right-1 w-4 h-4 rounded-full bg-red-600 text-white flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity text-[10px]"
                >
                  ×
                </button>
              </div>
            );
          })}

          {/* Add custom tile */}
          <button
            onClick={() => setShowAddModal(true)}
            className={`flex flex-col items-center justify-center select-none group`}
          >
            <div
              className={`w-13 h-13 rounded-2xl border-2 border-dashed flex items-center justify-center transition-colors active:scale-95 mb-1.5 ${
                isDark
                  ? 'border-neutral-700 text-neutral-400 hover:border-neutral-500 hover:text-neutral-200'
                  : 'border-neutral-300 text-neutral-500 hover:border-neutral-400 hover:text-neutral-700'
              }`}
            >
              <Plus size={20} />
            </div>
            <span className={`text-[11px] font-medium text-center ${mutedText}`}>
              Новый
            </span>
          </button>
        </div>
      </div>

      {/* Quick Navigation Drawer Buttons */}
      <div className="w-full max-w-md grid grid-cols-4 gap-2 mb-6">
        <button
          onClick={onOpenBookmarks}
          className={`flex flex-col items-center gap-1.5 py-3 px-2 rounded-xl transition-all active:scale-95 ${cardBg}`}
        >
          <BookmarkIcon size={18} className="text-blue-500" />
          <span className={`text-xs font-medium ${textColor}`}>Закладки</span>
        </button>

        <button
          onClick={onOpenHistory}
          className={`flex flex-col items-center gap-1.5 py-3 px-2 rounded-xl transition-all active:scale-95 ${cardBg}`}
        >
          <Clock size={18} className="text-amber-500" />
          <span className={`text-xs font-medium ${textColor}`}>История</span>
        </button>

        <button
          onClick={onOpenDownloads}
          className={`flex flex-col items-center gap-1.5 py-3 px-2 rounded-xl transition-all active:scale-95 ${cardBg}`}
        >
          <Download size={18} className="text-emerald-500" />
          <span className={`text-xs font-medium ${textColor}`}>Загрузки</span>
        </button>

        <button
          onClick={onOpenSettings}
          className={`flex flex-col items-center gap-1.5 py-3 px-2 rounded-xl transition-all active:scale-95 ${cardBg}`}
        >
          <Settings size={18} className="text-purple-500" />
          <span className={`text-xs font-medium ${textColor}`}>Опции</span>
        </button>
      </div>

      {/* Suggested lightweight test pages */}
      <div className="w-full max-w-md">
        <div className="flex items-center gap-1 mb-2 px-1">
          <Flame size={14} className="text-orange-500" />
          <span className={`text-xs font-bold uppercase tracking-wider ${mutedText}`}>
            Оптимизированные страницы
          </span>
        </div>

        <div className="space-y-2">
          <div
            onClick={() => onNavigate('https://habr.com/ru/articles/android-optimization')}
            className={`p-3 rounded-xl cursor-pointer transition-all active:scale-[0.99] ${cardBg} hover:opacity-95`}
          >
            <div className="flex items-center justify-between mb-1">
              <span className="text-xs font-semibold text-sky-500">Хабр IT</span>
              <span className="text-[10px] text-neutral-500">3 мин чтения</span>
            </div>
            <h4 className={`text-xs font-semibold leading-snug ${textColor}`}>
              Оптимизация WebView и памяти под Android 10 (Go Edition)
            </h4>
          </div>

          <div
            onClick={() => onNavigate('https://ru.wikipedia.org/wiki/Android_10')}
            className={`p-3 rounded-xl cursor-pointer transition-all active:scale-[0.99] ${cardBg} hover:opacity-95`}
          >
            <div className="flex items-center justify-between mb-1">
              <span className="text-xs font-semibold text-neutral-400">Википедия</span>
              <span className="text-[10px] text-neutral-500">Архив версий</span>
            </div>
            <h4 className={`text-xs font-semibold leading-snug ${textColor}`}>
              Android 10: Архитектура, Scoped Storage и системная тёмная тема
            </h4>
          </div>
        </div>
      </div>

      {/* Add Bookmark Modal */}
      {showAddModal && (
        <div className="fixed inset-0 z-50 bg-black/60 flex items-center justify-center p-4">
          <div
            className={`w-full max-w-xs p-5 rounded-2xl border shadow-2xl ${
              isDark
                ? 'bg-neutral-900 border-neutral-700 text-neutral-100'
                : 'bg-white border-neutral-200 text-neutral-900'
            }`}
          >
            <h3 className="text-sm font-bold mb-3">Добавить ярлык</h3>
            <form onSubmit={handleAddSubmit} className="space-y-3">
              <div>
                <label className="block text-[11px] text-neutral-400 mb-1">Название</label>
                <input
                  type="text"
                  required
                  value={newTitle}
                  onChange={(e) => setNewTitle(e.target.value)}
                  placeholder="Например: Новости"
                  className={`w-full px-3 py-1.5 text-xs rounded-lg border ${
                    isDark
                      ? 'bg-neutral-800 border-neutral-700 text-white'
                      : 'bg-neutral-50 border-neutral-300 text-black'
                  }`}
                />
              </div>
              <div>
                <label className="block text-[11px] text-neutral-400 mb-1">Веб-адрес (URL)</label>
                <input
                  type="text"
                  required
                  value={newUrl}
                  onChange={(e) => setNewUrl(e.target.value)}
                  placeholder="https://example.com"
                  className={`w-full px-3 py-1.5 text-xs rounded-lg border ${
                    isDark
                      ? 'bg-neutral-800 border-neutral-700 text-white'
                      : 'bg-neutral-50 border-neutral-300 text-black'
                  }`}
                />
              </div>
              <div className="flex justify-end gap-2 pt-2">
                <button
                  type="button"
                  onClick={() => setShowAddModal(false)}
                  className="px-3 py-1.5 text-xs font-medium text-neutral-400 hover:text-neutral-200"
                >
                  Отмена
                </button>
                <button
                  type="submit"
                  className="px-4 py-1.5 text-xs font-semibold bg-blue-600 hover:bg-blue-500 text-white rounded-lg shadow-xs"
                >
                  Сохранить
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
