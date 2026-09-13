import React from 'react';
import { cn } from '../utils/cn';

export function Button({
  className,
  variant = 'primary',
  size = 'md',
  disabled,
  children,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
}) {
  const base =
    'inline-flex items-center justify-center font-medium rounded-lg transition-all focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed';
  const variants = {
    primary: 'bg-emerald-600 hover:bg-emerald-500 text-white shadow-sm focus:ring-emerald-500',
    secondary:
      'bg-white hover:bg-slate-100 dark:bg-slate-800 dark:hover:bg-slate-700 text-slate-700 dark:text-slate-200 border border-slate-200 dark:border-slate-700 focus:ring-slate-400 dark:focus:ring-slate-500 shadow-sm dark:shadow-none',
    danger: 'bg-rose-600 hover:bg-rose-500 text-white focus:ring-rose-500',
    ghost:
      'text-slate-600 hover:text-slate-900 hover:bg-slate-100 dark:text-slate-400 dark:hover:text-slate-100 dark:hover:bg-slate-800 focus:ring-slate-400 dark:focus:ring-slate-500',
  };
  const sizes = {
    sm: 'text-xs px-2.5 py-1.5 gap-1.5',
    md: 'text-sm px-4 py-2 gap-2',
    lg: 'text-base px-5 py-2.5 gap-2.5',
  };

  return (
    <button
      className={cn(base, variants[variant], sizes[size], className)}
      disabled={disabled}
      {...props}
    >
      {children}
    </button>
  );
}

export function Card({ className, children }: { className?: string; children: React.ReactNode }) {
  return (
    <div
      className={cn(
        'rounded-xl border border-slate-200/90 bg-white p-5 shadow-sm dark:border-slate-800/80 dark:bg-slate-900',
        className,
      )}
    >
      {children}
    </div>
  );
}

export function Input({ className, ...props }: React.InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      className={cn(
        'w-full rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-sm text-slate-900 placeholder-slate-400 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500 disabled:cursor-not-allowed disabled:bg-slate-100 dark:border-slate-800 dark:bg-slate-950 dark:text-slate-100 dark:placeholder-slate-500 dark:disabled:bg-slate-900',
        className,
      )}
      {...props}
    />
  );
}

export function Textarea({
  className,
  ...props
}: React.TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return (
    <textarea
      className={cn(
        'w-full resize-y rounded-lg border border-slate-300 bg-white px-3.5 py-2 text-sm text-slate-900 placeholder-slate-400 transition-colors focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500 disabled:cursor-not-allowed disabled:bg-slate-100 dark:border-slate-800 dark:bg-slate-950 dark:text-slate-100 dark:placeholder-slate-500 dark:disabled:bg-slate-900',
        className,
      )}
      {...props}
    />
  );
}

export function Badge({
  children,
  variant = 'default',
  className,
}: {
  children: React.ReactNode;
  variant?: 'default' | 'success' | 'warning' | 'danger' | 'info' | 'purple';
  className?: string;
}) {
  const styles = {
    default:
      'bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 border-slate-200 dark:border-slate-700',
    success:
      'bg-emerald-50 dark:bg-emerald-950/60 text-emerald-700 dark:text-emerald-400 border-emerald-200 dark:border-emerald-800/60',
    warning:
      'bg-amber-50 dark:bg-amber-950/60 text-amber-700 dark:text-amber-400 border-amber-200 dark:border-amber-800/60',
    danger:
      'bg-rose-50 dark:bg-rose-950/60 text-rose-700 dark:text-rose-400 border-rose-200 dark:border-rose-800/60',
    info: 'bg-sky-50 dark:bg-sky-950/60 text-sky-700 dark:text-sky-400 border-sky-200 dark:border-sky-800/60',
    purple:
      'bg-indigo-50 dark:bg-indigo-950/60 text-indigo-700 dark:text-indigo-400 border-indigo-200 dark:border-indigo-800/60',
  };

  return (
    <span
      className={cn(
        'inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium',
        styles[variant],
        className,
      )}
    >
      {children}
    </span>
  );
}

export function Modal({
  isOpen,
  onClose,
  title,
  children,
}: {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
}) {
  if (!isOpen) return null;

  return (
    <div className="animate-in fade-in fixed inset-0 z-50 flex items-center justify-center bg-slate-900/50 p-4 backdrop-blur-sm duration-150 dark:bg-black/70">
      <div className="w-full max-w-lg overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-2xl dark:border-slate-800 dark:bg-slate-900">
        <div className="flex items-center justify-between border-b border-slate-200 px-6 py-4 dark:border-slate-800">
          <h3 className="text-base font-semibold text-slate-900 dark:text-slate-100">{title}</h3>
          <button
            onClick={onClose}
            className="rounded-lg p-1 text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-700 dark:hover:bg-slate-800 dark:hover:text-slate-100"
          >
            ✕
          </button>
        </div>
        <div className="max-h-[80vh] overflow-y-auto p-6">{children}</div>
      </div>
    </div>
  );
}

export function Pagination({
  page,
  pageSize,
  total,
  onPageChange,
  onPageSizeChange,
  pageSizeOptions = [10, 20, 50],
}: {
  page: number;
  pageSize: number;
  total: number;
  onPageChange: (page: number) => void;
  onPageSizeChange?: (pageSize: number) => void;
  pageSizeOptions?: number[];
}) {
  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  // Compute page numbers to display
  const getPageNumbers = () => {
    const delta = 2;
    const range: number[] = [];
    for (let i = Math.max(2, page - delta); i <= Math.min(totalPages - 1, page + delta); i++) {
      range.push(i);
    }
    if (page - delta > 2) range.unshift(-1); // ellipsis
    if (page + delta < totalPages - 1) range.push(-2); // ellipsis
    range.unshift(1);
    if (totalPages > 1) range.push(totalPages);
    return range;
  };

  return (
    <div className="flex flex-wrap items-center justify-between gap-4 border-t border-slate-200 bg-slate-50/50 px-6 py-3.5 text-xs text-slate-600 dark:border-slate-800 dark:bg-slate-900/40 dark:text-slate-400">
      <div className="flex items-center gap-3">
        <span>
          Total{' '}
          <strong className="font-semibold text-slate-900 dark:text-slate-200">{total}</strong>{' '}
          records (Page {page} of {totalPages})
        </span>
        {onPageSizeChange && (
          <div className="flex items-center gap-1.5 border-l border-slate-200 pl-2 dark:border-slate-800">
            <span>Per page:</span>
            <select
              value={pageSize}
              onChange={(e) => onPageSizeChange(Number(e.target.value))}
              className="rounded-lg border border-slate-300 bg-white px-2 py-1 text-xs text-slate-800 focus:border-emerald-500 focus:outline-none dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200"
            >
              {pageSizeOptions.map((sz) => (
                <option key={sz} value={sz}>
                  {sz} / page
                </option>
              ))}
            </select>
          </div>
        )}
      </div>

      <div className="flex items-center gap-1">
        <button
          onClick={() => onPageChange(page - 1)}
          disabled={page <= 1}
          className="rounded-lg border border-slate-200 bg-white px-2.5 py-1 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-100 disabled:opacity-40 disabled:hover:bg-white dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700 dark:disabled:hover:bg-slate-800"
        >
          Previous
        </button>

        <div className="flex items-center gap-1 px-1">
          {getPageNumbers().map((p, idx) => {
            if (p < 0) {
              return (
                <span key={idx} className="px-1 text-slate-400">
                  ...
                </span>
              );
            }
            const isActive = p === page;
            return (
              <button
                key={p}
                onClick={() => onPageChange(p)}
                className={`h-7 min-w-[28px] rounded-lg text-xs font-medium transition ${
                  isActive
                    ? 'bg-emerald-600 font-semibold text-white shadow-sm'
                    : 'text-slate-700 hover:bg-slate-200/70 dark:text-slate-300 dark:hover:bg-slate-800'
                }`}
              >
                {p}
              </button>
            );
          })}
        </div>

        <button
          onClick={() => onPageChange(page + 1)}
          disabled={page >= totalPages}
          className="rounded-lg border border-slate-200 bg-white px-2.5 py-1 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-100 disabled:opacity-40 disabled:hover:bg-white dark:border-slate-700 dark:bg-slate-800 dark:text-slate-300 dark:hover:bg-slate-700 dark:disabled:hover:bg-slate-800"
        >
          Next
        </button>
      </div>
    </div>
  );
}
