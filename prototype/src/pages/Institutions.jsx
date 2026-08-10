import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import { useApp } from '../context/AppContext';
import { formatCurrency, INSTITUTION_TYPES } from '../utils/helpers';
import {
  Building2, Search, CheckCircle, AlertCircle, ArrowRight,
  Users, Lock, Unlock, Filter,
} from 'lucide-react';

export default function Institutions() {
  const { institutions, categories } = useApp();
  const [search, setSearch] = useState('');
  const [typeFilter, setTypeFilter] = useState('all');

  const filtered = institutions.filter((inst) => {
    const matchesSearch = inst.name.toLowerCase().includes(search.toLowerCase()) ||
      inst.registrationNumber.toLowerCase().includes(search.toLowerCase());
    const matchesType = typeFilter === 'all' || INSTITUTION_TYPES[inst.institutionType] === typeFilter;
    return matchesSearch && matchesType;
  });

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900">Registered Institutions</h1>
        <p className="text-gray-500 mt-1">Browse verified institutions with transparent fee structures</p>
      </div>

      {/* Filters */}
      <div className="glass-card rounded-2xl p-4 flex flex-col sm:flex-row gap-3">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search institutions..."
            className="input-field pl-10"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        <div className="relative">
          <Filter className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
          <select
            className="input-field pl-10 pr-8 appearance-none cursor-pointer min-w-[180px]"
            value={typeFilter}
            onChange={(e) => setTypeFilter(e.target.value)}
          >
            <option value="all">All Types</option>
            {INSTITUTION_TYPES.map((t) => (
              <option key={t} value={t}>{t}</option>
            ))}
          </select>
        </div>
      </div>

      {/* Institution Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {filtered.map((inst) => {
          const cats = categories[inst.id] || [];
          const utilization = inst.totalFeesCollected > 0n
            ? Number((inst.totalFeesReleased * 10000n) / inst.totalFeesCollected) / 100
            : 0;

          return (
            <Link
              key={inst.id}
              to={`/institutions/${inst.id}`}
              className="glass-card rounded-2xl p-6 hover:shadow-xl hover:shadow-logos-900/10 hover:-translate-y-1 transition-all duration-300 group"
            >
              <div className="flex items-start justify-between mb-4">
                <div className="flex items-center gap-3">
                  <div className="w-14 h-14 rounded-2xl bg-gradient-to-br from-logos-100 to-logos-200 flex items-center justify-center text-2xl">
                    {inst.logo}
                  </div>
                  <div>
                    <h3 className="text-lg font-bold text-gray-900 group-hover:text-logos-700 transition-colors">
                      {inst.name}
                    </h3>
                    <p className="text-xs text-gray-500">{inst.location} • {inst.registrationNumber}</p>
                  </div>
                </div>
                <ArrowRight className="w-5 h-5 text-gray-300 group-hover:text-logos-600 group-hover:translate-x-1 transition-all" />
              </div>

              <div className="flex items-center gap-2 mb-4">
                {inst.isVerified ? (
                  <span className="badge-success">
                    <CheckCircle className="w-3 h-3 mr-1" />
                    Verified
                  </span>
                ) : (
                  <span className="badge-warning">
                    <AlertCircle className="w-3 h-3 mr-1" />
                    Pending Verification
                  </span>
                )}
                <span className="badge-info">{INSTITUTION_TYPES[inst.institutionType]}</span>
                <span className="badge bg-gray-100 text-gray-600">
                  <Users className="w-3 h-3 mr-1" />
                  {inst.students?.toLocaleString()} members
                </span>
              </div>

              <div className="grid grid-cols-2 gap-3 mb-4">
                <div className="bg-logos-50/50 rounded-xl p-3">
                  <div className="flex items-center gap-1 text-xs text-logos-600 font-medium mb-1">
                    <Lock className="w-3 h-3" /> Fees Locked
                  </div>
                  <p className="text-lg font-bold text-gray-900">{formatCurrency(inst.totalFeesCollected)}</p>
                  <p className="text-[10px] text-gray-400">ETH</p>
                </div>
                <div className="bg-emerald-50/50 rounded-xl p-3">
                  <div className="flex items-center gap-1 text-xs text-emerald-600 font-medium mb-1">
                    <Unlock className="w-3 h-3" /> Released
                  </div>
                  <p className="text-lg font-bold text-gray-900">{formatCurrency(inst.totalFeesReleased)}</p>
                  <p className="text-[10px] text-gray-400">ETH</p>
                </div>
              </div>

              {/* Utilization Bar */}
              <div>
                <div className="flex items-center justify-between text-xs text-gray-500 mb-1">
                  <span>Fund Utilization</span>
                  <span className="font-semibold text-gray-700">{utilization.toFixed(1)}%</span>
                </div>
                <div className="w-full h-2 bg-gray-100 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-gradient-to-r from-logos-500 to-logos-600 rounded-full transition-all duration-500"
                    style={{ width: `${Math.min(utilization, 100)}%` }}
                  />
                </div>
              </div>

              <div className="mt-3 text-xs text-gray-400">{cats.length} fee categories configured</div>
            </Link>
          );
        })}
      </div>
    </div>
  );
}
