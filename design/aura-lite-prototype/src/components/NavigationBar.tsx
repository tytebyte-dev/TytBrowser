import React from 'react';
import { NavType, ThemeMode } from '../types/browser';

interface NavigationBarProps {
  navType: NavType;
  theme: ThemeMode;
  isPrivate?: boolean;
  onBack: () => void;
  onHome: () => void;
  onRecents: () => void;
}

export const NavigationBar: React.FC<NavigationBarProps> = ({
  navType,
  theme,
  isPrivate = false,
  onBack,
  onHome,
  onRecents,
}) => {
  if (navType === 'hidden') {
    return null;
  }

  const isDark = theme === 'dark' || theme === 'amoled' || isPrivate;
  const barBg =
    theme === 'amoled' && isDark
      ? 'bg-black'
      : isDark
      ? 'bg-neutral-900'
      : 'bg-neutral-100';

  const iconClass = isDark
    ? 'text-neutral-400 active:text-white active:scale-90 transition-transform'
    : 'text-neutral-600 active:text-neutral-950 active:scale-90 transition-transform';

  if (navType === 'gesture') {
    return (
      <div
        className={`w-full h-8 flex items-center justify-center select-none ${barBg} transition-colors duration-200`}
      >
        <button
          onClick={onHome}
          onDoubleClick={onRecents}
          aria-label="Жестовая полоса Android 10 (Нажмите для Домой, двойной клик для Вкладок)"
          title="Android 10: Жестовая полоса (Клик — Домой, двойной клик — Вкладки)"
          className="group py-2 px-8 focus:outline-none"
        >
          <div
            className={`w-32 h-1 rounded-full transition-all duration-150 ${
              isDark
                ? 'bg-neutral-500 group-hover:bg-neutral-300 group-active:w-28 group-active:bg-blue-400'
                : 'bg-neutral-400 group-hover:bg-neutral-600 group-active:w-28 group-active:bg-blue-500'
            }`}
          />
        </button>
      </div>
    );
  }

  // Classic 3-button navigation (Android 8, 9, and Android 10 classic)
  return (
    <nav
      aria-label="Системная навигация Android"
      className={`w-full h-11 grid grid-cols-3 items-center select-none ${barBg} border-t ${
        isDark ? 'border-neutral-800' : 'border-neutral-200'
      } transition-colors duration-200`}
    >
      {/* 1. Back Button (Triangle pointing left) */}
      <button
        onClick={onBack}
        aria-label="Назад"
        title="Назад (Системная кнопка)"
        className={`h-full flex items-center justify-center ${iconClass} focus:outline-none`}
      >
        <svg
          className="w-4 h-4 fill-current"
          viewBox="0 0 24 24"
        >
          <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z" />
        </svg>
      </button>

      {/* 2. Home Button (Circle) */}
      <button
        onClick={onHome}
        aria-label="Домой"
        title="Домой (Системная кнопка)"
        className={`h-full flex items-center justify-center ${iconClass} focus:outline-none`}
      >
        <div className="w-4 h-4 rounded-full border-2 border-current flex items-center justify-center">
          <div className="w-1.5 h-1.5 rounded-full bg-current opacity-0 group-active:opacity-100" />
        </div>
      </button>

      {/* 3. Recents / Overview Button (Square) */}
      <button
        onClick={onRecents}
        aria-label="Вкладки и приложения"
        title="Менеджер вкладок (Системная кнопка)"
        className={`h-full flex items-center justify-center ${iconClass} focus:outline-none`}
      >
        <div className="w-3.5 h-3.5 rounded-xs border-2 border-current" />
      </button>
    </nav>
  );
};
