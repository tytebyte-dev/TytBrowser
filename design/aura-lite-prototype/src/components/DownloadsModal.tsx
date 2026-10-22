import React from 'react';
import { Download, FileText, CheckCircle, Pause, Play, Trash2, X, HardDrive } from 'lucide-react';
import { DownloadItem, ThemeMode } from '../types/browser';

interface DownloadsModalProps {
  isOpen: boolean;
  theme: ThemeMode;
  downloads: DownloadItem[];
  onClose: () => void;
  onClearDownloads: () => void;
  onToggleDownloadStatus: (id: string) => void;
}

export const DownloadsModal: React.FC<DownloadsModalProps> = ({
  isOpen,
  theme,
  downloads,
  onClose,
  onClearDownloads,
  onToggleDownloadStatus,
}) => {
  if (!isOpen) return null;

  const isDark = theme === 'dark' || theme === 'amoled';

  const modalBg =
    theme === 'amoled' && isDark
      ? 'bg-black border-neutral-800'
      : isDark
      ? 'bg-neutral-900 border-neutral-700'
      : 'bg-white border-neutral-200';

  const textColor = isDark ? 'text-neutral-100' : 'text-neutral-900';

  return (
    <div className="fixed inset-0 z-50 bg-black/60 flex items-center justify-center p-3 sm:p-5">
      <div
        className={`w-full max-w-md h-[75vh] max-h-[550px] rounded-3xl border shadow-2xl flex flex-col overflow-hidden ${modalBg} ${textColor}`}
      >
        {/* Header */}
        <div className="px-4 py-3.5 border-b border-neutral-800/40 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Download size={18} className="text-emerald-500" />
            <span className="font-bold text-sm">Загрузки</span>
            <span className="text-xs text-neutral-400 font-mono">({downloads.length})</span>
          </div>

          <button
            onClick={onClose}
            className="p-1 rounded-full text-neutral-400 hover:text-neutral-200"
            title="Закрыть"
          >
            <X size={18} />
          </button>
        </div>

        {/* Storage path info (Scoped Storage Android 10) */}
        <div className="px-4 py-2 bg-neutral-800/30 border-b border-neutral-800/40 flex items-center justify-between text-[11px] text-neutral-400">
          <div className="flex items-center gap-1.5 font-mono">
            <HardDrive size={13} className="text-blue-400" />
            <span>/storage/emulated/0/Download</span>
          </div>
          <span className="text-emerald-500 font-medium">Scoped Storage</span>
        </div>

        {/* Downloads List */}
        <div className="flex-1 overflow-y-auto divide-y divide-neutral-800/30">
          {downloads.length === 0 ? (
            <div className="p-8 text-center text-neutral-400 text-xs">
              Загруженных файлов нет
            </div>
          ) : (
            downloads.map((item) => (
              <div key={item.id} className="p-4 flex items-center justify-between gap-3">
                <div className="w-9 h-9 rounded-xl bg-blue-500/10 text-blue-400 flex items-center justify-center shrink-0">
                  <FileText size={18} />
                </div>

                <div className="min-w-0 flex-1">
                  <h4 className="text-xs font-semibold truncate text-neutral-200">
                    {item.fileName}
                  </h4>
                  <div className="flex items-center gap-2 text-[10px] text-neutral-400 mt-0.5">
                    <span className="font-mono">{item.fileSize}</span>
                    <span>·</span>
                    <span>{item.date}</span>
                    <span>·</span>
                    <span
                      className={`font-semibold ${
                        item.status === 'completed' ? 'text-emerald-400' : 'text-amber-400'
                      }`}
                    >
                      {item.status === 'completed'
                        ? 'Готово'
                        : item.status === 'downloading'
                        ? `Загрузка ${item.progress}%`
                        : 'Пауза'}
                    </span>
                  </div>

                  {item.status !== 'completed' && (
                    <div className="w-full h-1.5 bg-neutral-800 rounded-full overflow-hidden mt-1.5">
                      <div
                        className="h-full bg-blue-500 rounded-full transition-all duration-300"
                        style={{ width: `${item.progress}%` }}
                      />
                    </div>
                  )}
                </div>

                <div className="shrink-0 flex items-center gap-1">
                  {item.status !== 'completed' ? (
                    <button
                      onClick={() => onToggleDownloadStatus(item.id)}
                      className="p-1.5 rounded-lg text-neutral-300 hover:bg-neutral-800"
                      title={item.status === 'downloading' ? 'Пауза' : 'Возобновить'}
                    >
                      {item.status === 'downloading' ? <Pause size={14} /> : <Play size={14} />}
                    </button>
                  ) : (
                    <span className="p-1 text-emerald-400" title="Загрузка завершена">
                      <CheckCircle size={16} />
                    </span>
                  )}
                </div>
              </div>
            ))
          )}
        </div>

        {/* Footer */}
        {downloads.length > 0 && (
          <div className="p-3 border-t border-neutral-800/40 flex justify-end">
            <button
              onClick={onClearDownloads}
              className="flex items-center gap-1.5 text-xs font-semibold text-neutral-400 hover:text-red-400 px-3 py-1.5 rounded-lg hover:bg-red-500/10 transition-colors"
            >
              <Trash2 size={13} />
              <span>Очистить список</span>
            </button>
          </div>
        )}
      </div>
    </div>
  );
};
