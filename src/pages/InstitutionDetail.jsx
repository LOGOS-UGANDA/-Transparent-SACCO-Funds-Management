import React from 'react';
import { useParams, Link } from 'react-router-dom';
import { useApp } from '../context/AppContext';
import AllocationChart from '../components/AllocationChart';
import {
  ArrowLeft, CheckCircle, Lock, Unlock, Users,
  ExternalLink, ShieldCheck, CreditCard,
} from 'lucide-react';
import { formatCurrency, bpsToPercent, INSTITUTION_TYPES, formatDate, FEE_STATUS } from '../utils/helpers';

export default function InstitutionDetail() {
  const { id } = useParams();
  const { getInstitution, getCategories, payments } = useApp();
  const institution = getInstitution(Number(id));
  const cats = getCategories(Number(id));
  const instPayments = payments.filter((p) => p.institutionId === Number(id));

  if (!institution) {
    return (
      <div className="text-center py-20">
        <p className="text-gray-500 text-lg">Institution not found</p>
        <Link to="/institutions" className="text-logos-600 font-medium mt-2 inline-block">
          ← Back to Institutions
        </Link>
      </div>
    );
  }

  const utilization = institution.totalFeesCollected > 0n
    ? Number((institution.totalFeesReleased * 10000n) / institution.totalFeesCollected) / 100
    : 0;

  return (
    <div className="space-y-6">
      {/* Back */}
      <Link to="/institutions" className="inline-flex items-center gap-2 text-logos-600 hover:text-logos-700 font-medium text-sm">
        <ArrowLeft className="w-4 h-4" /> All Institutions
      </Link>

      {/* Header Card */}
      <div className="glass-card rounded-2xl p-8">
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div className="flex items-center gap-4">
            <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-logos-100 to-logos-200 flex items-center justify-center text-3xl">
              {institution.logo}
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900">{institution.name}</h1>
              <p className="text-gray-500 text-sm">{institution.location} • {institution.registrationNumber}</p>
              <div className="flex items-center gap-2 mt-2">
                {institution.isVerified ? (
                  <span className="badge-success"><CheckCircle className="w-3 h-3 mr-1" />Verified on Logos</span>
                ) : (
                  <span className="badge-warning">Pending Verification</span>
                )}
                <span className="badge-info">{INSTITUTION_TYPES[institution.institutionType]}</span>
              </div>
            </div>
          </div>
          <Link to="/pay" className="btn-primary inline-flex items-center gap-2">
            <CreditCard className="w-4 h-4" />
            Pay Fees
          </Link>
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 sm:grid-cols-4 gap-4">
        {[
          { icon: Lock, label: 'Total Collected', value: `${formatCurrency(institution.totalFeesCollected)} ETH`, bg: 'bg-logos-50', color: 'text-logos-600' },
          { icon: Unlock, label: 'Total Released', value: `${formatCurrency(institution.totalFeesReleased)} ETH`, bg: 'bg-emerald-50', color: 'text-emerald-600' },
          { icon: ShieldCheck, label: 'Utilization', value: `${utilization.toFixed(1)}%`, bg: 'bg-sky-50', color: 'text-sky-600' },
          { icon: Users, label: 'Members', value: institution.students?.toLocaleString() || '0', bg: 'bg-purple-50', color: 'text-purple-600' },
        ].map((s) => (
          <div key={s.label} className="glass-card rounded-2xl p-5">
            <div className={`w-10 h-10 ${s.bg} rounded-xl flex items-center justify-center mb-3`}>
              <s.icon className={`w-5 h-5 ${s.color}`} />
            </div>
            <p className="text-xl font-bold text-gray-900">{s.value}</p>
            <p className="text-sm text-gray-500">{s.label}</p>
          </div>
        ))}
      </div>

      {/* Categories + Chart */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="glass-card rounded-2xl p-6">
          <h3 className="text-lg font-bold text-gray-900 mb-4">Fee Categories</h3>
          <div className="space-y-3">
            {cats.map((cat) => {
              const catUtil = cat.totalCollected > 0n
                ? Number((cat.totalReleased * 10000n) / cat.totalCollected) / 100
                : 0;
              return (
                <div key={cat.id} className="p-4 rounded-xl bg-gray-50/80 hover:bg-logos-50/40 transition-colors">
                  <div className="flex items-center justify-between mb-2">
                    <div>
                      <p className="text-sm font-semibold text-gray-900">{cat.name}</p>
                      <p className="text-xs text-gray-500">{cat.description}</p>
                    </div>
                    <span className="badge-info">{bpsToPercent(cat.allocatedPercentage)}%</span>
                  </div>
                  <div className="flex items-center justify-between text-xs text-gray-500 mb-1">
                    <span>Collected: {formatCurrency(cat.totalCollected)} ETH</span>
                    <span>{catUtil.toFixed(0)}% released</span>
                  </div>
                  <div className="w-full h-1.5 bg-gray-200 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-gradient-to-r from-logos-400 to-logos-600 rounded-full"
                      style={{ width: `${Math.min(catUtil, 100)}%` }}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        <AllocationChart categories={cats} title="Fee Allocation Breakdown" />
      </div>

      {/* Recent Payments */}
      <div className="glass-card rounded-2xl p-6">
        <h3 className="text-lg font-bold text-gray-900 mb-4">Recent Payments</h3>
        {instPayments.length === 0 ? (
          <p className="text-gray-400 text-sm text-center py-8">No payments recorded yet</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-gray-100">
                  <th className="text-left py-3 px-4 text-gray-500 font-medium">Payer</th>
                  <th className="text-left py-3 px-4 text-gray-500 font-medium">Reference</th>
                  <th className="text-left py-3 px-4 text-gray-500 font-medium">Amount</th>
                  <th className="text-left py-3 px-4 text-gray-500 font-medium">Status</th>
                  <th className="text-left py-3 px-4 text-gray-500 font-medium">Date</th>
                </tr>
              </thead>
              <tbody>
                {instPayments.map((p) => (
                  <tr key={p.id} className="border-b border-gray-50 hover:bg-logos-50/30 transition-colors">
                    <td className="py-3 px-4 font-medium text-gray-900">{p.payerName}</td>
                    <td className="py-3 px-4 font-mono text-xs text-gray-600">{p.referenceNumber}</td>
                    <td className="py-3 px-4 font-semibold">{formatCurrency(p.amount)} ETH</td>
                    <td className="py-3 px-4"><span className="badge-locked">{FEE_STATUS[p.status]}</span></td>
                    <td className="py-3 px-4 text-gray-500">{formatDate(p.timestamp)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
