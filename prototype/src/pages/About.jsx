import React from 'react';
import {
  Shield, Lock, Search, Unlock, Users, Building2,
  Code, Globe, CheckCircle, ArrowRight, Zap, Eye,
} from 'lucide-react';

export default function About() {
  return (
    <div className="space-y-12">
      {/* Hero */}
      <div className="text-center max-w-3xl mx-auto">
        <div className="w-16 h-16 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-logos-600 to-logos-800 flex items-center justify-center shadow-lg shadow-logos-600/30">
          <Shield className="w-8 h-8 text-white" />
        </div>
        <h1 className="text-4xl font-bold text-gray-900 mb-4">
          Ending Fee Opacity in<br />
          <span className="text-gradient">African Institutions</span>
        </h1>
        <p className="text-lg text-gray-500 leading-relaxed">
          Fees paid to schools, universities, SACCOs, and cooperatives disappear into systems
          with no accountability. Payers have no way to verify where the money goes.
          <strong className="text-gray-700"> We're changing that with blockchain transparency.</strong>
        </p>
      </div>

      {/* The Problem */}
      <div className="glass-card rounded-2xl p-8">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 items-center">
          <div>
            <span className="text-xs font-bold text-red-500 tracking-wider uppercase">The Problem</span>
            <h2 className="text-2xl font-bold text-gray-900 mt-2 mb-4">
              Fees Disappear Into Black Boxes
            </h2>
            <div className="space-y-4 text-gray-600 leading-relaxed">
              <p>
                When students at Makerere University or Victoria University pay functional fees,
                those funds enter institutional accounts with no public ledger. There's no way
                to know if the "library fee" was actually spent on books, or if the "ICT fee"
                funded computer labs.
              </p>
              <p>
                The same problem exists in SACCOs and cooperatives — members contribute fees
                for specific purposes, but cannot verify that funds were used as promised.
              </p>
              <p>
                We used <strong>Makerere University</strong> and <strong>Victoria University</strong> as
                real research anchors to test whether a transparency solution could work at that scale.
                <strong className="text-logos-700"> The answer was clear: it can, and it's needed.</strong>
              </p>
            </div>
          </div>
          <div className="grid grid-cols-2 gap-4">
            {[
              { stat: '45K+', label: 'Students at Makerere affected', icon: Users },
              { stat: '12K+', label: 'Victoria University students', icon: Building2 },
              { stat: '0%', label: 'Fee transparency before Logos', icon: Eye },
              { stat: '100%', label: 'Target on-chain visibility', icon: CheckCircle },
            ].map((item) => (
              <div key={item.label} className="bg-logos-50/50 rounded-xl p-4 text-center">
                <item.icon className="w-6 h-6 text-logos-500 mx-auto mb-2" />
                <p className="text-2xl font-bold text-gray-900">{item.stat}</p>
                <p className="text-xs text-gray-500 mt-1">{item.label}</p>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* The Solution */}
      <div>
        <div className="text-center mb-8">
          <span className="text-xs font-bold text-logos-600 tracking-wider uppercase">The Solution</span>
          <h2 className="text-3xl font-bold text-gray-900 mt-2">
            Smart Contracts for Fee Accountability
          </h2>
          <p className="text-gray-500 mt-2 max-w-xl mx-auto">
            Built on the Logos blockchain, our smart contracts lock every fee payment and
            guarantee funds are only released for their stated purpose.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {[
            {
              icon: Lock,
              title: 'Immutable Fee Locking',
              desc: 'When fees are paid, smart contracts automatically allocate funds to predefined categories (tuition, library, ICT, sports). No one can move these funds arbitrarily.',
            },
            {
              icon: Search,
              title: 'Real-Time Verification',
              desc: 'Any payer — student, SACCO member, cooperative participant — can verify exactly where their money sits, what it was allocated for, and whether it has been released.',
            },
            {
              icon: Users,
              title: 'Multi-Sig Release',
              desc: 'Funds are released only after multiple authorized signers approve the release for its stated purpose. Every release includes evidence and a clear purpose statement.',
            },
          ].map((item) => (
            <div key={item.title} className="glass-card rounded-2xl p-6 hover:shadow-xl transition-shadow">
              <div className="w-12 h-12 rounded-2xl bg-gradient-to-br from-logos-100 to-logos-200 flex items-center justify-center mb-4">
                <item.icon className="w-6 h-6 text-logos-600" />
              </div>
              <h3 className="text-lg font-bold text-gray-900 mb-2">{item.title}</h3>
              <p className="text-sm text-gray-500 leading-relaxed">{item.desc}</p>
            </div>
          ))}
        </div>
      </div>

      {/* How Smart Contract Works */}
      <div className="glass-card rounded-2xl p-8 bg-gradient-to-br from-logos-50 to-white">
        <h2 className="text-2xl font-bold text-gray-900 mb-6 text-center">Smart Contract Architecture</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <div className="space-y-4">
            <h3 className="font-bold text-logos-800 flex items-center gap-2">
              <Code className="w-5 h-5" /> Core Functions
            </h3>
            {[
              { fn: 'registerInstitution()', desc: 'Register a school, university, SACCO, or cooperative with verified treasury address' },
              { fn: 'createFeeCategory()', desc: 'Define fee categories with exact percentage allocations (tuition 40%, library 15%, etc.)' },
              { fn: 'payFees()', desc: 'Pay fees — smart contract auto-allocates to each category and locks funds' },
              { fn: 'requestRelease()', desc: 'Request fund release with purpose statement and evidence (IPFS hash)' },
              { fn: 'approveRelease()', desc: 'Multi-signature approval — requires 2+ authorized signers' },
              { fn: 'verifyPaymentByRef()', desc: 'Anyone can verify a payment and see exact allocations' },
            ].map((item) => (
              <div key={item.fn} className="flex items-start gap-3 p-3 bg-white rounded-xl">
                <Zap className="w-4 h-4 text-logos-500 mt-1 flex-shrink-0" />
                <div>
                  <code className="text-sm font-semibold text-logos-700">{item.fn}</code>
                  <p className="text-xs text-gray-500 mt-0.5">{item.desc}</p>
                </div>
              </div>
            ))}
          </div>
          <div className="space-y-4">
            <h3 className="font-bold text-logos-800 flex items-center gap-2">
              <Shield className="w-5 h-5" /> Security Features
            </h3>
            {[
              'Role-based access control — only institution admins can manage categories',
              'Multi-signature approval prevents single-point corruption',
              'On-chain audit trail — every transaction is permanently recorded',
              'Category allocation validation — percentages must total ≤ 100%',
              'Evidence-based releases — IPFS hashes link to procurement documents',
              'Verified institutions only — platform owner validates registration',
            ].map((feature) => (
              <div key={feature} className="flex items-start gap-3 p-3 bg-white rounded-xl">
                <CheckCircle className="w-4 h-4 text-emerald-500 mt-0.5 flex-shrink-0" />
                <p className="text-sm text-gray-700">{feature}</p>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Research Anchors */}
      <div>
        <h2 className="text-2xl font-bold text-gray-900 mb-6 text-center">Research Anchors</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {[
            {
              name: 'Makerere University',
              logo: '🏛️',
              location: 'Kampala, Uganda',
              students: '45,000+',
              finding: 'Students pay 7+ functional fee categories per semester with zero visibility into how funds are used. Our system maps each fee to a purpose-locked smart contract category.',
            },
            {
              name: 'Victoria University',
              logo: '🎓',
              location: 'Kampala, Uganda',
              students: '12,000+',
              finding: 'Technology fees and library fees are collected but students cannot verify if labs were upgraded or books purchased. On-chain verification solves this completely.',
            },
          ].map((inst) => (
            <div key={inst.name} className="glass-card rounded-2xl p-6 border-2 border-logos-100">
              <div className="flex items-center gap-3 mb-4">
                <div className="w-14 h-14 rounded-2xl bg-logos-100 flex items-center justify-center text-2xl">
                  {inst.logo}
                </div>
                <div>
                  <h3 className="text-lg font-bold text-gray-900">{inst.name}</h3>
                  <p className="text-sm text-gray-500">{inst.location} • {inst.students} students</p>
                </div>
              </div>
              <p className="text-sm text-gray-600 leading-relaxed">{inst.finding}</p>
            </div>
          ))}
        </div>
      </div>

      {/* CTA */}
      <div className="rounded-3xl bg-gradient-to-r from-logos-700 to-logos-900 text-white p-8 lg:p-12 text-center">
        <h2 className="text-3xl font-bold mb-4">Ready to Make Fees Transparent?</h2>
        <p className="text-blue-200 max-w-xl mx-auto mb-8">
          Whether you're a university administrator, SACCO manager, or cooperative leader —
          join the movement for fee accountability on the Logos blockchain.
        </p>
        <div className="flex flex-wrap gap-3 justify-center">
          <a href="/pay" className="inline-flex items-center gap-2 bg-white text-logos-700 font-semibold px-8 py-3 rounded-xl hover:bg-blue-50 transition-all shadow-lg">
            Pay Fees Now
          </a>
          <a href="/verify" className="inline-flex items-center gap-2 bg-white/10 text-white font-semibold px-8 py-3 rounded-xl border border-white/20 hover:bg-white/20 transition-all">
            Verify a Payment
            <ArrowRight className="w-4 h-4" />
          </a>
        </div>
      </div>
    </div>
  );
}
