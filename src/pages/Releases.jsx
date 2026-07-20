import React, { useState } from 'react';
import { useApp } from '../context/AppContext';
import { formatCurrency, formatDate, RELEASE_STATUS } from '../utils/helpers';
import {
  ArrowRightLeft, Clock, CheckCircle, XCircle, Zap,
  Filter, Building2, FileText, Shield, Users,
} from 'lucide-react';

const STATUS_STYLES = {
  0: { badge: 'badge-warning', icon: Clock, label: 'Pending' },
  1: { badge: 'badge-info', icon: CheckCircle, label: 'Approved' },
  2: { badge: 'bg-red-100 text-red-700', icon: XCircle, label: 'Rejected' },
  3: { badge: 'badge-success', icon: Zap, label: 'Executed' },
};

export default function Releases() {
  const { releases, institutions, categories } = useApp();
  const [statusFilter, setStatusFilter] = useState('all');

  const filtered = releases.filter(
    (r) => statusFilter === 'all' || r.status === Number(statusFilter)
  );

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-gray-900">Fund Release Requests</h1>
        <p className="text-gray-500 mt-1">
          Multi-signature approval ensures funds are released only for their stated purpose
        </p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
        {[
          { label: 'Total Requests', value: releases.length, icon: FileText, color: 'bg-logos-50 text-logos-600' },
          { label: 'Pending', value: releases.filter((r) => r.status === 0).length, icon: Clock, color: 'bg-amber-50 text-amber-600' },
          { label: 'Executed', value: releases.filter((r) => r.status === 3).length, icon: Zap, color: 'bg-emerald-50 text-emerald-600' },
          { label: 'Rejected', value: releases.filter((r) => r.status === 2).length, icon: XCircle, color: 'bg-red-50 text-red-600' },
        ].map((s) => (
          <div key={s.label} className="glass-card rounded-2xl p-4 flex items-center gap-3">
            <div className={`w-10 h-10 rounded-xl ${s.color} flex items-center justify-center`}>
              <s.icon className="w-5 h-5" />
            </div>
            <div>
              <p className="text-xl font-bold text-gray-900">{s.value}</p>
              <p className="text-xs text-gray-500">{s.label}</p>
            </div>
          </div>
        ))}
      </div>

      {/* Filter */}
      <div className="glass-card rounded-2xl p-4 flex items-center gap-3">
        <Filter className="w-4 h-4 text-gray-400" />
        <select
          className="input-field max-w-xs"
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
        >
          <option value="all">All Statuses</option>
          {RELEASE_STATUS.map((s, i) => (
            <option key={i} value={i}>{s}</option>
          ))}
        </select>
      </div>

      {/* Release Cards */}
      <div className="space-y-4">
        {filtered.map((release) => {
          const inst = institutions.find((i) => i.id === release.institutionId);
          const allCats = categories[release.institutionId] || [];
          const cat = allCats.find((c) => c.id === release.categoryId);
          const style = STATUS_STYLES[release.status];
          const StatusIcon = style.icon;

          return (
            <div key={release.id} className="glass-card rounded-2xl p-6">
              <div className="flex flex-col sm:flex-row sm:items-start justify-between gap-4">
                <div className="flex-1">
                  <div className="flex items-center gap-3 mb-2">
                    <span className={`badge ${style.badge}`}>
                      <StatusIcon className="w-3 h-3 mr-1" />
                      {style.label}
                    </span>
                    <span className="text-xs text-gray-400">Release #{release.id}</span>
                  </div>
                  <h3 className="text-lg font-bold text-gray-900">{release.purpose}</h3>
                  <div className="flex flex-wrap items-center gap-3 mt-2 text-sm text-gray-500">
                    <span className="flex items-center gap-1">
                      <Building2 className="w-3.5 h-3.5" />
                      {inst?.name}
                    </span>
                    <span className="flex items-center gap-1">
                      <ArrowRightLeft className="w-3.5 h-3.5" />
                      {cat?.name || `Category #${release.categoryId}`}
                    </span>
                    <span className="flex items-center gap-1">
                      <Clock className="w-3.5 h-3.5" />
                      {formatDate(release.createdAt)}
                    </span>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-2xl font-bold text-gray-900">{formatCurrency(release.amount)} ETH</p>
                  <div className="flex items-center gap-2 mt-2 justify-end">
                    <Users className="w-3.5 h-3.5 text-gray-400" />
                    <span className="text-sm text-gray-500">
                      {release.approvalCount}/2 approvals
                    </span>
                  </div>
                </div>
              </div>

              {/* Progress bar for approvals */}
              <div className="mt-4">
                <div className="w-full h-1.5 bg-gray-100 rounded-full overflow-hidden">
                  <div
                    className={`h-full rounded-full transition-all duration-500 ${
                      release.status === 3 ? 'bg-emerald-500' :
                      release.status === 2 ? 'bg-red-400' :
                      'bg-logos-500'
                    }`}
                    style={{ width: `${(release.approvalCount / 2) * 100}%` }}
                  />
                </div>
              </div>

              {release.status === 3 && (
                <div className="mt-4 flex items-center gap-2 text-xs text-emerald-600 bg-emerald-50 rounded-lg px-3 py-2">
                  <Shield className="w-3.5 h-3.5" />
                  Released on {formatDate(release.executedAt)} — funds transferred to institution treasury
                </div>
              )}

              {release.evidenceHash && (
                <div className="mt-2 text-xs text-gray-400">
                  Evidence IPFS: {release.evidenceHash}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
