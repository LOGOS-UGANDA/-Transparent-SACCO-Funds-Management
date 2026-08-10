import React from 'react';
import { CheckCircle, AlertCircle, Info, XCircle } from 'lucide-react';

const ICONS = {
  success: CheckCircle,
  error: XCircle,
  info: Info,
  warning: AlertCircle,
};

const STYLES = {
  success: 'bg-emerald-50 border-emerald-200 text-emerald-800',
  error: 'bg-red-50 border-red-200 text-red-800',
  info: 'bg-logos-50 border-logos-200 text-logos-800',
  warning: 'bg-amber-50 border-amber-200 text-amber-800',
};

export default function Notification({ message, type = 'info' }) {
  const Icon = ICONS[type] || Info;

  return (
    <div className="fixed bottom-6 right-6 z-50 animate-slide-up">
      <div className={`flex items-center gap-3 px-5 py-3 rounded-xl border shadow-xl ${STYLES[type]}`}>
        <Icon className="w-5 h-5 flex-shrink-0" />
        <p className="text-sm font-medium">{message}</p>
      </div>
      <style>{`
        @keyframes slide-up {
          from { transform: translateY(20px); opacity: 0; }
          to   { transform: translateY(0);    opacity: 1; }
        }
        .animate-slide-up { animation: slide-up 0.3s ease-out; }
      `}</style>
    </div>
  );
}
