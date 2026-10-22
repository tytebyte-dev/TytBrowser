import React, { useState, useRef, useEffect } from 'react';
import {
  Lock,
  RotateCw,
  X,
  MoreVertical,
  BookOpen,
  ArrowLeft,
  ArrowRight,
  ShieldCheck,
  Search,
} from 'lucide-react';
import { AddressBarPosition, ThemeMode } from '../types/browser';

interface OmnibarProps {
  url: string;
  title: string;
  theme: ThemeMode;
  position: AddressBarPosition;
  isPrivate?: boolean;
  isLoading?: boolean;
  canGoBack?: boolean;
  canGoForward?: boolean;
  tabsCount: number;
  isReaderModeActive?: boolean;
  isArticlePage?: boolean;
  onNavigate: (url: string) => void;
  onReload: () => void;
  onGoBack: () => void;
  onGoForward: () => void;
  onToggleReaderMode: () => void;
  onOpenTabSwitcher: () => void;
  onOpenMenu: () => void;
  onHomeClick: () => void;
}

export const Omnibar: React.FC<OmnibarProps> = ({
  url,
  theme,
  position,
  isPrivate = false,
  isLoading = false,
  canGoBack = false,
  canGoForward = false,
  tabsCount,
  isReaderModeActive = false,
  isArticlePage = false,
  onNavigate,
  onReload,
  onGoBack,
  onGoForward,
  onToggleReaderMode,
  onOpenTabSwitcher,
  onOpenMenu,
  onHomeClick,
}) => {
  const [isEditing, setIsEditing] = useState(false);
  const [inputVal, setInputVal] = useState(url);
  const [showSecurityInfo, setShowSecurityInfo] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setInputVal(url);
  }, [url]);

  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (inputVal.trim()) {
      onNavigate(inputVal.trim());
      setIsEditing(false);
    }
  };

  const isDark = theme === 'dark' || theme === 'amoled' || isPrivate;
  const barBg =
    theme === 'amoled' && isDark
      ? 'bg-black'
      : isDark
      ? 'bg-neutral-900'
      : 'bg-white';

  const inputBg =
    theme === 'amoled' && isDark
      ? 'bg-neutral-950 border border-neutral-800'
      : isDark
      ? 'bg-neutral-800/80 border border-neutral-700/60'
      : 'bg-neutral-100 border border-neutral-200';

  const textColor = isDark ? 'text-neutral-100' : 'text-neutral-900';
  const mutedColor = isDark ? 'text-neutral-400' : 'text-neutral-500';

  // Domain display formatting
  let displayDomain = url;
  try {
    if (url.startsWith('aura://')) {
      displayDomain = url === 'aura://start' ? 'Поиск или URL' : url;
    } else {
      const parsed = new URL(url);
      displayDomain = parsed.hostname;
    }
  } catch {
    displayDomain = url;
  }

  const borderClass =
    position === 'bottom'
      ? isDark
        ? 'border-t border-neutral-800'
        : 'border-t border-neutral-200'
      : isDark
      ? 'border-b border-neutral-800'
      : 'border-b border-neutral-200';

  return (
    <div className={`relative w-full z-20 ${barBg} ${borderClass} transition-colors duration-200`}>
      {/* Top Loading Progress Line */}
      {isLoading && (
        <div className="absolute top-0 left-0 right-0 h-0.5 bg-neutral-200 dark:bg-neutral-800 overflow-hidden">
          <div className="h-full bg-blue-500 animate-[loading_1.2s_ease-in-out_infinite] w-2/3 rounded-r-full" />
        </div>
      )}

      <div className="px-2 py-1.5 flex items-center gap-1.5 min-h-[50px]">
        {/* Navigation Back / Forward Buttons */}
        <div className="flex items-center">
          <button
            onClick={onGoBack}
            disabled={!canGoBack}
            aria-label="Назад"
            title="Назад"
            className={`w-8 h-8 flex items-center justify-center rounded-full transition-colors ${
              canGoBack
                ? isDark
                  ? 'text-neutral-200 hover:bg-neutral-800 active:bg-neutral-700'
                  : 'text-neutral-700 hover:bg-neutral-200 active:bg-neutral-300'
                : 'text-neutral-400/40 cursor-not-allowed'
            }`}
          >
            <ArrowLeft size={16} />
          </button>
        </div>

        {/* Address / Search Bar Box */}
        <div className="flex-1 min-w-0">
          {isEditing ? (
            <form onSubmit={handleSubmit} className="flex items-center">
              <div
                className={`w-full flex items-center gap-1.5 px-3 py-1.5 rounded-full ${inputBg}`}
              >
                <Search size={14} className={mutedColor} />
                <input
                  ref={inputRef}
                  type="text"
                  value={inputVal}
                  onChange={(e) => setInputVal(e.target.value)}
                  placeholder="Введите запрос или веб-адрес..."
                  className={`w-full bg-transparent text-sm focus:outline-none ${textColor}`}
                />
                {inputVal && (
                  <button
                    type="button"
                    onClick={() => setInputVal('')}
                    className="p-0.5 text-neutral-400 hover:text-neutral-200"
                  >
                    <X size={14} />
                  </button>
                )}
              </div>
            </form>
          ) : (
            <div
              onClick={() => setIsEditing(true)}
              className={`flex items-center justify-between px-3 py-1.5 rounded-full cursor-pointer select-none transition-all ${inputBg} hover:opacity-90 active:scale-[0.99]`}
            >
              <div className="flex items-center gap-1.5 min-w-0 flex-1">
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    setShowSecurityInfo(!showSecurityInfo);
                  }}
                  title="Сведения о сайте и сертификате"
                  aria-label="Статус безопасности"
                  className="shrink-0 p-0.5 text-emerald-500 hover:text-emerald-400"
                >
                  <Lock size={13} strokeWidth={2.4} />
                </button>
                <span className={`text-sm truncate font-medium ${textColor}`}>
                  {displayDomain}
                </span>
                {url !== 'aura://start' && (
                  <span className="text-xs text-neutral-500 truncate hidden xs:inline">
                    {url.replace(/https?:\/\/[^/]+/, '')}
                  </span>
                )}
              </div>

              <div className="flex items-center gap-1 shrink-0 ml-1">
                {isArticlePage && (
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation();
                      onToggleReaderMode();
                    }}
                    title={isReaderModeActive ? 'Обычный вид' : 'Режим чтения (Без отвлечений)'}
                    className={`p-1 rounded-full transition-colors ${
                      isReaderModeActive
                        ? 'text-blue-500 bg-blue-500/10'
                        : isDark
                        ? 'text-neutral-400 hover:text-neutral-200'
                        : 'text-neutral-600 hover:text-neutral-900'
                    }`}
                  >
                    <BookOpen size={14} />
                  </button>
                )}

                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    onReload();
                  }}
                  title="Обновить страницу"
                  className={`p-1 rounded-full ${mutedColor} hover:${textColor} transition-transform active:rotate-180`}
                >
                  <RotateCw size={13} className={isLoading ? 'animate-spin' : ''} />
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Tabs Counter Button [ N ] */}
        <button
          onClick={onOpenTabSwitcher}
          aria-label={`Открыто вкладок: ${tabsCount}`}
          title="Список вкладок"
          className={`w-7 h-7 flex items-center justify-center rounded-md border text-xs font-bold font-mono transition-transform active:scale-95 ${
            isDark
              ? 'border-neutral-600 text-neutral-200 hover:border-neutral-400'
              : 'border-neutral-400 text-neutral-800 hover:border-neutral-600'
          }`}
        >
          {tabsCount}
        </button>

        {/* Overflow Menu Button (⋮) */}
        <button
          onClick={onOpenMenu}
          aria-label="Меню браузера"
          title="Меню браузера"
          className={`w-8 h-8 flex items-center justify-center rounded-full transition-colors ${
            isDark
              ? 'text-neutral-300 hover:bg-neutral-800 active:bg-neutral-700'
              : 'text-neutral-700 hover:bg-neutral-200 active:bg-neutral-300'
          }`}
        >
          <MoreVertical size={18} />
        </button>
      </div>

      {/* Security Info Popup */}
      {showSecurityInfo && (
        <div
          className={`absolute left-3 right-3 top-12 z-50 p-3.5 rounded-xl shadow-xl text-xs border ${
            isDark
              ? 'bg-neutral-900 border-neutral-700 text-neutral-200'
              : 'bg-white border-neutral-200 text-neutral-800'
          }`}
        >
          <div className="flex items-start justify-between gap-2 mb-2">
            <div className="flex items-center gap-1.5 font-semibold text-emerald-500">
              <ShieldCheck size={16} />
              <span>Безопасное соединение</span>
            </div>
            <button
              onClick={() => setShowSecurityInfo(false)}
              className="text-neutral-400 hover:text-neutral-200 p-0.5"
            >
              <X size={14} />
            </button>
          </div>
          <p className="text-neutral-400 mb-2 leading-relaxed">
            Подключение зашифровано по протоколу TLS 1.3. Сертификат проверен системным хранилищем Android.
          </p>
          <div className="space-y-1.5 pt-2 border-t border-neutral-800/40 text-[11px]">
            <div className="flex justify-between">
              <span className="text-neutral-500">Cookies:</span>
              <span className="font-mono">Только 1st-party (Сторонние блокированы)</span>
            </div>
            <div className="flex justify-between">
              <span className="text-neutral-500">Песочница:</span>
              <span className="font-mono">Android WebView Sandbox</span>
            </div>
            <div className="flex justify-between">
              <span className="text-neutral-500">Блокировка рекламы:</span>
              <span className="text-emerald-500 font-semibold">Активна</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
