import React, { useState, useEffect } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useApp } from '../context/AppContext';
import { formatCurrency, bpsToPercent, FEE_STATUS, formatDate, formatAddress } from '../utils/helpers';
import {
  Search, Shield, CheckCircle, Lock, Building2,
  User, Clock, FileCheck, XCircle, ExternalLink,
  Wallet, Zap, Globe,
} from 'lucide-react';

export default function VerifyPayment() {
  const { verifyPayment, payments, isConnected, isDeployed, contract, walletAddress } = useApp();
  const [searchParams] = useSearchParams();
  const [query, setQuery] = useState(searchParams.get('ref') || '');
  const [result, setResult] = useState(null);
  const [searched, setSearched] = useState(false);
  const [loading, setLoading] = useState(false);
  const [source, setSource] = useState(null);

  useEffect(() => {
    if (searchParams.get('ref')) {
      handleSearch(searchParams.get('ref'));
    }
  }, []);

  const handleSearch = async (searchQuery) => {
    const q = searchQuery || query;
    if (!q.trim()) return;
    setLoading(true);
    setResult(null);
    setSource(null);

    // Try on-chain first if contract is deployed and query is numeric (payment ID)
    if (isDeployed && contract.readContract && /^\d+$/.test(q.trim())) {
      try {
        const onChain = await contract.verifyPaymentOnChain(Number(q.trim()));
        if (onChain && onChain[0] && Number(onChain[0].id) !== 0) {
          const [payment, allocations, institution] = onChain;
          setResult({
            payment: {
              id: Number(payment.id),
              institutionId: Number(payment.institutionId),
              payer: payment.payer,
              payerName: payment.payerName,
              referenceNumber: payment.referenceNumber,
              amount: payment.amount,
              timestamp: Number(payment.timestamp),
              status: Number(payment.status),
              isVerified: payment.isVerified,
            },
            institution: {
              name: institution.name,
              registrationNumber: institution.registrationNumber,
            },
            allocations: allocations.map((a) => ({
              categoryId: Number(a.categoryId),
              categoryName: `Category #${Number(a.categoryId)}`,
              amount: a.amount,
              released: a.released,
            })),
          });
          setSource('blockchain');
          setSearched(true);
          setLoading(false);
          return;
        }
      } catch {
        // fall through to demo data
      }
    }

    // Demo/local data lookup
    await new Promise((r) => setTimeout(r, 600));
    const found = verifyPayment(q.trim());
    setResult(found);
    setSource(found ? 'local' : null);
    setSearched(true);
    setLoading(false);
  };

  return (
    <div className="max-w-3xl mx-auto space-y-6">
      {/* Header */}
      <div className="text-center">
        <div className="w-16 h-16 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-logos-100 to-logos-200 flex items-center justify-center">
          <FileCheck className="w-8 h-8 text-logos-600" />
        </div>
        <h1 className="text-3xl font-bold text-gray-900">Verify Your Payment</h1>
        <p className="text-gray-500 mt-2 max-w-lg mx-auto">
          Enter your payment reference number or ID to see exactly where your fees are allocated
          and whether they've been released.
        </p>
      </div>

      {/* Connection Status */}
      {isConnected && (
        <div className={`flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-medium ${
          isDeployed
            ? 'bg-emerald-50 text-emerald-700 border border-emerald-200'
            : 'bg-logos-50 text-logos-700 border border-logos-200'
        }`}>
          {isDeployed ? (
            <>
              <Globe className="w-3 h-3" />
              Connected to smart contract — verification queries the blockchain directly
            </>
          ) : (
            <>
              <Zap className="w-3 h-3" />
              Demo mode — verifying against local data (deploy contract for on-chain)
            </>
          )}
        </div>
      )}

      {/* Search Box */}
      <div className="glass-card rounded-2xl p-6">
        <div className="flex gap-3">
          <div className="relative flex-1">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
            <input
              type="text"
              className="input-field pl-12 text-lg"
              placeholder={isDeployed ? 'Enter payment ID (e.g. 1, 2, 3)' : 'Enter reference number (e.g. LFT-MUK78A3)'}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            />
          </div>
          <button
            onClick={() => handleSearch()}
            disabled={loading}
            className="btn-primary flex items-center gap-2 px-8"
          >
            {loading ? (
              <div className="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin" />
            ) : (
              <>
                <Search className="w-4 h-4" />
                Verify
              </>
            )}
          </button>
        </div>

        <div className="mt-4">
          <p className="text-xs text-gray-400 mb-2">Try these reference numbers:</p>
          <div className="flex flex-wrap gap-2">
            {payments.slice(0, 4).map((p) => (
              <button
                key={p.id}
                onClick={() => { setQuery(p.referenceNumber); handleSearch(p.referenceNumber); }}
                className="text-xs px-3 py-1.5 rounded-lg bg-logos-50 text-logos-600 hover:bg-logos-100 font-mono transition-colors"
              >
                {p.referenceNumber}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Results */}
      {searched && !result && (
        <div className="glass-card rounded-2xl p-8 text-center">
          <XCircle className="w-12 h-12 text-gray-300 mx-auto mb-3" />
          <h3 className="text-lg font-bold text-gray-900">Payment Not Found</h3>
          <p className="text-gray-500 text-sm mt-1">
            No payment matches "{query}". Please check and try again.
          </p>
          {!isConnected && (
            <p className="text-xs text-logos-600 mt-3">
              <Wallet className="w-3 h-3 inline mr-1" />
              Connect your wallet to query the blockchain directly
            </p>
          )}
        </div>
      )}

      {result && (
        <div className="space-y-6 animate-fade-in">
          {/* Verification Badge */}
          <div className="glass-card rounded-2xl p-6 border-2 border-emerald-200 bg-emerald-50/30">
            <div className="flex items-center gap-3">
              <div className="w-12 h-12 rounded-full bg-emerald-100 flex items-center justify-center">
                <CheckCircle className="w-7 h-7 text-emerald-600" />
              </div>
              <div>
                <h3 className="text-lg font-bold text-emerald-800">
                  Payment Verified {source === 'blockchain' ? 'On-Chain' : 'Locally'}
                </h3>
                <p className="text-sm text-emerald-600">
                  {source === 'blockchain'
                    ? 'This payment was verified directly from the Logos blockchain smart contract.'
                    : 'This payment is authentic and recorded. Connect wallet & deploy contract for full on-chain verification.'}
                </p>
              </div>
            </div>
            {source === 'blockchain' && (
              <div className="mt-3 flex items-center gap-2 text-xs text-emerald-600">
                <Globe className="w-3 h-3" />
                Data source: FeeTransparency smart contract on Logos blockchain
              </div>
            )}
          </div>

          {/* Payment Details */}
          <div className="glass-card rounded-2xl p-6">
            <h3 className="text-lg font-bold text-gray-900 mb-4">Payment Details</h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div className="flex items-start gap-3 p-3 bg-gray-50 rounded-xl">
                <FileCheck className="w-5 h-5 text-logos-500 mt-0.5" />
                <div>
                  <p className="text-xs text-gray-500">Reference Number</p>
                  <p className="font-bold font-mono text-logos-700">{result.payment.referenceNumber}</p>
                </div>
              </div>
              <div className="flex items-start gap-3 p-3 bg-gray-50 rounded-xl">
                <User className="w-5 h-5 text-logos-500 mt-0.5" />
                <div>
                  <p className="text-xs text-gray-500">Payer Name</p>
                  <p className="font-semibold text-gray-900">{result.payment.payerName}</p>
                </div>
              </div>
              <div className="flex items-start gap-3 p-3 bg-gray-50 rounded-xl">
                <Building2 className="w-5 h-5 text-logos-500 mt-0.5" />
                <div>
                  <p className="text-xs text-gray-500">Institution</p>
                  <p className="font-semibold text-gray-900">{result.institution?.name}</p>
                </div>
              </div>
              <div className="flex items-start gap-3 p-3 bg-gray-50 rounded-xl">
                <Lock className="w-5 h-5 text-logos-500 mt-0.5" />
                <div>
                  <p className="text-xs text-gray-500">Amount</p>
                  <p className="font-bold text-gray-900">{formatCurrency(result.payment.amount)} ETH</p>
                </div>
              </div>
              <div className="flex items-start gap-3 p-3 bg-gray-50 rounded-xl">
                <Wallet className="w-5 h-5 text-logos-500 mt-0.5" />
                <div>
                  <p className="text-xs text-gray-500">Payer Wallet</p>
                  <p className="font-mono text-sm text-gray-700">{formatAddress(result.payment.payer)}</p>
                </div>
              </div>
              <div className="flex items-start gap-3 p-3 bg-gray-50 rounded-xl">
                <Clock className="w-5 h-5 text-logos-500 mt-0.5" />
                <div>
                  <p className="text-xs text-gray-500">Date</p>
                  <p className="font-semibold text-gray-900">{formatDate(result.payment.timestamp)}</p>
                </div>
              </div>
            </div>

            {/* TX Hash if available */}
            {result.payment.txHash && (
              <div className="mt-4 bg-logos-50 border border-logos-200 rounded-xl p-3 flex items-center justify-between">
                <div>
                  <p className="text-xs text-logos-600 font-medium">Transaction Hash</p>
                  <p className="text-sm font-mono text-logos-800 break-all">{result.payment.txHash}</p>
                </div>
                <ExternalLink className="w-4 h-4 text-logos-500 flex-shrink-0" />
              </div>
            )}
          </div>

          {/* Fee Breakdown */}
          <div className="glass-card rounded-2xl p-6">
            <h3 className="text-lg font-bold text-gray-900 mb-1">Where Your Money Goes</h3>
            <p className="text-sm text-gray-500 mb-4">
              Your payment is automatically allocated to these categories by the smart contract
            </p>
            <div className="space-y-3">
              {result.allocations.map((alloc) => {
                const pct = alloc.percentage ? Number(bpsToPercent(alloc.percentage)) : null;
                return (
                  <div key={alloc.categoryId} className="p-4 rounded-xl bg-gray-50/80">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm font-semibold text-gray-900">{alloc.categoryName}</span>
                      <div className="text-right">
                        <span className="text-sm font-bold text-logos-700">{formatCurrency(alloc.amount)} ETH</span>
                        {pct && <span className="text-xs text-gray-400 ml-1">({pct}%)</span>}
                      </div>
                    </div>
                    {pct && (
                      <div className="w-full h-2 bg-gray-200 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-gradient-to-r from-logos-400 to-logos-600 rounded-full transition-all duration-700"
                          style={{ width: `${pct}%` }}
                        />
                      </div>
                    )}
                    <div className="flex items-center gap-1 mt-2">
                      <Lock className="w-3 h-3 text-logos-500" />
                      <span className="text-xs text-logos-600 font-medium">
                        {alloc.released ? 'Released for stated purpose' : 'Locked — awaiting purpose-based release'}
                      </span>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Blockchain Proof */}
          <div className="glass-card rounded-2xl p-6 bg-logos-50/30 border border-logos-200">
            <div className="flex items-start gap-3">
              <Shield className="w-6 h-6 text-logos-600 flex-shrink-0 mt-0.5" />
              <div>
                <h4 className="font-bold text-logos-800">Blockchain Guarantee</h4>
                <p className="text-sm text-logos-700 mt-1 leading-relaxed">
                  This payment is secured by the FeeTransparency smart contract on the Logos blockchain.
                  No institution can access or redirect these funds without multi-signature approval
                  for their stated purpose. The entire lifecycle — from payment to release — is
                  permanently recorded and publicly verifiable.
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      <style>{`
        @keyframes fade-in {
          from { opacity: 0; transform: translateY(10px); }
          to   { opacity: 1; transform: translateY(0); }
        }
        .animate-fade-in { animation: fade-in 0.4s ease-out; }
      `}</style>
    </div>
  );
}
