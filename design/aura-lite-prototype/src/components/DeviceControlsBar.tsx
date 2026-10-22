import React from 'react';
import {
  Sun,
  Moon,
  Smartphone,
  Maximize2,
  Minimize2,
  RotateCcw,
  Sliders,
  Zap,
  Layout,
  Layers,
} from 'lucide-react';
import {
  AddressBarPosition,
  AndroidVersion,
  BrowserSettings,
  NavType,
  ThemeMode,
} from '../types/browser';

interface DeviceControlsBarProps {
  settings: BrowserSettings;
  deviceFrameEnabled: boolean;
  onUpdateSettings: (newSettings: Partial<BrowserSettings>) => void;
  onToggleDeviceFrame: () => void;
  onResetApp: () => void;
  onOpenSettings: () => void;
}

export const DeviceControlsBar: React.FC<DeviceControlsBarProps> = ({
  settings,
  deviceFrameEnabled,
  onUpdateSettings,
  onToggleDeviceFrame,
  onResetApp,
  onOpenSettings,
}) => {
  return (
    <header className="w-full bg-neutral-900 border-b border-neutral-800 text-neutral-200 px-4 py-2.5 z-40 shrink-0 select-none">
      <div className="max-w-7xl mx-auto flex flex-wrap items-center justify-between gap-3 text-xs">
        {/* Left: App Title & Badge */}
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2">
            <div className="w-6 h-6 rounded-lg bg-blue-600 flex items-center justify-center font-bold text-white text-[11px]">
              AL
            </div>
            <span className="font-bold tracking-tight text-white text-sm">
              Aura Lite <span className="font-normal text-neutral-400 text-xs">Android Browser</span>
            </span>
          </div>

          <div className="hidden sm:flex items-center gap-1.5 text-neutral-400 font-mono text-[11px] pl-2 border-l border-neutral-800">
            <span>Android 10 & Legacy</span>
            <span>·</span>
            <span className="text-emerald-400">Lightweight</span>
          </div>
        </div>

        {/* Center: Quick Interactive Toggles */}
        <div className="flex flex-wrap items-center gap-2">
          {/* Theme Quick Switcher */}
          <div className="flex items-center bg-neutral-800 rounded-lg p-0.5 border border-neutral-700/80">
            <button
              onClick={() => onUpdateSettings({ theme: 'light' })}
              className={`px-2.5 py-1 rounded-md font-medium transition-colors flex items-center gap-1 ${
                settings.theme === 'light'
                  ? 'bg-neutral-100 text-neutral-900 shadow-xs'
                  : 'text-neutral-400 hover:text-white'
              }`}
              title="Светлая тема (Classic Material)"
            >
              <Sun size={12} />
              <span className="hidden md:inline">Светлая</span>
            </button>

            <button
              onClick={() => onUpdateSettings({ theme: 'dark' })}
              className={`px-2.5 py-1 rounded-md font-medium transition-colors flex items-center gap-1 ${
                settings.theme === 'dark'
                  ? 'bg-neutral-700 text-white shadow-xs'
                  : 'text-neutral-400 hover:text-white'
              }`}
              title="Тёмная тема (Android 10 Dark)"
            >
              <Moon size={12} />
              <span className="hidden md:inline">Тёмная</span>
            </button>

            <button
              onClick={() => onUpdateSettings({ theme: 'amoled' })}
              className={`px-2.5 py-1 rounded-md font-medium transition-colors flex items-center gap-1 ${
                settings.theme === 'amoled'
                  ? 'bg-black text-emerald-400 border border-neutral-700 shadow-xs'
                  : 'text-neutral-400 hover:text-white'
              }`}
              title="AMOLED Черная тема (Экономия батареи)"
            >
              <span className="w-2 h-2 rounded-full bg-emerald-500" />
              <span className="hidden md:inline">AMOLED</span>
            </button>
          </div>

          {/* Android Version Selector */}
          <div className="flex items-center bg-neutral-800 rounded-lg p-0.5 border border-neutral-700/80">
            <button
              onClick={() => onUpdateSettings({ androidVersion: 'android10' })}
              className={`px-2 py-1 rounded-md text-[11px] font-semibold transition-colors ${
                settings.androidVersion === 'android10'
                  ? 'bg-blue-600 text-white'
                  : 'text-neutral-400 hover:text-white'
              }`}
              title="Android 10 (Q) - API 29"
            >
              Android 10
            </button>

            <button
              onClick={() => onUpdateSettings({ androidVersion: 'android9' })}
              className={`px-2 py-1 rounded-md text-[11px] font-semibold transition-colors ${
                settings.androidVersion === 'android9'
                  ? 'bg-blue-600 text-white'
                  : 'text-neutral-400 hover:text-white'
              }`}
              title="Android 9 (Pie) - API 28"
            >
              Android 9
            </button>

            <button
              onClick={() => onUpdateSettings({ androidVersion: 'android8' })}
              className={`px-2 py-1 rounded-md text-[11px] font-semibold transition-colors ${
                settings.androidVersion === 'android8'
                  ? 'bg-blue-600 text-white'
                  : 'text-neutral-400 hover:text-white'
              }`}
              title="Android 8.1 (Oreo) - API 27"
            >
              Android 8
            </button>
          </div>

          {/* Navigation Bar Selector */}
          <div className="hidden lg:flex items-center bg-neutral-800 rounded-lg p-0.5 border border-neutral-700/80">
            <button
              onClick={() => onUpdateSettings({ navType: 'three_button' })}
              className={`px-2 py-1 rounded-md text-[11px] font-semibold transition-colors ${
                settings.navType === 'three_button'
                  ? 'bg-neutral-700 text-white'
                  : 'text-neutral-400 hover:text-white'
              }`}
              title="3 системные кнопки (◀ ● ■)"
            >
              3 кнопки
            </button>

            <button
              onClick={() => onUpdateSettings({ navType: 'gesture' })}
              className={`px-2 py-1 rounded-md text-[11px] font-semibold transition-colors ${
                settings.navType === 'gesture'
                  ? 'bg-neutral-700 text-white'
                  : 'text-neutral-400 hover:text-white'
              }`}
              title="Жестовая полоса Android 10"
            >
              Жесты
            </button>
          </div>

          {/* Omnibar Position Toggle */}
          <div className="hidden lg:flex items-center bg-neutral-800 rounded-lg p-0.5 border border-neutral-700/80">
            <button
              onClick={() =>
                onUpdateSettings({
                  addressBarPosition:
                    settings.addressBarPosition === 'top' ? 'bottom' : 'top',
                })
              }
              className="px-2 py-1 rounded-md text-[11px] font-medium text-neutral-300 hover:text-white flex items-center gap-1"
              title="Сменить расположение адресной строки (Сверху / Снизу)"
            >
              <Layout size={12} />
              <span>
                Строка:{' '}
                <strong className="text-blue-400">
                  {settings.addressBarPosition === 'top' ? 'Вверху' : 'Внизу'}
                </strong>
              </span>
            </button>
          </div>
        </div>

        {/* Right: Frame toggle & full Settings */}
        <div className="flex items-center gap-2">
          {/* Toggle Device Frame View */}
          <button
            onClick={onToggleDeviceFrame}
            className={`px-2.5 py-1.5 rounded-lg border flex items-center gap-1.5 font-medium transition-colors ${
              deviceFrameEnabled
                ? 'bg-neutral-800 border-neutral-700 text-blue-400'
                : 'bg-neutral-800 border-neutral-700 text-neutral-300 hover:text-white'
            }`}
            title={
              deviceFrameEnabled
                ? 'Переключить в свободный вид (без корпуса)'
                : 'Показать в корпусе смартфона Android'
            }
          >
            <Smartphone size={14} />
            <span className="hidden sm:inline">
              {deviceFrameEnabled ? 'Корпус: Вкл' : 'Корпус: Выкл'}
            </span>
          </button>

          {/* Settings modal trigger */}
          <button
            onClick={onOpenSettings}
            className="p-1.5 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-300 hover:text-white border border-neutral-700 transition-colors"
            title="Все настройки браузера"
          >
            <Sliders size={15} />
          </button>

          {/* Reset App */}
          <button
            onClick={onResetApp}
            className="p-1.5 rounded-lg bg-neutral-800 hover:bg-neutral-700 text-neutral-300 hover:text-white border border-neutral-700 transition-colors"
            title="Сброс на главную страницу"
          >
            <RotateCcw size={15} />
          </button>
        </div>
      </div>
    </header>
  );
};
