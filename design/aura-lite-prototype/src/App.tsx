import React, { useState, useEffect } from 'react';
import {
  Bookmark,
  BrowserSettings,
  DownloadItem,
  HistoryItem,
  SearchEngine,
  Tab,
  ThemeMode,
} from './types/browser';
import {
  DEFAULT_BOOKMARKS,
  INITIAL_DOWNLOADS,
  INITIAL_HISTORY,
  INITIAL_PRIVATE_TABS,
  INITIAL_TABS,
} from './data/mockData';
import { StatusBar } from './components/StatusBar';
import { NavigationBar } from './components/NavigationBar';
import { Omnibar } from './components/Omnibar';
import { HomeScreen } from './components/HomeScreen';
import { WebPageContent } from './components/WebPageContent';
import { TabSwitcher } from './components/TabSwitcher';
import { ReaderView } from './components/ReaderView';
import { BrowserMenu } from './components/BrowserMenu';
import { BookmarksHistoryModal } from './components/BookmarksHistoryModal';
import { DownloadsModal } from './components/DownloadsModal';
import { SettingsModal } from './components/SettingsModal';
import { AndroidDeviceFrame } from './components/AndroidDeviceFrame';
import { DeviceControlsBar } from './components/DeviceControlsBar';

const INITIAL_SETTINGS: BrowserSettings = {
  theme: 'dark',
  androidVersion: 'android10',
  navType: 'three_button',
  addressBarPosition: 'top',
  searchEngine: 'duckduckgo',
  blockAds: true,
  blockTrackers: true,
  blockImages: false,
  javaScriptEnabled: true,
  desktopMode: false,
  clearOnExit: false,
  fontSize: 'medium',
  readerModeTheme: 'dark',
};

export default function App() {
  const [settings, setSettings] = useState<BrowserSettings>(INITIAL_SETTINGS);
  const [deviceFrameEnabled, setDeviceFrameEnabled] = useState(true);

  // Tabs management
  const [tabs, setTabs] = useState<Tab[]>(INITIAL_TABS);
  const [privateTabs, setPrivateTabs] = useState<Tab[]>(INITIAL_PRIVATE_TABS);
  const [activeTabId, setActiveTabId] = useState<string>('tab-1');
  const [isCurrentTabPrivate, setIsCurrentTabPrivate] = useState<boolean>(false);

  // UI Views and Modals
  const [isTabSwitcherOpen, setIsTabSwitcherOpen] = useState(false);
  const [isReaderModeOpen, setIsReaderModeOpen] = useState(false);
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isBookmarksOpen, setIsBookmarksOpen] = useState(false);
  const [isHistoryOpen, setIsHistoryOpen] = useState(false);
  const [isDownloadsOpen, setIsDownloadsOpen] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  // Data
  const [bookmarks, setBookmarks] = useState<Bookmark[]>(DEFAULT_BOOKMARKS);
  const [history, setHistory] = useState<HistoryItem[]>(INITIAL_HISTORY);
  const [downloads, setDownloads] = useState<DownloadItem[]>(INITIAL_DOWNLOADS);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  // Active Tab Reference
  const currentTabList = isCurrentTabPrivate ? privateTabs : tabs;
  const currentTab =
    currentTabList.find((t) => t.id === activeTabId) ||
    currentTabList[0] || {
      id: 'tab-fallback',
      title: 'Быстрый доступ',
      url: 'aura://start',
      isPrivate: isCurrentTabPrivate,
      pageType: 'home',
      canGoBack: false,
      canGoForward: false,
    };

  // Toast notification helper
  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => {
      setToastMessage(null);
    }, 2500);
  };

  // Navigation Logic
  const handleNavigate = (inputUrl: string) => {
    setIsLoading(true);

    let resolvedType: Tab['pageType'] = 'custom';
    let cleanUrl = inputUrl.trim();
    let title = cleanUrl;
    let query = '';

    if (
      cleanUrl === 'aura://start' ||
      cleanUrl === 'about:blank' ||
      cleanUrl === 'start'
    ) {
      resolvedType = 'home';
      cleanUrl = 'aura://start';
      title = 'Быстрый доступ';
    } else if (cleanUrl.includes('habr.com')) {
      resolvedType = 'article_habr';
      cleanUrl = 'https://habr.com/ru/articles/android-optimization';
      title = 'Хабр: Оптимизация под Android 10';
    } else if (cleanUrl.includes('wikipedia.org')) {
      resolvedType = 'article_wiki';
      cleanUrl = 'https://ru.wikipedia.org/wiki/Android_10';
      title = 'Википедия: Android 10 (Q)';
    } else if (
      !cleanUrl.includes('.') ||
      cleanUrl.includes(' ') ||
      cleanUrl.startsWith('?')
    ) {
      // Search query
      resolvedType = 'search_results';
      query = cleanUrl.replace(/^\?/, '').trim();
      cleanUrl = `https://${settings.searchEngine}.com/search?q=${encodeURIComponent(
        query
      )}`;
      title = `Поиск: ${query}`;
    } else {
      if (!cleanUrl.startsWith('http://') && !cleanUrl.startsWith('https://')) {
        cleanUrl = 'https://' + cleanUrl;
      }
      resolvedType = 'custom';
      title = cleanUrl.replace(/https?:\/\//, '');
    }

    // Update active tab
    const updater = (prevTabs: Tab[]) =>
      prevTabs.map((t) => {
        if (t.id === currentTab.id) {
          return {
            ...t,
            url: cleanUrl,
            title,
            pageType: resolvedType,
            searchQuery: query,
            canGoBack: true,
          };
        }
        return t;
      });

    if (isCurrentTabPrivate) {
      setPrivateTabs(updater);
    } else {
      setTabs(updater);
      // Add to history if not in private mode and not start page
      if (cleanUrl !== 'aura://start') {
        const newHistItem: HistoryItem = {
          id: `hist-${Date.now()}`,
          title,
          url: cleanUrl,
          visitedAt: 'Только что',
          isPrivate: false,
        };
        setHistory((prev) => [newHistItem, ...prev]);
      }
    }

    // Reset modals & reader
    setIsReaderModeOpen(false);
    setIsTabSwitcherOpen(false);

    // Simulated loading delay
    setTimeout(() => {
      setIsLoading(false);
    }, 450);
  };

  // Hardware/System Nav Bar Handlers
  const handleSystemBack = () => {
    if (isReaderModeOpen) {
      setIsReaderModeOpen(false);
      return;
    }
    if (isTabSwitcherOpen) {
      setIsTabSwitcherOpen(false);
      return;
    }
    if (isBookmarksOpen || isHistoryOpen || isDownloadsOpen || isSettingsOpen) {
      setIsBookmarksOpen(false);
      setIsHistoryOpen(false);
      setIsDownloadsOpen(false);
      setIsSettingsOpen(false);
      return;
    }
    if (currentTab.canGoBack && currentTab.pageType !== 'home') {
      handleNavigate('aura://start');
    } else {
      showToast('Начальный экран браузера');
    }
  };

  const handleSystemHome = () => {
    setIsReaderModeOpen(false);
    setIsTabSwitcherOpen(false);
    setIsBookmarksOpen(false);
    setIsHistoryOpen(false);
    setIsDownloadsOpen(false);
    setIsSettingsOpen(false);
    handleNavigate('aura://start');
  };

  const handleSystemRecents = () => {
    setIsTabSwitcherOpen((prev) => !prev);
  };

  const handleReload = () => {
    setIsLoading(true);
    setTimeout(() => {
      setIsLoading(false);
      showToast('Страница обновлена');
    }, 400);
  };

  // Tab Operations
  const handleNewTab = (isPrivate: boolean) => {
    const newId = `tab-${Date.now()}`;
    const newTabObj: Tab = {
      id: newId,
      title: isPrivate ? 'Приватная вкладка' : 'Быстрый доступ',
      url: 'aura://start',
      isPrivate,
      pageType: 'home',
      canGoBack: false,
      canGoForward: false,
    };

    if (isPrivate) {
      setPrivateTabs((prev) => [...prev, newTabObj]);
      setIsCurrentTabPrivate(true);
    } else {
      setTabs((prev) => [...prev, newTabObj]);
      setIsCurrentTabPrivate(false);
    }
    setActiveTabId(newId);
    setIsTabSwitcherOpen(false);
    setIsReaderModeOpen(false);
  };

  const handleSelectTab = (id: string, isPrivate: boolean) => {
    setActiveTabId(id);
    setIsCurrentTabPrivate(isPrivate);
    setIsTabSwitcherOpen(false);
    setIsReaderModeOpen(false);
  };

  const handleCloseTab = (id: string, isPrivate: boolean) => {
    if (isPrivate) {
      const remaining = privateTabs.filter((t) => t.id !== id);
      setPrivateTabs(remaining);
      if (activeTabId === id && remaining.length > 0) {
        setActiveTabId(remaining[0].id);
      } else if (remaining.length === 0) {
        handleNewTab(true);
      }
    } else {
      const remaining = tabs.filter((t) => t.id !== id);
      setTabs(remaining);
      if (activeTabId === id && remaining.length > 0) {
        setActiveTabId(remaining[0].id);
      } else if (remaining.length === 0) {
        handleNewTab(false);
      }
    }
  };

  const handleCloseAllTabs = (isPrivate: boolean) => {
    if (isPrivate) {
      setPrivateTabs([]);
      handleNewTab(true);
      showToast('Все приватные вкладки закрыты');
    } else {
      setTabs([]);
      handleNewTab(false);
      showToast('Все вкладки закрыты');
    }
  };

  // Bookmarks & Downloads
  const handleAddBookmark = (title: string, url: string) => {
    const newBm: Bookmark = {
      id: `bm-${Date.now()}`,
      title,
      url,
      category: 'Пользовательские',
      iconName: 'Globe',
      color: 'bg-blue-600',
    };
    setBookmarks((prev) => [...prev, newBm]);
    showToast(`Ярлык «${title}» добавлен`);
  };

  const handleDeleteBookmark = (id: string) => {
    setBookmarks((prev) => prev.filter((b) => b.id !== id));
    showToast('Закладка удалена');
  };

  const handleBookmarkCurrent = () => {
    if (currentTab.url === 'aura://start') {
      showToast('Стартовая страница уже сохранена');
      return;
    }
    const alreadySaved = bookmarks.some((b) => b.url === currentTab.url);
    if (alreadySaved) {
      showToast('Страница уже в закладках');
    } else {
      handleAddBookmark(currentTab.title, currentTab.url);
    }
  };

  const handleClearHistory = () => {
    setHistory([]);
    showToast('История посещений очищена');
  };

  const handleDeleteHistoryItem = (id: string) => {
    setHistory((prev) => prev.filter((h) => h.id !== id));
  };

  const handleClearDownloads = () => {
    setDownloads([]);
    showToast('Список загрузок очищен');
  };

  const handleToggleDownloadStatus = (id: string) => {
    setDownloads((prev) =>
      prev.map((d) => {
        if (d.id === id) {
          const nextStatus = d.status === 'downloading' ? 'paused' : 'downloading';
          return { ...d, status: nextStatus };
        }
        return d;
      })
    );
  };

  const handleClearSession = () => {
    setHistory([]);
    setPrivateTabs([]);
    handleNavigate('aura://start');
    showToast('Сессия и кэш очищены (Android WebView ClearCache)');
  };

  const isArticle =
    currentTab.pageType === 'article_habr' ||
    currentTab.pageType === 'article_wiki';

  const isDarkTheme =
    settings.theme === 'dark' ||
    settings.theme === 'amoled' ||
    isCurrentTabPrivate;

  return (
    <div className="min-h-screen bg-neutral-950 text-neutral-100 flex flex-col font-sans select-none antialiased">
      {/* Top Device & Environment Testing Controls Bar */}
      <DeviceControlsBar
        settings={settings}
        deviceFrameEnabled={deviceFrameEnabled}
        onUpdateSettings={(newSettings) =>
          setSettings((prev) => ({ ...prev, ...newSettings }))
        }
        onToggleDeviceFrame={() => setDeviceFrameEnabled((prev) => !prev)}
        onResetApp={handleSystemHome}
        onOpenSettings={() => setIsSettingsOpen(true)}
      />

      {/* Main Presentation Viewport */}
      <main className="flex-1 flex items-center justify-center p-0 sm:p-4 overflow-hidden relative">
        <AndroidDeviceFrame
          enabled={deviceFrameEnabled}
          theme={settings.theme}
          androidVersion={settings.androidVersion}
          deviceColor="obsidian"
        >
          {/* Inner Browser Container */}
          <div
            className={`w-full h-full flex flex-col overflow-hidden relative ${
              settings.theme === 'amoled' && isDarkTheme
                ? 'bg-black text-neutral-100'
                : isDarkTheme
                ? 'bg-neutral-900 text-neutral-100'
                : 'bg-white text-neutral-900'
            }`}
          >
            {/* 1. Android Status Bar (System UI) */}
            <StatusBar
              theme={settings.theme}
              androidVersion={settings.androidVersion}
              currentTime="10:42"
              isPrivate={isCurrentTabPrivate}
              hasActiveDownload={downloads.some((d) => d.status === 'downloading')}
            />

            {/* 2. Top Omnibar (if configured at top) */}
            {settings.addressBarPosition === 'top' && !isTabSwitcherOpen && !isReaderModeOpen && (
              <Omnibar
                url={currentTab.url}
                title={currentTab.title}
                theme={settings.theme}
                position="top"
                isPrivate={isCurrentTabPrivate}
                isLoading={isLoading}
                canGoBack={currentTab.canGoBack}
                canGoForward={currentTab.canGoForward}
                tabsCount={currentTabList.length}
                isReaderModeActive={isReaderModeOpen}
                isArticlePage={isArticle}
                onNavigate={handleNavigate}
                onReload={handleReload}
                onGoBack={handleSystemBack}
                onGoForward={() => {}}
                onToggleReaderMode={() => setIsReaderModeOpen((prev) => !prev)}
                onOpenTabSwitcher={() => setIsTabSwitcherOpen(true)}
                onOpenMenu={() => setIsMenuOpen((prev) => !prev)}
                onHomeClick={handleSystemHome}
              />
            )}

            {/* 3. Main Screen Content Switcher */}
            <div className="flex-1 flex flex-col overflow-hidden relative">
              {isTabSwitcherOpen ? (
                <TabSwitcher
                  tabs={tabs}
                  privateTabs={privateTabs}
                  activeTabId={activeTabId}
                  theme={settings.theme}
                  onSelectTab={handleSelectTab}
                  onCloseTab={handleCloseTab}
                  onNewTab={handleNewTab}
                  onCloseAllTabs={handleCloseAllTabs}
                  onCloseSwitcher={() => setIsTabSwitcherOpen(false)}
                />
              ) : isReaderModeOpen ? (
                <ReaderView
                  title={currentTab.title}
                  url={currentTab.url}
                  settings={settings}
                  onClose={() => setIsReaderModeOpen(false)}
                />
              ) : currentTab.pageType === 'home' ? (
                <HomeScreen
                  theme={settings.theme}
                  isPrivate={isCurrentTabPrivate}
                  bookmarks={bookmarks}
                  settings={settings}
                  onNavigate={handleNavigate}
                  onOpenBookmarks={() => setIsBookmarksOpen(true)}
                  onOpenDownloads={() => setIsDownloadsOpen(true)}
                  onOpenHistory={() => setIsHistoryOpen(true)}
                  onOpenSettings={() => setIsSettingsOpen(true)}
                  onAddBookmark={handleAddBookmark}
                  onDeleteBookmark={handleDeleteBookmark}
                  onSearch={(q) => handleNavigate(q)}
                  onSelectSearchEngine={(eng) =>
                    setSettings((prev) => ({ ...prev, searchEngine: eng }))
                  }
                />
              ) : (
                <WebPageContent
                  pageType={currentTab.pageType}
                  url={currentTab.url}
                  theme={settings.theme}
                  settings={settings}
                  searchQuery={currentTab.searchQuery}
                  onNavigate={handleNavigate}
                  onBookmarkCurrent={handleBookmarkCurrent}
                />
              )}

              {/* Overflow Menu Popup */}
              <BrowserMenu
                isOpen={isMenuOpen}
                theme={settings.theme}
                position={settings.addressBarPosition}
                settings={settings}
                isArticlePage={isArticle}
                onClose={() => setIsMenuOpen(false)}
                onNewTab={() => handleNewTab(false)}
                onNewPrivateTab={() => handleNewTab(true)}
                onAddBookmark={handleBookmarkCurrent}
                onToggleReaderMode={() => setIsReaderModeOpen((prev) => !prev)}
                onToggleDesktopMode={() =>
                  setSettings((prev) => ({
                    ...prev,
                    desktopMode: !prev.desktopMode,
                  }))
                }
                onToggleAdBlock={() =>
                  setSettings((prev) => ({
                    ...prev,
                    blockAds: !prev.blockAds,
                  }))
                }
                onToggleBlockImages={() =>
                  setSettings((prev) => ({
                    ...prev,
                    blockImages: !prev.blockImages,
                  }))
                }
                onToggleTheme={() =>
                  setSettings((prev) => ({
                    ...prev,
                    theme:
                      prev.theme === 'light'
                        ? 'dark'
                        : prev.theme === 'dark'
                        ? 'amoled'
                        : 'light',
                  }))
                }
                onOpenBookmarks={() => setIsBookmarksOpen(true)}
                onOpenHistory={() => setIsHistoryOpen(true)}
                onOpenDownloads={() => setIsDownloadsOpen(true)}
                onOpenSettings={() => setIsSettingsOpen(true)}
                onClearSession={handleClearSession}
              />
            </div>

            {/* 4. Bottom Omnibar (if configured at bottom for one-handed reachability) */}
            {settings.addressBarPosition === 'bottom' &&
              !isTabSwitcherOpen &&
              !isReaderModeOpen && (
                <Omnibar
                  url={currentTab.url}
                  title={currentTab.title}
                  theme={settings.theme}
                  position="bottom"
                  isPrivate={isCurrentTabPrivate}
                  isLoading={isLoading}
                  canGoBack={currentTab.canGoBack}
                  canGoForward={currentTab.canGoForward}
                  tabsCount={currentTabList.length}
                  isReaderModeActive={isReaderModeOpen}
                  isArticlePage={isArticle}
                  onNavigate={handleNavigate}
                  onReload={handleReload}
                  onGoBack={handleSystemBack}
                  onGoForward={() => {}}
                  onToggleReaderMode={() =>
                    setIsReaderModeOpen((prev) => !prev)
                  }
                  onOpenTabSwitcher={() => setIsTabSwitcherOpen(true)}
                  onOpenMenu={() => setIsMenuOpen((prev) => !prev)}
                  onHomeClick={handleSystemHome}
                />
              )}

            {/* 5. Android Navigation Bar (3-button or Gesture pill) */}
            <NavigationBar
              navType={settings.navType}
              theme={settings.theme}
              isPrivate={isCurrentTabPrivate}
              onBack={handleSystemBack}
              onHome={handleSystemHome}
              onRecents={handleSystemRecents}
            />

            {/* Android Toast notification bubble */}
            {toastMessage && (
              <div className="absolute bottom-14 left-1/2 -translate-x-1/2 z-50 px-4 py-2 bg-neutral-900/90 text-neutral-100 text-xs rounded-full border border-neutral-700/80 shadow-lg backdrop-blur-sm animate-in fade-in slide-in-from-bottom-2 duration-150">
                {toastMessage}
              </div>
            )}
          </div>
        </AndroidDeviceFrame>
      </main>

      {/* Bookmarks & History Modal */}
      <BookmarksHistoryModal
        isOpen={isBookmarksOpen || isHistoryOpen}
        initialTab={isBookmarksOpen ? 'bookmarks' : 'history'}
        theme={settings.theme}
        bookmarks={bookmarks}
        history={history}
        onClose={() => {
          setIsBookmarksOpen(false);
          setIsHistoryOpen(false);
        }}
        onSelectUrl={handleNavigate}
        onDeleteBookmark={handleDeleteBookmark}
        onClearHistory={handleClearHistory}
        onDeleteHistoryItem={handleDeleteHistoryItem}
      />

      {/* Downloads Sheet */}
      <DownloadsModal
        isOpen={isDownloadsOpen}
        theme={settings.theme}
        downloads={downloads}
        onClose={() => setIsDownloadsOpen(false)}
        onClearDownloads={handleClearDownloads}
        onToggleDownloadStatus={handleToggleDownloadStatus}
      />

      {/* Settings Modal */}
      <SettingsModal
        isOpen={isSettingsOpen}
        settings={settings}
        onClose={() => setIsSettingsOpen(false)}
        onUpdateSettings={(newSettings) =>
          setSettings((prev) => ({ ...prev, ...newSettings }))
        }
        onResetSettings={() => setSettings(INITIAL_SETTINGS)}
      />
    </div>
  );
}
