import React, { useRef, useEffect } from 'react';
import {
  Plus,
  EyeOff,
  Star,
  BookOpen,
  Monitor,
  ShieldAlert,
  Download,
  Clock,
  Settings,
  Share2,
  ImageOff,
  LogOut,
  Moon,
  Sun,
  X,
} from 'lucide-react';
import { AddressBarPosition, BrowserSettings, ThemeMode } from '../types/browser';

interface BrowserMenuProps {
  isOpen: boolean;
  theme: ThemeMode;
  position: AddressBarPosition;
  settings: BrowserSettings;
  isArticlePage?: boolean;
  onClose: () => void;
  onNewTab: () => void;
  onNewPrivateTab: () => void;
  onAddBookmark: () => void;
  onToggleReaderMode: () => void;
  onToggleDesktopMode: () => void;
  onToggleAdBlock: () => void;
  onToggleBlockImages: () => void;
  onToggleTheme: () => void;
  onOpenBookmarks: () => void;
  onOpenHistory: () => void;
  onOpenDownloads: () => void;
  onOpenSettings: () => void;
  onClearSession: () => void;
}

export const BrowserMenu: React.FC<BrowserMenuProps> = ({
  isOpen,
  theme,
  position,
  settings,
  isArticlePage = false,
  onClose,
  onNewTab,
  onNewPrivateTab,
  onAddBookmark,
  onToggleReaderMode,
  onToggleDesktopMode,
  onToggleAdBlock,
  onToggleBlockImages,
  onToggleTheme,
  onOpenBookmarks,
  onOpenHistory,
  onOpenDownloads,
  onOpenSettings,
  onClearSession,
}) => {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const isDark = theme === 'dark' || theme === 'amoled';

  const menuBg =
    theme === 'amoled' && isDark
      ? 'bg-black border-neutral-800 text-neutral-200'
      : isDark
      ? 'bg-neutral-900 border-neutral-700 text-neutral-200'
      : 'bg-white border-neutral-200 text-neutral-800';

  const itemHover = isDark
    ? 'hover:bg-neutral-800 active:bg-neutral-700'
    : 'hover:bg-neutral-100 active:bg-neutral-200';

  const positionClass =
    position === 'bottom'
      ? 'bottom-14 right-2 origin-bottom-right'
      : 'top-14 right-2 origin-top-right';

  return (
    <div
      ref={menuRef}
      className={`absolute ${positionClass} z-50 w-64 rounded-2xl border shadow-2xl py-1.5 text-xs select-none ${menuBg} animate-in fade-in zoom-in-95 duration-100`}
    >
      {/* Quick Action Strip (New tab, Star, Share, Night theme) */}
      <div className="px-2 py-1.5 flex items-center justify-around border-b border-neutral-800/40 pb-2 mb-1">
        <button
          onClick={() => {
            onNewTab();
            onClose();
          }}
          className={`p-2 rounded-xl transition-colors ${itemHover}`}
          title="Новая вкладка"
        >
          <Plus size={18} />
        </button>

        <button
          onClick={() => {
            onAddBookmark();
            onClose();
          }}
          className={`p-2 rounded-xl transition-colors ${itemHover} text-amber-500`}
          title="В закладки"
        >
          <Star size={18} />
        </button>

        <button
          onClick={() => {
            onToggleTheme();
            onClose();
          }}
          className={`p-2 rounded-xl transition-colors ${itemHover}`}
          title="Переключить тему (Светлая / Тёмная / AMOLED)"
        >
          {isDark ? <Sun size={18} className="text-amber-400" /> : <Moon size={18} />}
        </button>

        <button
          onClick={() => {
            if (navigator.share) {
              navigator.share({ title: 'Aura Lite Web', url: window.location.href }).catch(() => {});
            }
            onClose();
          }}
          className={`p-2 rounded-xl transition-colors ${itemHover}`}
          title="Поделиться"
        >
          <Share2 size={18} />
        </button>
      </div>

      {/* Main Menu List */}
      <div className="space-y-0.5">
        <button
          onClick={() => {
            onNewPrivateTab();
            onClose();
          }}
          className={`w-full px-3.5 py-2 flex items-center gap-3 text-left transition-colors ${itemHover}`}
        >
          <EyeOff size={16} className="text-purple-400 shrink-0" />
          <span className="font-medium">Новая вкладка инкогнито</span>
        </button>

        {isArticlePage && (
          <button
            onClick={() => {
              onToggleReaderMode();
              onClose();
            }}
            className={`w-full px-3.5 py-2 flex items-center gap-3 text-left transition-colors ${itemHover}`}
          >
            <BookOpen size={16} className="text-blue-500 shrink-0" />
            <span className="font-medium">Режим чтения</span>
          </button>
        )}

        {/* Desktop Version Toggle */}
        <button
          onClick={onToggleDesktopMode}
          className={`w-full px-3.5 py-2 flex items-center justify-between text-left transition-colors ${itemHover}`}
        >
          <div className="flex items-center gap-3">
            <Monitor size={16} className="text-neutral-400 shrink-0" />
            <span className="font-medium">Версия для ПК</span>
          </div>
          <div
            className={`w-8 h-4 rounded-full transition-colors flex items-center px-0.5 ${
              settings.desktopMode ? 'bg-blue-600 justify-end' : 'bg-neutral-600 justify-start'
            }`}
          >
            <div className="w-3 h-3 rounded-full bg-white shadow-xs" />
          </div>
        </button>

        {/* Ad-Blocker Toggle */}
        <button
          onClick={onToggleAdBlock}
          className={`w-full px-3.5 py-2 flex items-center justify-between text-left transition-colors ${itemHover}`}
        >
          <div className="flex items-center gap-3">
            <ShieldAlert size={16} className="text-emerald-500 shrink-0" />
            <span className="font-medium">Блокировка рекламы</span>
          </div>
          <div
            className={`w-8 h-4 rounded-full transition-colors flex items-center px-0.5 ${
              settings.blockAds ? 'bg-emerald-600 justify-end' : 'bg-neutral-600 justify-start'
            }`}
          >
            <div className="w-3 h-3 rounded-full bg-white shadow-xs" />
          </div>
        </button>

        {/* Low RAM / Block Images Mode */}
        <button
          onClick={onToggleBlockImages}
          className={`w-full px-3.5 py-2 flex items-center justify-between text-left transition-colors ${itemHover}`}
        >
          <div className="flex items-center gap-3">
            <ImageOff size={16} className="text-amber-500 shrink-0" />
            <span className="font-medium">Режим малого ОЗУ (Без фото)</span>
          </div>
          <div
            className={`w-8 h-4 rounded-full transition-colors flex items-center px-0.5 ${
              settings.blockImages ? 'bg-amber-600 justify-end' : 'bg-neutral-600 justify-start'
            }`}
          >
            <div className="w-3 h-3 rounded-full bg-white shadow-xs" />
          </div>
        </button>

        <div className="h-px bg-neutral-800/40 my-1" />

        <button
          onClick={() => {
            onOpenBookmarks();
            onClose();
          }}
          className={`w-full px-3.5 py-2 flex items-center gap-3 text-left transition-colors ${itemHover}`}
        >
          <Star size={16} className="text-amber-400 shrink-0" />
          <span className="font-medium">Закладки</span>
        </button>

        <button
          onClick={() => {
            onOpenHistory();
            onClose();
          }}
          className={`w-full px-3.5 py-2 flex items-center gap-3 text-left transition-colors ${itemHover}`}
        >
          <Clock size={16} className="text-sky-400 shrink-0" />
          <span className="font-medium">История</span>
        </button>

        <button
          onClick={() => {
            onOpenDownloads();
            onClose();
          }}
          className={`w-full px-3.5 py-2 flex items-center gap-3 text-left transition-colors ${itemHover}`}
        >
          <Download size={16} className="text-emerald-400 shrink-0" />
          <span className="font-medium">Загрузки</span>
        </button>

        <button
          onClick={() => {
            onOpenSettings();
            onClose();
          }}
          className={`w-full px-3.5 py-2 flex items-center gap-3 text-left transition-colors ${itemHover}`}
        >
          <Settings size={16} className="text-purple-400 shrink-0" />
          <span className="font-medium">Настройки браузера</span>
        </button>

        <div className="h-px bg-neutral-800/40 my-1" />

        <button
          onClick={() => {
            onClearSession();
            onClose();
          }}
          className={`w-full px-3.5 py-2 flex items-center gap-3 text-left text-red-400 transition-colors ${itemHover}`}
        >
          <LogOut size={16} className="shrink-0" />
          <span className="font-medium">Очистить кэш и выйти</span>
        </button>
      </div>
    </div>
  );
};
