import React, { useState } from 'react';
import { Plus, X, Shield, Globe, Trash2, EyeOff } from 'lucide-react';
import { Tab, ThemeMode } from '../types/browser';

interface TabSwitcherProps {
  tabs: Tab[];
  privateTabs: Tab[];
  activeTabId: string;
  theme: ThemeMode;
  onSelectTab: (id: string, isPrivate: boolean) => void;
  onCloseTab: (id: string, isPrivate: boolean) => void;
  onNewTab: (isPrivate: boolean) => void;
  onCloseAllTabs: (isPrivate: boolean) => void;
  onCloseSwitcher: () => void;
}

export const TabSwitcher: React.FC<TabSwitcherProps> = ({
  tabs,
  privateTabs,
  activeTabId,
  theme,
  onSelectTab,
  onCloseTab,
  onNewTab,
  onCloseAllTabs,
  onCloseSwitcher,
}) => {
  const [currentMode, setCurrentMode] = useState<'regular' | 'private'>('regular');

  const isDark = theme === 'dark' || theme === 'amoled' || currentMode === 'private';
  const displayTabs = currentMode === 'regular' ? tabs : privateTabs;

  const bgClass =
    theme === 'amoled' && isDark
      ? 'bg-black'
      : isDark
      ? 'bg-neutral-900'
      : 'bg-neutral-100';

  const cardBg =
    theme === 'amoled' && isDark
      ? 'bg-neutral-950 border-neutral-800'
      : isDark
      ? 'bg-neutral-800/90 border-neutral-700'
      : 'bg-white border-neutral-200';

  return (
    <div className={`flex-1 w-full flex flex-col overflow-hidden ${bgClass} select-none transition-colors duration-200`}>
      {/* Top Bar with Mode Segmented Switcher & New Tab */}
      <div className={`px-4 py-3 border-b ${isDark ? 'border-neutral-800' : 'border-neutral-200'} flex items-center justify-between`}>
        {/* Regular vs Private Tab switcher */}
        <div className={`p-1 rounded-xl flex items-center gap-1 ${isDark ? 'bg-neutral-800' : 'bg-neutral-200'}`}>
          <button
            onClick={() => setCurrentMode('regular')}
            className={`px-3 py-1 text-xs font-semibold rounded-lg transition-colors ${
              currentMode === 'regular'
                ? isDark
                  ? 'bg-neutral-700 text-white shadow-xs'
                  : 'bg-white text-neutral-900 shadow-xs'
                : 'text-neutral-400 hover:text-neutral-200'
            }`}
          >
            Обычные ({tabs.length})
          </button>

          <button
            onClick={() => setCurrentMode('private')}
            className={`flex items-center gap-1.5 px-3 py-1 text-xs font-semibold rounded-lg transition-colors ${
              currentMode === 'private'
                ? 'bg-purple-900/60 text-purple-200 border border-purple-500/40 shadow-xs'
                : 'text-neutral-400 hover:text-neutral-200'
            }`}
          >
            <EyeOff size={13} />
            <span>Инкогнито ({privateTabs.length})</span>
          </button>
        </div>

        {/* Close Switcher (Return to Active tab) */}
        <button
          onClick={onCloseSwitcher}
          className={`p-1.5 rounded-full ${isDark ? 'text-neutral-300 hover:bg-neutral-800' : 'text-neutral-700 hover:bg-neutral-200'}`}
          title="Вернуться к странице"
        >
          <X size={18} />
        </button>
      </div>

      {/* Private Mode Banner info */}
      {currentMode === 'private' && (
        <div className="px-4 py-2 bg-purple-950/40 border-b border-purple-900/50 flex items-center gap-2 text-xs text-purple-300">
          <Shield size={14} className="shrink-0 text-purple-400" />
          <span>История посещений и cookies не сохраняются на диске Android.</span>
        </div>
      )}

      {/* Tab Grid */}
      <div className="flex-1 overflow-y-auto p-4">
        {displayTabs.length === 0 ? (
          <div className="h-full flex flex-col items-center justify-center text-center p-6 text-neutral-400">
            <Globe size={36} className="mb-2 opacity-40" />
            <p className="text-sm font-semibold mb-1">Нет открытых вкладок</p>
            <p className="text-xs max-w-xs mb-4">
              Нажмите кнопку ниже, чтобы открыть новую легковесную вкладку.
            </p>
            <button
              onClick={() => onNewTab(currentMode === 'private')}
              className="px-4 py-2 text-xs font-semibold bg-blue-600 hover:bg-blue-500 text-white rounded-xl shadow-xs"
            >
              + Создать вкладку
            </button>
          </div>
        ) : (
          <div className="grid grid-cols-2 gap-3 max-w-lg mx-auto">
            {displayTabs.map((tab) => {
              const isActive = tab.id === activeTabId;
              return (
                <div
                  key={tab.id}
                  onClick={() => onSelectTab(tab.id, currentMode === 'private')}
                  className={`group relative rounded-xl border flex flex-col overflow-hidden cursor-pointer transition-all duration-150 active:scale-95 ${cardBg} ${
                    isActive
                      ? 'ring-2 ring-blue-500 shadow-md'
                      : 'hover:border-neutral-500'
                  }`}
                >
                  {/* Tab Card Header */}
                  <div
                    className={`px-2.5 py-1.5 flex items-center justify-between text-xs border-b ${
                      isDark ? 'border-neutral-800' : 'border-neutral-200'
                    }`}
                  >
                    <span
                      className={`font-medium truncate text-[11px] pr-2 ${
                        isDark ? 'text-neutral-200' : 'text-neutral-800'
                      }`}
                    >
                      {tab.title}
                    </span>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onCloseTab(tab.id, currentMode === 'private');
                      }}
                      className="p-1 rounded-full text-neutral-400 hover:text-red-400 hover:bg-neutral-700/40"
                      title="Закрыть вкладку"
                    >
                      <X size={12} />
                    </button>
                  </div>

                  {/* Tab Card Preview Thumbnail */}
                  <div className="h-28 p-2 flex flex-col justify-center items-center text-center text-neutral-400 text-[11px] bg-neutral-900/10">
                    <span className="font-mono text-[10px] truncate max-w-full text-neutral-500 mb-1">
                      {tab.url}
                    </span>
                    <div className="w-8 h-8 rounded-full bg-neutral-500/10 flex items-center justify-center">
                      <Globe size={16} />
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Bottom Action Strip */}
      <div
        className={`px-4 py-2.5 border-t ${
          isDark ? 'border-neutral-800' : 'border-neutral-200'
        } flex items-center justify-between`}
      >
        <button
          onClick={() => onCloseAllTabs(currentMode === 'private')}
          disabled={displayTabs.length === 0}
          className={`flex items-center gap-1 text-xs font-medium text-neutral-400 hover:text-red-400 disabled:opacity-30 disabled:cursor-not-allowed`}
        >
          <Trash2 size={14} />
          <span>Закрыть все</span>
        </button>

        <button
          onClick={() => onNewTab(currentMode === 'private')}
          className="flex items-center gap-1.5 px-4 py-2 rounded-xl text-xs font-semibold bg-blue-600 hover:bg-blue-500 text-white shadow-xs active:scale-95 transition-transform"
        >
          <Plus size={15} />
          <span>Новая вкладка</span>
        </button>
      </div>
    </div>
  );
};
