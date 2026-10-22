import React, { useState } from 'react';
import { X, Type, Sun, Moon, Coffee } from 'lucide-react';
import { BrowserSettings } from '../types/browser';

interface ReaderViewProps {
  title: string;
  url: string;
  settings: BrowserSettings;
  onClose: () => void;
}

export const ReaderView: React.FC<ReaderViewProps> = ({
  title,
  url,
  settings,
  onClose,
}) => {
  const [readerTheme, setReaderTheme] = useState<'light' | 'sepia' | 'dark' | 'black'>(
    settings.theme === 'amoled'
      ? 'black'
      : settings.theme === 'dark'
      ? 'dark'
      : 'sepia'
  );
  const [fontSize, setFontSize] = useState<number>(16);
  const [isSerif, setIsSerif] = useState<boolean>(true);

  // Theme styling definitions
  const themeStyles = {
    light: 'bg-[#FAF9F5] text-[#2D2D2D]',
    sepia: 'bg-[#F4ECD8] text-[#3D332A]',
    dark: 'bg-[#1E1E1E] text-[#D8D8D8]',
    black: 'bg-black text-[#A0A0A0]',
  };

  const toolbarBg = {
    light: 'bg-[#FAF9F5] border-b border-neutral-300',
    sepia: 'bg-[#F4ECD8] border-b border-[#E0D5BE]',
    dark: 'bg-[#1E1E1E] border-b border-neutral-800',
    black: 'bg-black border-b border-neutral-900',
  };

  return (
    <div
      className={`flex-1 w-full flex flex-col overflow-hidden ${themeStyles[readerTheme]} transition-colors duration-200 select-text`}
      style={{
        fontFamily: isSerif
          ? "'Georgia', 'Cambria', serif"
          : "'Plus Jakarta Sans', system-ui, sans-serif",
      }}
    >
      {/* Reader Mode Floating Toolbar */}
      <div
        className={`px-4 py-2.5 flex items-center justify-between select-none ${toolbarBg[readerTheme]}`}
      >
        <div className="flex items-center gap-2">
          <button
            onClick={onClose}
            className="p-1 rounded-full hover:bg-black/10 dark:hover:bg-white/10 transition-colors"
            title="Выйти из режима чтения"
          >
            <X size={18} />
          </button>
          <span className="text-xs font-semibold tracking-wider uppercase font-sans">
            Режим чтения
          </span>
        </div>

        {/* Controls: Theme & Font size */}
        <div className="flex items-center gap-2 font-sans">
          {/* Serif / Sans Toggle */}
          <button
            onClick={() => setIsSerif(!isSerif)}
            className="px-2 py-1 text-xs font-semibold rounded hover:bg-black/10 dark:hover:bg-white/10"
            title="Сменить шрифт"
          >
            {isSerif ? 'Serif' : 'Sans'}
          </button>

          {/* Font Size Buttons */}
          <div className="flex items-center gap-1 border border-current/20 rounded px-1.5 py-0.5 text-xs">
            <button
              onClick={() => setFontSize((prev) => Math.max(prev - 2, 13))}
              className="px-1 hover:font-bold"
              title="Уменьшить шрифт"
            >
              A-
            </button>
            <span className="tabular-nums text-[11px] px-1">{fontSize}px</span>
            <button
              onClick={() => setFontSize((prev) => Math.min(prev + 2, 24))}
              className="px-1 hover:font-bold"
              title="Увеличить шрифт"
            >
              A+
            </button>
          </div>

          {/* Theme Palette Buttons */}
          <div className="flex items-center gap-1">
            <button
              onClick={() => setReaderTheme('light')}
              className={`w-5 h-5 rounded-full bg-[#FAF9F5] border border-neutral-400 ${
                readerTheme === 'light' ? 'ring-2 ring-blue-500' : ''
              }`}
              title="Светлая бумага"
            />
            <button
              onClick={() => setReaderTheme('sepia')}
              className={`w-5 h-5 rounded-full bg-[#F4ECD8] border border-[#C5B494] ${
                readerTheme === 'sepia' ? 'ring-2 ring-blue-500' : ''
              }`}
              title="Сепия"
            />
            <button
              onClick={() => setReaderTheme('dark')}
              className={`w-5 h-5 rounded-full bg-[#1E1E1E] border border-neutral-700 ${
                readerTheme === 'dark' ? 'ring-2 ring-blue-500' : ''
              }`}
              title="Тёмный"
            />
            <button
              onClick={() => setReaderTheme('black')}
              className={`w-5 h-5 rounded-full bg-black border border-neutral-800 ${
                readerTheme === 'black' ? 'ring-2 ring-blue-500' : ''
              }`}
              title="AMOLED Чёрный"
            />
          </div>
        </div>
      </div>

      {/* Reader Content Body */}
      <div className="flex-1 overflow-y-auto px-5 py-6">
        <div className="max-w-xl mx-auto space-y-4">
          <h1
            className="font-bold leading-tight"
            style={{ fontSize: `${fontSize * 1.4}px` }}
          >
            {title}
          </h1>

          <div className="text-xs opacity-75 font-sans pb-3 border-b border-current/20 flex items-center justify-between">
            <span className="truncate">{url}</span>
            <span className="shrink-0 ml-2">~ 3 мин чтения</span>
          </div>

          <div
            className="space-y-4 leading-relaxed"
            style={{ fontSize: `${fontSize}px`, lineHeight: 1.7 }}
          >
            <p>
              В современных смартфонах объем фоновых данных и скриптов в мобильном вебе растет экспоненциально.
              Для Android 10 и старых моделей с ограниченным объемом памяти (от 1 до 2 ГБ RAM)
              режим чтения убирает рекламные фреймы, сторонние счетчики аналитики и скрипты отслеживания.
            </p>

            <p>
              Благодаря оптимизации парсера DOM-дерева в оперативной памяти устройства сохраняется
              исключительно текст статьи и семантическая разметка заголовков. Потребление памяти страницы
              сокращается со 120 МБ до менее чем 8 МБ.
            </p>

            <blockquote className="pl-3 border-l-2 border-current/40 italic my-3 opacity-90">
              «Оптимизация под слабые устройства — это не компромисс, а проявление уважения к ресурсам пользователя и автономности батареи.»
            </blockquote>

            <p>
              Поддержка темной темы и AMOLED-черного фона снижает нагрузку на зрение при чтении
              в ночное время и продлевает время автономной работы смартфона до 35% за счет отключения пикселей на органических дисплеях.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
