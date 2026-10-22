import React from 'react';
import {
  Settings,
  Moon,
  Sun,
  Smartphone,
  Shield,
  Zap,
  Sliders,
  Check,
  X,
  Database,
  Search,
  Eye,
  Info,
} from 'lucide-react';
import {
  AddressBarPosition,
  AndroidVersion,
  BrowserSettings,
  NavType,
  SearchEngine,
  ThemeMode,
} from '../types/browser';

interface SettingsModalProps {
  isOpen: boolean;
  settings: BrowserSettings;
  onClose: () => void;
  onUpdateSettings: (newSettings: Partial<BrowserSettings>) => void;
  onResetSettings: () => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  isOpen,
  settings,
  onClose,
  onUpdateSettings,
  onResetSettings,
}) => {
  if (!isOpen) return null;

  const isDark = settings.theme === 'dark' || settings.theme === 'amoled';

  const modalBg =
    settings.theme === 'amoled' && isDark
      ? 'bg-black border-neutral-800'
      : isDark
      ? 'bg-neutral-900 border-neutral-700'
      : 'bg-white border-neutral-200';

  const textColor = isDark ? 'text-neutral-100' : 'text-neutral-900';
  const cardBg = isDark ? 'bg-neutral-800/60 border-neutral-700/60' : 'bg-neutral-50 border-neutral-200';
  const activePill = 'bg-blue-600 text-white font-semibold';
  const inactivePill = isDark
    ? 'bg-neutral-800 text-neutral-300 hover:bg-neutral-700'
    : 'bg-neutral-200 text-neutral-700 hover:bg-neutral-300';

  return (
    <div className="fixed inset-0 z-50 bg-black/60 flex items-center justify-center p-3 sm:p-5 select-none">
      <div
        className={`w-full max-w-lg h-[88vh] max-h-[680px] rounded-3xl border shadow-2xl flex flex-col overflow-hidden ${modalBg} ${textColor}`}
      >
        {/* Header */}
        <div className="px-5 py-4 border-b border-neutral-800/40 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Settings size={20} className="text-purple-400" />
            <h2 className="font-bold text-base">Настройки браузера</h2>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded-full text-neutral-400 hover:text-neutral-200 transition-colors"
            title="Закрыть настройки"
          >
            <X size={20} />
          </button>
        </div>

        {/* Scrollable Settings Groups */}
        <div className="flex-1 overflow-y-auto px-5 py-4 space-y-6">
          {/* 1. Тема и внешний вид */}
          <section className="space-y-3">
            <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-neutral-400">
              <Sun size={14} className="text-amber-500" />
              <span>Тема оформления</span>
            </div>

            <div className="grid grid-cols-3 gap-2">
              <button
                onClick={() => onUpdateSettings({ theme: 'light' })}
                className={`p-3 rounded-2xl border text-center transition-all ${
                  settings.theme === 'light'
                    ? 'border-blue-500 ring-2 ring-blue-500/30 bg-blue-50/10'
                    : cardBg
                }`}
              >
                <div className="w-6 h-6 rounded-full bg-white border border-neutral-300 mx-auto mb-1.5 flex items-center justify-center shadow-xs">
                  <Sun size={13} className="text-amber-500" />
                </div>
                <span className="text-xs font-semibold block">Светлая</span>
                <span className="text-[10px] text-neutral-400 block mt-0.5">Classic Material</span>
              </button>

              <button
                onClick={() => onUpdateSettings({ theme: 'dark' })}
                className={`p-3 rounded-2xl border text-center transition-all ${
                  settings.theme === 'dark'
                    ? 'border-blue-500 ring-2 ring-blue-500/30 bg-blue-50/10'
                    : cardBg
                }`}
              >
                <div className="w-6 h-6 rounded-full bg-neutral-900 border border-neutral-700 mx-auto mb-1.5 flex items-center justify-center shadow-xs">
                  <Moon size={13} className="text-blue-400" />
                </div>
                <span className="text-xs font-semibold block">Тёмная</span>
                <span className="text-[10px] text-neutral-400 block mt-0.5">Android 10 Dark</span>
              </button>

              <button
                onClick={() => onUpdateSettings({ theme: 'amoled' })}
                className={`p-3 rounded-2xl border text-center transition-all ${
                  settings.theme === 'amoled'
                    ? 'border-blue-500 ring-2 ring-blue-500/30 bg-blue-50/10'
                    : cardBg
                }`}
              >
                <div className="w-6 h-6 rounded-full bg-black border border-neutral-800 mx-auto mb-1.5 flex items-center justify-center shadow-xs">
                  <div className="w-2.5 h-2.5 rounded-full bg-emerald-500" />
                </div>
                <span className="text-xs font-semibold block">AMOLED</span>
                <span className="text-[10px] text-neutral-400 block mt-0.5">Экономия батареи</span>
              </button>
            </div>
          </section>

          {/* 2. Адаптация под Android (Android 10 и ниже) */}
          <section className="space-y-3">
            <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-neutral-400">
              <Smartphone size={14} className="text-sky-400" />
              <span>Адаптация для Android</span>
            </div>

            <div className={`p-3.5 rounded-2xl border ${cardBg} space-y-3`}>
              {/* Android OS Version selector */}
              <div>
                <label className="text-xs font-semibold block mb-1.5">
                  Версия Android
                </label>
                <div className="grid grid-cols-3 gap-1.5">
                  {(
                    [
                      { id: 'android10', label: 'Android 10 (Q)', api: 'API 29' },
                      { id: 'android9', label: 'Android 9 (Pie)', api: 'API 28' },
                      { id: 'android8', label: 'Android 8 (Oreo)', api: 'API 27' },
                    ] as const
                  ).map((v) => (
                    <button
                      key={v.id}
                      onClick={() => onUpdateSettings({ androidVersion: v.id })}
                      className={`py-2 px-2 rounded-xl text-center transition-all ${
                        settings.androidVersion === v.id ? activePill : inactivePill
                      }`}
                    >
                      <span className="text-xs font-semibold block">{v.label}</span>
                      <span className="text-[10px] opacity-75 font-mono block">{v.api}</span>
                    </button>
                  ))}
                </div>
                <p className="text-[11px] text-neutral-400 mt-2">
                  {settings.androidVersion === 'android10'
                    ? 'Android 10: Поддержка системного темного режима, Scoped Storage и жестовой навигации.'
                    : settings.androidVersion === 'android9'
                    ? 'Android 9 Pie: Material 2 элементы, поддержка каплевидных вырезов и классическая многозадачность.'
                    : 'Android 8.1 Oreo: Классический прямоугольный экран, 3 кнопки навигации и совместимость со старыми WebView.'}
                </p>
              </div>

              {/* Navigation Bar Style */}
              <div className="pt-2 border-t border-neutral-800/40">
                <label className="text-xs font-semibold block mb-1.5">
                  Панель навигации внизу
                </label>
                <div className="grid grid-cols-3 gap-1.5">
                  <button
                    onClick={() => onUpdateSettings({ navType: 'three_button' })}
                    className={`py-2 px-2 rounded-xl text-xs font-medium ${
                      settings.navType === 'three_button' ? activePill : inactivePill
                    }`}
                  >
                    3 кнопки (◀ ● ■)
                  </button>
                  <button
                    onClick={() => onUpdateSettings({ navType: 'gesture' })}
                    className={`py-2 px-2 rounded-xl text-xs font-medium ${
                      settings.navType === 'gesture' ? activePill : inactivePill
                    }`}
                  >
                    Жест Android 10
                  </button>
                  <button
                    onClick={() => onUpdateSettings({ navType: 'hidden' })}
                    className={`py-2 px-2 rounded-xl text-xs font-medium ${
                      settings.navType === 'hidden' ? activePill : inactivePill
                    }`}
                  >
                    Полноэкранный
                  </button>
                </div>
              </div>

              {/* Address bar position */}
              <div className="pt-2 border-t border-neutral-800/40">
                <label className="text-xs font-semibold block mb-1.5">
                  Положение адресной строки (Omnibar)
                </label>
                <div className="grid grid-cols-2 gap-2">
                  <button
                    onClick={() => onUpdateSettings({ addressBarPosition: 'top' })}
                    className={`py-2 px-3 rounded-xl text-xs font-medium ${
                      settings.addressBarPosition === 'top' ? activePill : inactivePill
                    }`}
                  >
                    Сверху (Классика)
                  </button>
                  <button
                    onClick={() => onUpdateSettings({ addressBarPosition: 'bottom' })}
                    className={`py-2 px-3 rounded-xl text-xs font-medium ${
                      settings.addressBarPosition === 'bottom' ? activePill : inactivePill
                    }`}
                  >
                    Снизу (Для одной руки)
                  </button>
                </div>
              </div>
            </div>
          </section>

          {/* 3. Оптимизация памяти и данных (Легковесность) */}
          <section className="space-y-3">
            <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-neutral-400">
              <Zap size={14} className="text-amber-500" />
              <span>Память и экономия трафика</span>
            </div>

            <div className={`p-3.5 rounded-2xl border ${cardBg} space-y-3`}>
              <div className="flex items-center justify-between">
                <div>
                  <h4 className="text-xs font-semibold">Блокировать изображения</h4>
                  <p className="text-[11px] text-neutral-400">
                    Экономит до 80% оперативной памяти и мобильных мегабайт
                  </p>
                </div>
                <button
                  onClick={() => onUpdateSettings({ blockImages: !settings.blockImages })}
                  className={`w-10 h-6 rounded-full transition-colors flex items-center px-0.5 ${
                    settings.blockImages ? 'bg-amber-500 justify-end' : 'bg-neutral-600 justify-start'
                  }`}
                >
                  <div className="w-5 h-5 rounded-full bg-white shadow-xs" />
                </button>
              </div>

              <div className="flex items-center justify-between pt-2 border-t border-neutral-800/40">
                <div>
                  <h4 className="text-xs font-semibold">Блокировка рекламы и трекеров</h4>
                  <p className="text-[11px] text-neutral-400">
                    Встроенный блокировщик без внешних тяжелых расширений
                  </p>
                </div>
                <button
                  onClick={() => onUpdateSettings({ blockAds: !settings.blockAds })}
                  className={`w-10 h-6 rounded-full transition-colors flex items-center px-0.5 ${
                    settings.blockAds ? 'bg-emerald-500 justify-end' : 'bg-neutral-600 justify-start'
                  }`}
                >
                  <div className="w-5 h-5 rounded-full bg-white shadow-xs" />
                </button>
              </div>

              <div className="flex items-center justify-between pt-2 border-t border-neutral-800/40">
                <div>
                  <h4 className="text-xs font-semibold">Выполнение JavaScript</h4>
                  <p className="text-[11px] text-neutral-400">
                    Отключение ускоряет рендеринг текстовых статей
                  </p>
                </div>
                <button
                  onClick={() => onUpdateSettings({ javaScriptEnabled: !settings.javaScriptEnabled })}
                  className={`w-10 h-6 rounded-full transition-colors flex items-center px-0.5 ${
                    settings.javaScriptEnabled ? 'bg-blue-600 justify-end' : 'bg-neutral-600 justify-start'
                  }`}
                >
                  <div className="w-5 h-5 rounded-full bg-white shadow-xs" />
                </button>
              </div>

              <div className="flex items-center justify-between pt-2 border-t border-neutral-800/40">
                <div>
                  <h4 className="text-xs font-semibold">Очищать кэш при закрытии</h4>
                  <p className="text-[11px] text-neutral-400">
                    Не засоряет внутреннюю память (ROM) смартфона
                  </p>
                </div>
                <button
                  onClick={() => onUpdateSettings({ clearOnExit: !settings.clearOnExit })}
                  className={`w-10 h-6 rounded-full transition-colors flex items-center px-0.5 ${
                    settings.clearOnExit ? 'bg-blue-600 justify-end' : 'bg-neutral-600 justify-start'
                  }`}
                >
                  <div className="w-5 h-5 rounded-full bg-white shadow-xs" />
                </button>
              </div>
            </div>
          </section>

          {/* 4. Поисковик */}
          <section className="space-y-3">
            <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-neutral-400">
              <Search size={14} className="text-blue-500" />
              <span>Поисковая система</span>
            </div>

            <div className={`p-3.5 rounded-2xl border ${cardBg}`}>
              <div className="grid grid-cols-2 gap-2">
                {(
                  [
                    { id: 'duckduckgo', name: 'DuckDuckGo (Без слежки)' },
                    { id: 'google', name: 'Google' },
                    { id: 'yandex', name: 'Яндекс' },
                    { id: 'bing', name: 'Bing' },
                  ] as const
                ).map((eng) => (
                  <button
                    key={eng.id}
                    onClick={() => onUpdateSettings({ searchEngine: eng.id })}
                    className={`py-2 px-3 rounded-xl text-xs text-left font-medium transition-all ${
                      settings.searchEngine === eng.id ? activePill : inactivePill
                    }`}
                  >
                    {eng.name}
                  </button>
                ))}
              </div>
            </div>
          </section>

          {/* 5. Информация о браузере */}
          <section className="space-y-2 pt-2">
            <div className={`p-4 rounded-2xl border ${cardBg} text-xs space-y-2`}>
              <div className="flex items-center justify-between">
                <span className="font-bold">Aura Lite Browser</span>
                <span className="text-[11px] font-mono text-emerald-500 font-semibold">v2.4.1 (Stable)</span>
              </div>
              <div className="text-[11px] text-neutral-400 space-y-1">
                <div className="flex justify-between">
                  <span>Размер пакета (APK):</span>
                  <span className="font-mono text-neutral-300">2.1 МБ</span>
                </div>
                <div className="flex justify-between">
                  <span>Средний расход RAM:</span>
                  <span className="font-mono text-neutral-300">24 – 38 МБ</span>
                </div>
                <div className="flex justify-between">
                  <span>Целевая платформа:</span>
                  <span className="font-mono text-neutral-300">Android 5.0 – 10+ (API 21-29)</span>
                </div>
                <div className="flex justify-between">
                  <span>Движок:</span>
                  <span className="font-mono text-neutral-300">Android System WebView (Blink)</span>
                </div>
              </div>
            </div>
          </section>
        </div>

        {/* Footer */}
        <div className="px-5 py-3 border-t border-neutral-800/40 flex justify-between items-center text-xs">
          <button
            onClick={onResetSettings}
            className="text-neutral-400 hover:text-neutral-200"
          >
            Сбросить настройки
          </button>
          <button
            onClick={onClose}
            className="px-5 py-2 font-semibold bg-blue-600 hover:bg-blue-500 text-white rounded-xl shadow-xs"
          >
            Применить
          </button>
        </div>
      </div>
    </div>
  );
};
