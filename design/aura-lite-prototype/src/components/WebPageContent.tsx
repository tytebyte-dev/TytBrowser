import React from 'react';
import {
  ExternalLink,
  Shield,
  Clock,
  User,
  Share2,
  BookmarkPlus,
  ImageOff,
  Search,
  CheckCircle,
  FileText,
} from 'lucide-react';
import { BrowserSettings, ThemeMode } from '../types/browser';

interface WebPageContentProps {
  pageType: 'home' | 'article_habr' | 'article_wiki' | 'article_tech' | 'search_results' | 'custom';
  url: string;
  theme: ThemeMode;
  settings: BrowserSettings;
  searchQuery?: string;
  onNavigate: (url: string) => void;
  onBookmarkCurrent: () => void;
}

export const WebPageContent: React.FC<WebPageContentProps> = ({
  pageType,
  url,
  theme,
  settings,
  searchQuery = '',
  onNavigate,
  onBookmarkCurrent,
}) => {
  const isDark = theme === 'dark' || theme === 'amoled';

  const containerBg =
    theme === 'amoled' && isDark
      ? 'bg-black text-neutral-200'
      : isDark
      ? 'bg-neutral-900 text-neutral-200'
      : 'bg-white text-neutral-800';

  const cardBg = isDark ? 'bg-neutral-800/80' : 'bg-neutral-50';
  const borderCol = isDark ? 'border-neutral-800' : 'border-neutral-200';

  // Desktop view scaling simulation
  const desktopWrapperClass = settings.desktopMode ? 'min-w-[760px] text-xs' : 'w-full';

  if (pageType === 'article_habr') {
    return (
      <div className={`flex-1 w-full overflow-y-auto px-4 py-5 ${containerBg}`}>
        <div className={`max-w-2xl mx-auto space-y-4 ${desktopWrapperClass}`}>
          {/* Header Metadata */}
          <div className="flex items-center justify-between text-xs text-neutral-400 pb-2 border-b border-neutral-800/40">
            <span className="font-semibold text-sky-500">Хабр / Разработка для Android</span>
            <div className="flex items-center gap-2">
              <span className="tabular-nums">Сегодня, 10:20</span>
              <span>·</span>
              <span>4 мин</span>
            </div>
          </div>

          <h1 className="text-lg md:text-xl font-bold leading-tight tracking-tight">
            Оптимизация WebView и потребления ОЗУ под Android 10 (Go Edition) и старые смартфоны
          </h1>

          <div className="flex items-center justify-between text-xs text-neutral-400">
            <div className="flex items-center gap-2">
              <div className="w-6 h-6 rounded-full bg-sky-700 text-white flex items-center justify-center font-bold text-[10px]">
                IT
              </div>
              <span className="font-medium text-neutral-300">Александр Васильев</span>
            </div>
            <button
              onClick={onBookmarkCurrent}
              className="flex items-center gap-1 text-blue-500 hover:text-blue-400 text-xs"
            >
              <BookmarkPlus size={14} />
              <span>В закладки</span>
            </button>
          </div>

          {/* Low-RAM image block notice or illustration */}
          {settings.blockImages ? (
            <div className={`p-4 rounded-xl border border-dashed flex items-center gap-3 ${borderCol} ${cardBg}`}>
              <ImageOff size={22} className="text-amber-500 shrink-0" />
              <div className="text-xs">
                <span className="font-semibold text-amber-500 block">
                  Режим экономии трафика: Изображение скрыто
                </span>
                <span className="text-neutral-400">Сэкономлено ~480 КБ оперативной памяти и мобильного трафика.</span>
              </div>
            </div>
          ) : (
            <div className={`p-4 rounded-xl border ${borderCol} ${cardBg}`}>
              <div className="h-32 bg-gradient-to-tr from-sky-950 via-slate-900 to-indigo-950 rounded-lg flex flex-col items-center justify-center text-center p-3 text-sky-200">
                <span className="text-xs font-mono uppercase tracking-widest text-sky-400 mb-1">
                  Android Memory Profile
                </span>
                <span className="text-sm font-semibold">
                  Chrome (380 MB) vs Aura Lite (28 MB)
                </span>
                <span className="text-[10px] text-sky-300/70 mt-1">
                  Архитектурный замер на устройстве с 1.5 GB RAM (Android 10 Go)
                </span>
              </div>
            </div>
          )}

          {/* Article Prose */}
          <div className="text-xs md:text-sm leading-relaxed space-y-3.5 text-neutral-300">
            <p>
              Смартфоны под управлением Android 10 и более ранних версий (Android 9 Pie, Android 8 Oreo)
              часто имеют ограниченный объем оперативной памяти (1–2 ГБ). Стандартные сборки Chromium
              потребляют от 300 до 600 МБ сразу после запуска десятка вкладок, что приводит к вытеснению
              фоновых приложений из памяти (Low Memory Killer).
            </p>

            <h3 className="text-sm font-bold text-neutral-100 pt-2">
              1. Преимущества системного Android WebView
            </h3>
            <p>
              В отличие от монолитного Chrome, легковесный браузер использует уже загруженный в память
              системный компонент <code className="text-sky-400 font-mono text-xs">com.google.android.webview</code>.
              Это позволяет уменьшить размер самого APK до 2-3 МБ и снизить холодный запуск до 150 миллисекунд.
            </p>

            <div className={`p-3 rounded-lg border font-mono text-[11px] leading-normal ${cardBg} ${borderCol}`}>
              <span className="text-neutral-400">// Настройка WebSettings для экономии памяти</span>
              <br />
              webSettings.setRenderPriority(RenderPriority.HIGH);
              <br />
              webSettings.setCacheMode(WebSettings.LOAD_DEFAULT);
              <br />
              webSettings.setAppCacheEnabled(true);
            </div>

            <h3 className="text-sm font-bold text-neutral-100 pt-2">
              2. Тёмная тема в Android 10 (Q)
            </h3>
            <p>
              Именно в Android 10 Google стандартизировал глобальный переключатель Dark Theme.
              Для OLED-экранов на бюджетных устройствах режим с чистым черным фоном снижает энергопотребление
              дисплея до 40% при веб-серфинге.
            </p>
          </div>

          <div className="pt-4 border-t border-neutral-800/40 flex justify-between items-center text-xs text-neutral-400">
            <span>Метки: android, webview, optimization, lite</span>
            <span className="text-emerald-500 font-semibold">1.4k просмотров</span>
          </div>
        </div>
      </div>
    );
  }

  if (pageType === 'article_wiki') {
    return (
      <div className={`flex-1 w-full overflow-y-auto px-4 py-5 ${containerBg}`}>
        <div className={`max-w-2xl mx-auto space-y-4 ${desktopWrapperClass}`}>
          <div className="flex items-center justify-between pb-2 border-b border-neutral-800/40 text-xs">
            <span className="font-semibold text-neutral-400">Свободная энциклопедия</span>
            <span className="text-neutral-500 text-[11px]">ru.wikipedia.org</span>
          </div>

          <h1 className="text-xl font-bold leading-tight">Android 10</h1>

          <div className={`p-3 rounded-xl border ${cardBg} ${borderCol} text-xs space-y-1.5`}>
            <div className="font-bold text-neutral-200">Краткая карточка версии:</div>
            <div className="flex justify-between text-neutral-400">
              <span>Кодовое имя:</span>
              <span className="text-neutral-200 font-mono">Queen Cake (Q)</span>
            </div>
            <div className="flex justify-between text-neutral-400">
              <span>API Level:</span>
              <span className="text-neutral-200 font-mono">29</span>
            </div>
            <div className="flex justify-between text-neutral-400">
              <span>Дата релиза:</span>
              <span className="text-neutral-200">3 сентября 2019</span>
            </div>
            <div className="flex justify-between text-neutral-400">
              <span>Навигация:</span>
              <span className="text-neutral-200">Жестовая система + 3 классические кнопки</span>
            </div>
          </div>

          <div className="text-xs md:text-sm leading-relaxed space-y-3 text-neutral-300">
            <p>
              <strong>Android 10</strong> — десятая версия мобильной операционной системы Android,
              разработанная Google. Это первый релиз, официально отказавшийся от десертных названий в маркетинге.
            </p>
            <h3 className="text-sm font-bold text-neutral-100 pt-1">Ключевые новшества:</h3>
            <ul className="list-disc list-inside space-y-1 text-neutral-300 pl-1">
              <li>
                <strong>Системная темная тема (Dark Theme):</strong> нативное API для переключения темы в приложениях.
              </li>
              <li>
                <strong>Жестовая навигация:</strong> отказ от кнопки назад в пользу свайпа от края экрана и тонкой нижней полосы.
              </li>
              <li>
                <strong>Scoped Storage:</strong> изолированное дисковое хранилище для защиты пользовательских файлов.
              </li>
              <li>
                <strong>Project Mainline:</strong> модульные обновления системных компонентов безопасности через Google Play.
              </li>
            </ul>
          </div>
        </div>
      </div>
    );
  }

  if (pageType === 'search_results') {
    return (
      <div className={`flex-1 w-full overflow-y-auto px-4 py-5 ${containerBg}`}>
        <div className={`max-w-2xl mx-auto space-y-4 ${desktopWrapperClass}`}>
          <div className="flex items-center gap-2 pb-2 border-b border-neutral-800/40 text-xs">
            <Search size={14} className="text-blue-500" />
            <span className="text-neutral-400">Результаты поиска для:</span>
            <span className="font-semibold text-neutral-200">"{searchQuery || 'легковесный браузер'}"</span>
          </div>

          <div className="space-y-4">
            <div
              onClick={() => onNavigate('https://habr.com/ru/articles/android-optimization')}
              className={`p-3 rounded-xl border cursor-pointer hover:border-blue-500/50 transition-colors ${cardBg} ${borderCol}`}
            >
              <div className="text-[11px] text-emerald-500 font-mono mb-0.5">https://habr.com/ru/articles/android-optimization</div>
              <h3 className="text-sm font-bold text-blue-400 hover:underline">
                Оптимизация WebView и потребления ОЗУ под Android 10
              </h3>
              <p className="text-xs text-neutral-400 mt-1">
                Практическое руководство по созданию ультра-быстрого браузера с потреблением памяти до 30 МБ.
              </p>
            </div>

            <div
              onClick={() => onNavigate('https://ru.wikipedia.org/wiki/Android_10')}
              className={`p-3 rounded-xl border cursor-pointer hover:border-blue-500/50 transition-colors ${cardBg} ${borderCol}`}
            >
              <div className="text-[11px] text-emerald-500 font-mono mb-0.5">https://ru.wikipedia.org/wiki/Android_10</div>
              <h3 className="text-sm font-bold text-blue-400 hover:underline">
                Android 10 — Википедия
              </h3>
              <p className="text-xs text-neutral-400 mt-1">
                Подробная история операционной системы Android версии 10: архитектура, системные жесты и тёмная тема.
              </p>
            </div>

            <div
              onClick={() => onNavigate('https://4pda.to/forum/index.php?showtopic=lite_browsers')}
              className={`p-3 rounded-xl border cursor-pointer hover:border-blue-500/50 transition-colors ${cardBg} ${borderCol}`}
            >
              <div className="text-[11px] text-emerald-500 font-mono mb-0.5">https://4pda.to/forum/lite_browsers</div>
              <h3 className="text-sm font-bold text-blue-400 hover:underline">
                Клуб любителей легковесных браузеров для слабых устройств (Android 5.0 - 10)
              </h3>
              <p className="text-xs text-neutral-400 mt-1">
                Обсуждение браузеров Via, Lightning, Monument, FOSS Browser и авторских модификаций без рекламы.
              </p>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Custom URL or generic loaded page
  return (
    <div className={`flex-1 w-full overflow-y-auto px-4 py-8 flex flex-col items-center justify-center text-center ${containerBg}`}>
      <div className="max-w-md p-6 rounded-2xl border space-y-3">
        <div className="w-12 h-12 mx-auto rounded-full bg-blue-500/20 text-blue-400 flex items-center justify-center">
          <FileText size={24} />
        </div>
        <h3 className="text-base font-bold text-neutral-100">
          Загружена веб-страница
        </h3>
        <p className="text-xs font-mono text-neutral-400 break-all">
          {url}
        </p>
        <p className="text-xs text-neutral-400 leading-relaxed">
          Легковесный движок Aura Lite отрендерил страницу с активной защитой от трекеров и оптимизацией кеша для Android 10.
        </p>
        <div className="pt-2">
          <button
            onClick={() => onNavigate('aura://start')}
            className="px-4 py-2 text-xs font-semibold bg-neutral-800 hover:bg-neutral-700 text-white rounded-lg transition-colors"
          >
            Вернуться на главную
          </button>
        </div>
      </div>
    </div>
  );
};
