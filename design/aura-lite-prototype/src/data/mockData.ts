import { Bookmark, DownloadItem, HistoryItem, Tab } from '../types/browser';

export const DEFAULT_BOOKMARKS: Bookmark[] = [
  {
    id: 'bm-1',
    title: 'Wikipedia',
    url: 'https://ru.wikipedia.org',
    category: 'Энциклопедия',
    iconName: 'BookOpen',
    color: 'bg-neutral-600',
  },
  {
    id: 'bm-2',
    title: 'Хабр (Habr)',
    url: 'https://habr.com/ru',
    category: 'IT-статьи',
    iconName: 'Code',
    color: 'bg-sky-600',
  },
  {
    id: 'bm-3',
    title: '4PDA',
    url: 'https://4pda.to',
    category: 'Android & Прошивки',
    iconName: 'Smartphone',
    color: 'bg-emerald-600',
  },
  {
    id: 'bm-4',
    title: 'GitHub',
    url: 'https://github.com',
    category: 'Разработка',
    iconName: 'GitBranch',
    color: 'bg-neutral-800',
  },
  {
    id: 'bm-5',
    title: 'Погода',
    url: 'https://weather.lite.net',
    category: 'Погода',
    iconName: 'CloudSun',
    color: 'bg-amber-600',
  },
  {
    id: 'bm-6',
    title: 'Reddit Lite',
    url: 'https://teddit.net',
    category: 'Сообщества',
    iconName: 'MessageSquare',
    color: 'bg-orange-600',
  },
  {
    id: 'bm-7',
    title: 'OpenStreetMap',
    url: 'https://osm.org',
    category: 'Карты',
    iconName: 'MapPin',
    color: 'bg-teal-600',
  },
  {
    id: 'bm-8',
    title: 'Архив Веба',
    url: 'https://archive.org',
    category: 'Архив',
    iconName: 'Archive',
    color: 'bg-indigo-600',
  },
];

export const INITIAL_TABS: Tab[] = [
  {
    id: 'tab-1',
    title: 'Быстрый доступ',
    url: 'aura://start',
    isPrivate: false,
    pageType: 'home',
    canGoBack: false,
    canGoForward: false,
  },
  {
    id: 'tab-2',
    title: 'Хабр: Оптимизация под Android 10 и Go Edition',
    url: 'https://habr.com/ru/articles/android-optimization',
    isPrivate: false,
    pageType: 'article_habr',
    canGoBack: true,
    canGoForward: false,
  },
  {
    id: 'tab-3',
    title: 'Википедия: Android 10 (Queen Cake)',
    url: 'https://ru.wikipedia.org/wiki/Android_10',
    isPrivate: false,
    pageType: 'article_wiki',
    canGoBack: true,
    canGoForward: false,
  },
];

export const INITIAL_PRIVATE_TABS: Tab[] = [
  {
    id: 'tab-priv-1',
    title: 'Приватная вкладка',
    url: 'aura://start',
    isPrivate: true,
    pageType: 'home',
    canGoBack: false,
    canGoForward: false,
  },
];

export const INITIAL_HISTORY: HistoryItem[] = [
  {
    id: 'hist-1',
    title: 'Хабр: Оптимизация под Android 10 и Go Edition',
    url: 'https://habr.com/ru/articles/android-optimization',
    visitedAt: 'Сегодня, 10:24',
    isPrivate: false,
  },
  {
    id: 'hist-2',
    title: 'Википедия: Android 10 (Queen Cake)',
    url: 'https://ru.wikipedia.org/wiki/Android_10',
    visitedAt: 'Сегодня, 09:41',
    isPrivate: false,
  },
  {
    id: 'hist-3',
    title: '4PDA: Легковесные браузеры с WebView',
    url: 'https://4pda.to/forum/index.php?showtopic=lite_browsers',
    visitedAt: 'Вчера, 18:15',
    isPrivate: false,
  },
  {
    id: 'hist-4',
    title: 'GitHub: Chromium Blink minimal webview wrapper',
    url: 'https://github.com/topics/android-lightweight-browser',
    visitedAt: 'Вчера, 14:02',
    isPrivate: false,
  },
  {
    id: 'hist-5',
    title: 'OpenStreetMap: Центральный район',
    url: 'https://osm.org/query?center',
    visitedAt: '20 окт, 11:30',
    isPrivate: false,
  },
];

export const INITIAL_DOWNLOADS: DownloadItem[] = [
  {
    id: 'dl-1',
    fileName: 'android10_api29_developer_guide.pdf',
    fileSize: '3.4 МБ',
    progress: 100,
    status: 'completed',
    date: 'Сегодня, 09:12',
    fileType: 'pdf',
  },
  {
    id: 'dl-2',
    fileName: 'aura_browser_lite_v2.4_release.apk',
    fileSize: '2.1 МБ',
    progress: 100,
    status: 'completed',
    date: 'Вчера, 20:45',
    fileType: 'apk',
  },
  {
    id: 'dl-3',
    fileName: 'dark_wallpaper_amoled_1080p.png',
    fileSize: '1.2 МБ',
    progress: 68,
    status: 'downloading',
    date: 'Сегодня, 10:35',
    fileType: 'image',
  },
];
