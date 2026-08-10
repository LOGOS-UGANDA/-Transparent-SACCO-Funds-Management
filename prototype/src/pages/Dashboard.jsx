import React from 'react';
import { Link } from 'react-router-dom';
import { useApp } from '../context/AppContext';
import StatCard from '../components/StatCard';
import AllocationChart from '../components/AllocationChart';
import {
  Building2,
  CreditCard,
  Lock,
  Unlock,
  ShieldCheck,
  TrendingUp,
  ArrowRight,
  ExternalLink,
  CheckCircle,
  Clock,
  Search,
} from 'lucide-react';
import { formatCurrency, FEE_STATUS, INSTITUTION_TYPES, formatDate } from '../utils/helpers';
import {
  BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid,
} from 'recharts';

export default function Dashboard() {
  const { institutions, categories, payments, releases, stats } = useApp();

  const chartData = institutions.map((inst) => ({
    name: inst.name.split(' ')[0],
    collected: Number(inst.totalFeesCollected) / 1e18,
    released: Number(inst.totalFeesReleased) / 1e18,
  }));

  const recentPayments = [...payments].sort((a, b) => b.timestamp - a.timestamp).slice(0, 5);
  const pendingReleases = releases.filter((r) => r.status === 0);

  return (
    <div className="space-y-8">
      {/* Hero Section */}
      <div className="relative overflow-hidden rounded-3xl bg-gradient-to-br from-logos-700 via-logos-600 to-logos-800 text-white p-8 lg:p-12">
        <div className="absolute top-0 right-0 w-96 h-96 bg-white/5 rounded-full -translate-y-1/2 translate-x-1/2" />
        <div className="absolute bottom-0 left-0 w-64 h-64 bg-white/5 rounded-full translate-y-1/2 -translate-x-1/2" />
        <div className="relative z-10">
          <div className="flex items-center gap-2 mb-3">
            <ShieldCheck className="w-5 h-5 text-blue-200" />
            <span className="text-blue-200 text-sm font-medium tracking-wider uppercase">Logos Blockchain</span>
          </div>
          <h1 className="text-3xl lg:text-5xl font-bold mb-4 leading-tight">
            Every Fee Accounted For.<br />
            <span className="text-blue-200">Every Shilling Transparent.</span>
          </h1>
          <p className="text-blue-100 text-lg max-w-2xl mb-8 leading-relaxed">
            Smart contracts lock institutional fees and release them only for their stated purpose.
            Universities, SACCOs, and cooperatives — fully accountable on the Logos blockchain.
          </p>
          <div className="flex flex-wrap gap-3">
            <Link to="/verify" className="inline-flex items-center gap-2 bg-white text-logos-700 font-semibold px-6 py-3 rounded-xl hover:bg-blue-50 transition-all shadow-lg">
              <Search className="w-4 h-4" />
              Verify a Payment
            </Link>
            <Link to="/institutions" className="inline-flex items-center gap-2 bg-white/10 text-white font-semibold px-6 py-3 rounded-xl hover:bg-white/20 transition-all border border-white/20">
              View Institutions
              <ArrowRight className="w-4 h-4" />
            </Link>
          </div>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          icon={Building2}
          label="Registered Institutions"
          value={stats.totalInstitutions}
          subValue="Universities, SACCOs, Cooperatives"
          color="blue"
          trend={12}
        />
        <StatCard
          icon={Lock}
          label="Total Fees Locked"
          value={`${formatCurrency(stats.totalFeesLocked)} ETH`}
          subValue="Secured by smart contracts"
          color="indigo"
          trend={24}
        />
        <StatCard
          icon={Unlock}
          label="Fees Released"
          value={`${formatCurrency(stats.totalFeesReleased)} ETH`}
          subValue="Released for stated purposes"
          color="emerald"
        />
        <StatCard
          icon={ShieldCheck}
          label="Transparency Score"
          value={`${stats.transparencyScore}%`}
          subValue="All transactions verifiable"
          color="sky"
        />
      </div>

      {/* Charts Row */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Bar Chart */}
        <div className="glass-card rounded-2xl p-6">
          <h3 className="text-lg font-bold text-gray-900 mb-1">Fees by Institution</h3>
          <p className="text-sm text-gray-500 mb-4">Collected vs Released (ETH)</p>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={chartData} barGap={4}>
              <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
              <XAxis dataKey="name" tick={{ fontSize: 12, fill: '#64748b' }} />
              <YAxis tick={{ fontSize: 12, fill: '#64748b' }} />
              <Tooltip
                contentStyle={{
                  background: 'rgba(255,255,255,0.95)',
                  border: '1px solid #dbeafe',
                  borderRadius: '12px',
                  boxShadow: '0 4px 20px rgba(0,0,0,0.08)',
                }}
                formatter={(val) => [`${val.toFixed(0)} ETH`]}
              />
              <Bar dataKey="collected" fill="#2563eb" radius={[6, 6, 0, 0]} name="Collected" />
              <Bar dataKey="released" fill="#93c5fd" radius={[6, 6, 0, 0]} name="Released" />
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* Allocation Pie */}
        <AllocationChart
          categories={categories[1] || []}
          title="Makerere University — Fee Allocation"
        />
      </div>

      {/* Recent Activity Row */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Recent Payments */}
        <div className="glass-card rounded-2xl p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-bold text-gray-900">Recent Payments</h3>
            <Link to="/verify" className="text-logos-600 text-sm font-medium hover:text-logos-700 flex items-center gap-1">
              View all <ArrowRight className="w-3 h-3" />
            </Link>
          </div>
          <div className="space-y-3">
            {recentPayments.map((p) => {
              const inst = institutions.find((i) => i.id === p.institutionId);
              return (
                <div key={p.id} className="flex items-center justify-between p-3 rounded-xl bg-gray-50/80 hover:bg-logos-50/50 transition-colors">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-xl bg-logos-100 flex items-center justify-center text-lg">
                      {inst?.logo || '🏛️'}
                    </div>
                    <div>
                      <p className="text-sm font-semibold text-gray-900">{p.payerName}</p>
                      <p className="text-xs text-gray-500">{inst?.name} • {p.referenceNumber}</p>
                    </div>
                  </div>
                  <div className="text-right">
                    <p className="text-sm font-bold text-gray-900">{formatCurrency(p.amount)} ETH</p>
                    <span className="badge-locked text-[10px]">{FEE_STATUS[p.status]}</span>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Pending Releases */}
        <div className="glass-card rounded-2xl p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-bold text-gray-900">Pending Fund Releases</h3>
            <Link to="/releases" className="text-logos-600 text-sm font-medium hover:text-logos-700 flex items-center gap-1">
              View all <ArrowRight className="w-3 h-3" />
            </Link>
          </div>
          <div className="space-y-3">
            {pendingReleases.length === 0 ? (
              <div className="text-center py-8 text-gray-400">
                <CheckCircle className="w-10 h-10 mx-auto mb-2 opacity-50" />
                <p className="text-sm">All releases processed</p>
              </div>
            ) : (
              pendingReleases.map((r) => {
                const inst = institutions.find((i) => i.id === r.institutionId);
                return (
                  <div key={r.id} className="p-4 rounded-xl border-2 border-amber-200 bg-amber-50/50">
                    <div className="flex items-start justify-between mb-2">
                      <div>
                        <p className="text-sm font-semibold text-gray-900">{r.purpose}</p>
                        <p className="text-xs text-gray-500">{inst?.name}</p>
                      </div>
                      <span className="badge-warning">
                        <Clock className="w-3 h-3 mr-1" />
                        Pending
                      </span>
                    </div>
                    <div className="flex items-center justify-between mt-2">
                      <p className="text-lg font-bold text-amber-700">{formatCurrency(r.amount)} ETH</p>
                      <p className="text-xs text-gray-400">
                        {r.approvalCount}/{2} approvals
                      </p>
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>
      </div>

      {/* Trust Indicators */}
      <div className="glass-card rounded-2xl p-8">
        <div className="text-center mb-8">
          <h3 className="text-2xl font-bold text-gray-900">How It Works</h3>
          <p className="text-gray-500 mt-1">Built on the Logos blockchain for complete transparency</p>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          {[
            { step: '01', title: 'Pay Fees', desc: 'Students and members pay fees through the platform. Each payment is recorded on-chain.', icon: CreditCard },
            { step: '02', title: 'Funds Locked', desc: 'Smart contracts automatically lock funds and allocate them to stated fee categories.', icon: Lock },
            { step: '03', title: 'Verify Anytime', desc: 'Anyone can verify where their money sits and what it was allocated for — in real time.', icon: Search },
            { step: '04', title: 'Purpose Release', desc: 'Funds are released only after multi-signature approval for their designated purpose.', icon: Unlock },
          ].map((item) => (
            <div key={item.step} className="text-center group">
              <div className="w-14 h-14 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-logos-100 to-logos-200 flex items-center justify-center group-hover:from-logos-600 group-hover:to-logos-700 transition-all duration-300">
                <item.icon className="w-6 h-6 text-logos-600 group-hover:text-white transition-colors" />
              </div>
              <span className="text-xs font-bold text-logos-400 tracking-wider">STEP {item.step}</span>
              <h4 className="text-base font-bold text-gray-900 mt-1">{item.title}</h4>
              <p className="text-sm text-gray-500 mt-2 leading-relaxed">{item.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
