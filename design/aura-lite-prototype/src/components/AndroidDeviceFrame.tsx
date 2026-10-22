import React from 'react';
import { AndroidVersion, ThemeMode } from '../types/browser';

interface AndroidDeviceFrameProps {
  children: React.ReactNode;
  enabled: boolean;
  theme: ThemeMode;
  androidVersion: AndroidVersion;
  deviceColor?: 'obsidian' | 'silver' | 'teal';
}

export const AndroidDeviceFrame: React.FC<AndroidDeviceFrameProps> = ({
  children,
  enabled,
  theme,
  androidVersion,
  deviceColor = 'obsidian',
}) => {
  if (!enabled) {
    return (
      <div className="w-full h-full flex flex-col max-w-lg mx-auto shadow-2xl border border-neutral-800 rounded-2xl overflow-hidden">
        {children}
      </div>
    );
  }

  // Hardware chassis styles
  const chassisColors = {
    obsidian: 'bg-neutral-950 border-[#222225] shadow-black/80',
    silver: 'bg-neutral-300 border-[#C0C4CC] shadow-neutral-900/40',
    teal: 'bg-emerald-950 border-[#1B4D3E] shadow-emerald-950/60',
  };

  // Bezel radius depending on Android version
  // Android 10: high curvature rounded corners (modern)
  // Android 9: medium curvature
  // Android 8: classic subtle curvature
  const cornerRadiusClass =
    androidVersion === 'android10'
      ? 'rounded-[46px]'
      : androidVersion === 'android9'
      ? 'rounded-[36px]'
      : 'rounded-[24px]';

  const screenRadiusClass =
    androidVersion === 'android10'
      ? 'rounded-[36px]'
      : androidVersion === 'android9'
      ? 'rounded-[26px]'
      : 'rounded-[14px]';

  return (
    <div className="relative flex items-center justify-center p-2 sm:p-4 my-auto">
      {/* Outer Phone Hardware Body */}
      <div
        className={`relative ${chassisColors[deviceColor]} ${cornerRadiusClass} p-3 sm:p-3.5 shadow-2xl border-[4px] ring-1 ring-white/10 select-none transition-all duration-300`}
      >
        {/* Left Side: Volume Rocker buttons */}
        <div className="absolute -left-[7px] top-28 w-[3px] h-20 bg-neutral-700 rounded-l-md" />

        {/* Right Side: Power Button */}
        <div className="absolute -right-[7px] top-32 w-[3px] h-12 bg-neutral-700 rounded-r-md" />

        {/* Top Speaker Ear-piece slit for Android 8/9/10 */}
        <div className="absolute top-1.5 left-1/2 -translate-x-1/2 w-14 h-1 bg-neutral-800 rounded-full z-40" />

        {/* Screen Display Area (Standard 19.5:9 or 18:9 aspect ratio) */}
        <div
          className={`relative w-[360px] xs:w-[380px] sm:w-[395px] h-[760px] max-h-[82vh] overflow-hidden flex flex-col bg-black ${screenRadiusClass} shadow-inner transition-colors duration-200`}
        >
          {children}
        </div>
      </div>
    </div>
  );
};
