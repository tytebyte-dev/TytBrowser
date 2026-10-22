export type ThemeMode = 'light' | 'dark' | 'amoled';

export type AndroidVersion = 'android10' | 'android9' | 'android8';

export type NavType = 'three_button' | 'gesture' | 'hidden';

export type AddressBarPosition = 'top' | 'bottom';

export type SearchEngine = 'duckduckgo' | 'google' | 'yandex' | 'bing';

export interface Tab {
  id: string;
  title: string;
  url: string;
  favicon?: string;
  isPrivate: boolean;
  pageType: 'home' | 'article_habr' | 'article_wiki' | 'article_tech' | 'search_results' | 'custom';
  searchQuery?: string;
  canGoBack: boolean;
  canGoForward: boolean;
  scrollPosition?: number;
}

export interface Bookmark {
  id: string;
  title: string;
  url: string;
  category: string;
  iconName: string;
  color: string;
}

export interface HistoryItem {
  id: string;
  title: string;
  url: string;
  visitedAt: string;
  isPrivate: boolean;
}

export interface DownloadItem {
  id: string;
  fileName: string;
  fileSize: string;
  progress: number; // 0 - 100
  status: 'completed' | 'downloading' | 'paused';
  date: string;
  fileType: 'pdf' | 'apk' | 'image' | 'zip';
}

export interface BrowserSettings {
  theme: ThemeMode;
  androidVersion: AndroidVersion;
  navType: NavType;
  addressBarPosition: AddressBarPosition;
  searchEngine: SearchEngine;
  blockAds: boolean;
  blockTrackers: boolean;
  blockImages: boolean; // low-RAM / data saver
  javaScriptEnabled: boolean;
  desktopMode: boolean;
  clearOnExit: boolean;
  fontSize: 'small' | 'medium' | 'large';
  readerModeTheme: 'light' | 'sepia' | 'dark';
}
