import React from 'react';
import { Wifi, Battery, Volume2, ShieldCheck, Download } from 'lucide-react';
import { AndroidVersion, ThemeMode } from '../types/browser';

interface StatusBarProps {
  theme: ThemeMode;
  androidVersion: AndroidVersion;
  currentTime?: string;
  isPrivate?: boolean;
  hasActiveDownload?: boolean;
}

export const StatusBar: React.FC<StatusBarProps> = ({
  theme,
  androidVersion,
  currentTime = '10:42',
  isPrivate = false,
  hasActiveDownload = false,
}) => {
  const isDark = theme === 'dark' || theme === 'amoled' || isPrivate;
  const textColor = isDark ? 'text-neutral-200' : 'text-neutral-800';
  const iconColor = isDark ? 'text-neutral-300' : 'text-neutral-700';

  return (
    <div
      className={`w-full px-5 pt-2 pb-1.5 flex items-center justify-between text-xs font-medium select-none z-30 transition-colors duration-200 ${
        theme === 'amoled' && isDark
          ? 'bg-black'
          : isDark
          ? 'bg-neutral-900'
          : 'bg-neutral-100'
      } ${textColor}`}
    >
      {/* Left side: Time & Notification indicators */}
      <div className="flex items-center gap-2">
        <span className="font-semibold tracking-tight tabular-nums text-[13px]">
          {currentTime}
        </span>
        {isPrivate && (
          <span
            title="Инкогнито активен"
            className="flex items-center text-[10px] text-purple-400 font-mono tracking-wider ml-1"
          >
            ● INC
          </span>
        )}
        {hasActiveDownload && (
          <Download
            size={12}
            className="text-blue-500 animate-pulse ml-0.5"
            aria-label="Загрузка файла"
          />
        )}
      </div>

      {/* Center Camera Notch / Punch-hole representation */}
      <div className="flex items-center justify-center">
        {androidVersion === 'android10' ? (
          <div className="w-3.5 h-3.5 rounded-full bg-neutral-950 border border-neutral-800 shadow-inner" />
        ) : androidVersion === 'android9' ? (
          <div className="w-4 h-3 bg-neutral-950 rounded-b-full border-b border-neutral-800 shadow-inner" />
        ) : (
          /* Android 8 Oreo - Classic top speaker / sensor look */
          <div className="w-10 h-1 bg-neutral-800/60 rounded-full" />
        )}
      </div>

      {/* Right side: Signal, Wi-Fi, Battery */}
      <div className="flex items-center gap-2">
        <span className="text-[10px] font-bold text-neutral-400">4G</span>
        <Wifi size={13} className={iconColor} strokeWidth={2.4} />
        <div className="flex items-center gap-1">
          <span className="text-[11px] font-semibold tabular-nums">84%</span>
          <div className="relative flex items-center">
            <Battery size={15} className={iconColor} strokeWidth={2.2} />
            <div className="absolute left-[3px] top-[4px] bottom-[4px] w-[7px] bg-emerald-500 rounded-xs" />
          </div>
        </div>
      </div>
    </div>
  );
};
