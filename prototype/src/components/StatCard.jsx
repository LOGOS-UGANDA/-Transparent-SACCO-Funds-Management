import React from 'react';

export default function StatCard({ icon: Icon, label, value, subValue, color = 'blue', trend }) {
  const colorMap = {
    blue:    'from-logos-500 to-logos-700 shadow-logos-500/25',
    emerald: 'from-emerald-500 to-emerald-700 shadow-emerald-500/25',
    amber:   'from-amber-500 to-amber-700 shadow-amber-500/25',
    purple:  'from-purple-500 to-purple-700 shadow-purple-500/25',
    sky:     'from-sky-500 to-sky-700 shadow-sky-500/25',
    indigo:  'from-indigo-500 to-indigo-700 shadow-indigo-500/25',
  };

  return (
    <div className="stat-card group">
      <div className="flex items-start justify-between">
        <div
          className={`w-12 h-12 rounded-xl bg-gradient-to-br ${colorMap[color]} shadow-lg flex items-center justify-center`}
        >
          <Icon className="w-6 h-6 text-white" />
        </div>
        {trend && (
          <span className={`text-xs font-semibold px-2 py-1 rounded-full ${
            trend > 0 ? 'bg-emerald-100 text-emerald-700' : 'bg-red-100 text-red-700'
          }`}>
            {trend > 0 ? '+' : ''}{trend}%
          </span>
        )}
      </div>
      <div className="mt-4">
        <p className="text-2xl font-bold text-gray-900">{value}</p>
        <p className="text-sm text-gray-500 mt-1">{label}</p>
        {subValue && <p className="text-xs text-gray-400 mt-1">{subValue}</p>}
      </div>
    </div>
  );
}
