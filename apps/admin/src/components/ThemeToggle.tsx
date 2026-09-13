import { useState, useRef, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Sun, Moon, Monitor, Check } from 'lucide-react';
import { useTheme, ThemeMode } from '../utils/theme';
import { cn } from '../utils/cn';

interface ThemeToggleProps {
  className?: string;
  showLabel?: boolean;
}

export function ThemeToggle({ className, showLabel = false }: ThemeToggleProps) {
  const { t } = useTranslation();
  const { theme, setTheme } = useTheme();
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }
    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen]);

  const options: { mode: ThemeMode; label: string; icon: typeof Sun }[] = [
    { mode: 'auto', label: t('theme.auto'), icon: Monitor },
    { mode: 'light', label: t('theme.light'), icon: Sun },
    { mode: 'dark', label: t('theme.dark'), icon: Moon },
  ];

  const currentOption = options.find((opt) => opt.mode === theme) || options[0];
  const CurrentIcon = currentOption.icon;

  return (
    <div className={cn('relative inline-block text-left', className)} ref={dropdownRef}>
      <button
        type="button"
        onClick={() => setIsOpen(!isOpen)}
        title={t('theme.theme')}
        className="flex items-center gap-1.5 rounded-lg border border-slate-200 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-100 hover:text-slate-900 dark:border-slate-800 dark:bg-slate-900 dark:text-slate-300 dark:shadow-none dark:hover:bg-slate-800 dark:hover:text-white"
      >
        <CurrentIcon className="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400" />
        {showLabel && <span>{currentOption.label}</span>}
      </button>

      {isOpen && (
        <div className="animate-in fade-in-50 zoom-in-95 absolute right-0 top-full z-50 mt-1.5 w-36 rounded-xl border border-slate-200 bg-white p-1 shadow-xl duration-100 dark:border-slate-800 dark:bg-slate-900">
          <div className="mb-1 border-b border-slate-100 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-wider text-slate-400 dark:border-slate-800 dark:text-slate-500">
            {t('theme.theme')}
          </div>
          {options.map(({ mode, label, icon: Icon }) => {
            const isSelected = theme === mode;
            return (
              <button
                key={mode}
                type="button"
                onClick={() => {
                  setTheme(mode);
                  setIsOpen(false);
                }}
                className={cn(
                  'flex w-full items-center justify-between rounded-lg px-2.5 py-1.5 text-left text-xs transition',
                  isSelected
                    ? 'bg-emerald-50 font-medium text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-400'
                    : 'text-slate-700 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800',
                )}
              >
                <div className="flex items-center gap-2">
                  <Icon
                    className={cn(
                      'h-3.5 w-3.5',
                      isSelected ? 'text-emerald-600 dark:text-emerald-400' : 'text-slate-400',
                    )}
                  />
                  <span>{label}</span>
                </div>
                {isSelected && (
                  <Check className="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400" />
                )}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
