import React, { useState } from 'react';
import { useApp } from '../context/AppContext';
import { generateRefNumber, bpsToPercent, formatCurrency, formatAddress } from '../utils/helpers';
import {
  CreditCard, Lock, Building2, User,
  Coins, ArrowRight, CheckCircle, Shield, AlertTriangle,
  Wallet, ExternalLink, Loader2, Zap,
} from 'lucide-react';

export default function PayFees() {
  const {
    institutions, getCategories, addPayment,
    isConnected, isConnecting, connectWallet, showNotification,
    walletAddress, balance, contract, isDeployed, txHash, txLoading,
  } = useApp();

  const [step, setStep] = useState(1);
  const [form, setForm] = useState({
    institutionId: '',
    payerName: '',
    amount: '',
  });
  const [receipt, setReceipt] = useState(null);
  const [processing, setProcessing] = useState(false);
  const [localTxHash, setLocalTxHash] = useState(null);

  const selectedInst = form.institutionId
    ? institutions.find((i) => i.id === Number(form.institutionId))
    : null;
  const cats = selectedInst ? getCategories(selectedInst.id) : [];

  const handleSubmit = async () => {
    if (!form.institutionId || !form.payerName || !form.amount) {
      showNotification('Please fill in all fields', 'warning');
      return;
    }
    if (!isConnected) {
      showNotification('Please connect your wallet first', 'warning');
      return;
    }

    setProcessing(true);
    const refNumber = generateRefNumber();
    const amountWei = BigInt(Math.floor(parseFloat(form.amount) * 1e18));

    if (isDeployed && contract.writeContract) {
      try {
        showNotification('Confirm the transaction in your wallet...', 'info');
        const { tx, receipt: txReceipt } = await contract.payFees(
          Number(form.institutionId),
          form.payerName,
          refNumber,
          form.amount
        );
        setLocalTxHash(tx.hash);

        const payment = {
          institutionId: Number(form.institutionId),
          payer: walletAddress,
          payerName: form.payerName,
          referenceNumber: refNumber,
          amount: amountWei,
          timestamp: Math.floor(Date.now() / 1000),
          status: 0,
          isVerified: true,
          txHash: tx.hash,
          blockNumber: txReceipt.blockNumber,
        };

        addPayment(payment);
        setReceipt({ ...payment, allocations: cats });
        showNotification('Payment locked on-chain successfully!', 'success');
      } catch (err) {
        const msg = err?.reason || err?.shortMessage || err?.message || 'Transaction failed';
        if (msg.includes('user rejected') || msg.includes('denied')) {
          showNotification('Transaction rejected in wallet', 'warning');
        } else {
          showNotification(`Transaction failed: ${msg}`, 'error');
        }
        setProcessing(false);
        return;
      }
    } else {
      await new Promise((r) => setTimeout(r, 1500));
      const payment = {
        institutionId: Number(form.institutionId),
        payer: walletAddress || '0xDEMO...ADDR',
        payerName: form.payerName,
        referenceNumber: refNumber,
        amount: amountWei,
        timestamp: Math.floor(Date.now() / 1000),
        status: 0,
        isVerified: true,
        txHash: null,
      };
      addPayment(payment);
      setReceipt({ ...payment, allocations: cats });
      showNotification('Payment recorded (demo mode — deploy contract for on-chain)', 'success');
    }

    setProcessing(false);
    setStep(3);
  };

  // ── Step 3: Receipt ──
  if (step === 3 && receipt) {
    return (
      <div className="max-w-2xl mx-auto space-y-6">
        <div className="text-center">
          <div className="w-20 h-20 mx-auto mb-4 rounded-full bg-emerald-100 flex items-center justify-center">
            <CheckCircle className="w-10 h-10 text-emerald-600" />
          </div>
          <h1 className="text-3xl font-bold text-gray-900">Payment Successful</h1>
          <p className="text-gray-500 mt-2">Your fees have been locked on the Logos blockchain</p>
        </div>

        <div className="glass-card rounded-2xl p-6 space-y-4">
          <div className="text-center py-4 border-b border-gray-100">
            <p className="text-sm text-gray-500">Reference Number</p>
            <p className="text-2xl font-bold font-mono text-logos-700 mt-1">{receipt.referenceNumber}</p>
            <p className="text-xs text-gray-400 mt-1">Save this number to verify your payment anytime</p>
          </div>

          {/* Transaction hash */}
          {(localTxHash || receipt.txHash) && (
            <div className="bg-emerald-50 border border-emerald-200 rounded-xl p-3 flex items-center justify-between">
              <div>
                <p className="text-xs text-emerald-600 font-medium">Transaction Hash</p>
                <p className="text-sm font-mono text-emerald-800 break-all">
                  {localTxHash || receipt.txHash}
                </p>
              </div>
              <ExternalLink className="w-4 h-4 text-emerald-500 flex-shrink-0" />
            </div>
          )}

          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-xs text-gray-500">Payer</p>
              <p className="font-semibold text-gray-900">{receipt.payerName}</p>
            </div>
            <div>
              <p className="text-xs text-gray-500">Institution</p>
              <p className="font-semibold text-gray-900">{selectedInst?.name}</p>
            </div>
            <div>
              <p className="text-xs text-gray-500">Amount</p>
              <p className="font-semibold text-gray-900">{formatCurrency(receipt.amount)} ETH</p>
            </div>
            <div>
              <p className="text-xs text-gray-500">From Wallet</p>
              <p className="font-mono text-sm text-gray-700">{formatAddress(receipt.payer)}</p>
            </div>
          </div>

          <div className="pt-4 border-t border-gray-100">
            <p className="text-sm font-semibold text-gray-900 mb-3">Fee Allocation Breakdown</p>
            {receipt.allocations.map((cat) => {
              const alloc = (BigInt(receipt.amount) * BigInt(cat.allocatedPercentage)) / 10000n;
              return (
                <div key={cat.id} className="flex items-center justify-between py-2 border-b border-gray-50 last:border-0">
                  <div className="flex items-center gap-2">
                    <div className="w-2 h-2 rounded-full bg-logos-500" />
                    <span className="text-sm text-gray-700">{cat.name}</span>
                  </div>
                  <div className="text-right">
                    <span className="text-sm font-semibold">{formatCurrency(alloc)} ETH</span>
                    <span className="text-xs text-gray-400 ml-2">({bpsToPercent(cat.allocatedPercentage)}%)</span>
                  </div>
                </div>
              );
            })}
          </div>

          <div className="bg-logos-50 rounded-xl p-4 flex items-start gap-3">
            <Shield className="w-5 h-5 text-logos-600 flex-shrink-0 mt-0.5" />
            <div>
              <p className="text-sm font-semibold text-logos-800">Secured by Smart Contract</p>
              <p className="text-xs text-logos-600 mt-1">
                These funds are locked on the Logos blockchain and can only be released
                for their stated purpose with multi-signature approval.
              </p>
            </div>
          </div>
        </div>

        <div className="flex gap-3">
          <button
            onClick={() => {
              setStep(1);
              setForm({ institutionId: '', payerName: '', amount: '' });
              setReceipt(null);
              setLocalTxHash(null);
            }}
            className="btn-secondary flex-1"
          >
            Make Another Payment
          </button>
          <a href={`/verify?ref=${receipt.referenceNumber}`} className="btn-primary flex-1 text-center">
            Verify Payment
          </a>
        </div>
      </div>
    );
  }

  // ── Steps 1 & 2 ──
  return (
    <div className="max-w-3xl mx-auto space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-gray-900">Pay Institutional Fees</h1>
        <p className="text-gray-500 mt-1">Fees are locked by smart contracts and allocated transparently</p>
      </div>

      {/* Progress Steps */}
      <div className="flex items-center gap-3">
        {[
          { num: 1, label: 'Details' },
          { num: 2, label: 'Confirm' },
          { num: 3, label: 'Receipt' },
        ].map((s, i) => (
          <React.Fragment key={s.num}>
            <div className={`flex items-center gap-2 ${step >= s.num ? 'text-logos-700' : 'text-gray-400'}`}>
              <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold ${
                step >= s.num ? 'bg-logos-600 text-white' : 'bg-gray-200 text-gray-500'
              }`}>
                {step > s.num ? <CheckCircle className="w-4 h-4" /> : s.num}
              </div>
              <span className="text-sm font-medium hidden sm:block">{s.label}</span>
            </div>
            {i < 2 && <div className={`flex-1 h-0.5 ${step > s.num ? 'bg-logos-600' : 'bg-gray-200'}`} />}
          </React.Fragment>
        ))}
      </div>

      {/* Wallet Status */}
      {!isConnected ? (
        <div className="bg-amber-50 border-2 border-amber-200 rounded-xl p-4 flex items-start gap-3">
          <AlertTriangle className="w-5 h-5 text-amber-600 flex-shrink-0 mt-0.5" />
          <div className="flex-1">
            <p className="text-sm font-semibold text-amber-800">Wallet Not Connected</p>
            <p className="text-xs text-amber-600 mt-1">Connect your wallet to send transactions on the Logos blockchain</p>
            <button
              onClick={connectWallet}
              disabled={isConnecting}
              className="mt-3 inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-amber-600 text-white text-sm font-semibold hover:bg-amber-700 transition-colors disabled:opacity-70"
            >
              {isConnecting ? <Loader2 className="w-4 h-4 animate-spin" /> : <Wallet className="w-4 h-4" />}
              {isConnecting ? 'Connecting...' : 'Connect Wallet'}
            </button>
          </div>
        </div>
      ) : (
        <div className="bg-emerald-50 border border-emerald-200 rounded-xl p-3 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-emerald-100 flex items-center justify-center">
              <Wallet className="w-4 h-4 text-emerald-600" />
            </div>
            <div>
              <p className="text-sm font-semibold text-emerald-800">Wallet Connected</p>
              <p className="text-xs font-mono text-emerald-600">{walletAddress}</p>
            </div>
          </div>
          <div className="text-right">
            <p className="text-sm font-bold text-emerald-800">{balance ? `${parseFloat(balance).toFixed(4)} ETH` : '—'}</p>
            <p className="text-xs text-emerald-600">Available Balance</p>
          </div>
        </div>
      )}

      {/* Mode indicator */}
      {isConnected && (
        <div className={`flex items-center gap-2 px-3 py-2 rounded-lg text-xs font-medium ${
          isDeployed
            ? 'bg-emerald-50 text-emerald-700 border border-emerald-200'
            : 'bg-logos-50 text-logos-700 border border-logos-200'
        }`}>
          <Zap className="w-3 h-3" />
          {isDeployed
            ? 'Live Mode — transactions will be sent to the smart contract on-chain'
            : 'Demo Mode — contract not deployed. Payments are simulated locally.'}
        </div>
      )}

      {step === 1 && (
        <div className="glass-card rounded-2xl p-6 space-y-5">
          <div>
            <label className="block text-sm font-semibold text-gray-700 mb-2">
              <Building2 className="w-4 h-4 inline mr-1" />
              Select Institution
            </label>
            <select
              className="input-field"
              value={form.institutionId}
              onChange={(e) => setForm({ ...form, institutionId: e.target.value })}
            >
              <option value="">Choose an institution...</option>
              {institutions.filter((i) => i.isVerified).map((inst) => (
                <option key={inst.id} value={inst.id}>
                  {inst.logo} {inst.name} ({inst.registrationNumber})
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-semibold text-gray-700 mb-2">
              <User className="w-4 h-4 inline mr-1" />
              Full Name
            </label>
            <input
              type="text"
              className="input-field"
              placeholder="Enter your full name"
              value={form.payerName}
              onChange={(e) => setForm({ ...form, payerName: e.target.value })}
            />
          </div>

          <div>
            <label className="block text-sm font-semibold text-gray-700 mb-2">
              <Coins className="w-4 h-4 inline mr-1" />
              Amount (ETH)
            </label>
            <input
              type="number"
              className="input-field"
              placeholder="0.00"
              step="0.01"
              min="0"
              value={form.amount}
              onChange={(e) => setForm({ ...form, amount: e.target.value })}
            />
            {isConnected && balance && (
              <p className="text-xs text-gray-400 mt-1">
                Wallet balance: {parseFloat(balance).toFixed(4)} ETH
              </p>
            )}
          </div>

          {selectedInst && cats.length > 0 && (
            <div className="bg-logos-50/50 rounded-xl p-4">
              <p className="text-sm font-semibold text-logos-800 mb-3">
                <Lock className="w-4 h-4 inline mr-1" />
                Your fees will be automatically allocated to:
              </p>
              {cats.map((cat) => (
                <div key={cat.id} className="flex items-center justify-between py-1.5 text-sm">
                  <span className="text-gray-700">{cat.name}</span>
                  <span className="font-semibold text-logos-700">{bpsToPercent(cat.allocatedPercentage)}%</span>
                </div>
              ))}
            </div>
          )}

          <button
            className="btn-primary w-full flex items-center justify-center gap-2"
            onClick={() => {
              if (!form.institutionId || !form.payerName || !form.amount) {
                showNotification('Please fill in all fields', 'warning');
                return;
              }
              setStep(2);
            }}
          >
            Continue to Confirmation
            <ArrowRight className="w-4 h-4" />
          </button>
        </div>
      )}

      {step === 2 && (
        <div className="glass-card rounded-2xl p-6 space-y-5">
          <h3 className="text-lg font-bold text-gray-900">Confirm Your Payment</h3>

          <div className="bg-gray-50 rounded-xl p-4 space-y-3">
            <div className="flex justify-between">
              <span className="text-sm text-gray-500">Institution</span>
              <span className="text-sm font-semibold">{selectedInst?.name}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-500">Payer Name</span>
              <span className="text-sm font-semibold">{form.payerName}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-sm text-gray-500">Amount</span>
              <span className="text-sm font-semibold">{form.amount} ETH</span>
            </div>
            {walletAddress && (
              <div className="flex justify-between">
                <span className="text-sm text-gray-500">From Wallet</span>
                <span className="text-sm font-mono text-gray-600">{formatAddress(walletAddress)}</span>
              </div>
            )}
            <div className="flex justify-between">
              <span className="text-sm text-gray-500">Mode</span>
              <span className={`text-sm font-semibold ${isDeployed ? 'text-emerald-600' : 'text-amber-600'}`}>
                {isDeployed ? 'On-Chain (Live)' : 'Demo (Simulated)'}
              </span>
            </div>
          </div>

          {isDeployed && (
            <div className="bg-logos-50 rounded-xl p-4">
              <p className="text-sm text-logos-700">
                <Shield className="w-4 h-4 inline mr-1" />
                This will send a real transaction from your wallet. You will be prompted to
                confirm in MetaMask. Funds will be locked in the FeeTransparency smart contract.
              </p>
            </div>
          )}

          {!isDeployed && (
            <div className="bg-amber-50 rounded-xl p-4">
              <p className="text-sm text-amber-700">
                <AlertTriangle className="w-4 h-4 inline mr-1" />
                Running in demo mode. This payment will be simulated locally.
                Deploy the smart contract to enable on-chain locking.
              </p>
            </div>
          )}

          <div className="flex gap-3">
            <button onClick={() => setStep(1)} className="btn-secondary flex-1">
              Go Back
            </button>
            <button
              onClick={handleSubmit}
              disabled={processing}
              className="btn-primary flex-1 flex items-center justify-center gap-2 disabled:opacity-70"
            >
              {processing ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  {isDeployed ? 'Confirming on-chain...' : 'Processing...'}
                </>
              ) : (
                <>
                  <Lock className="w-4 h-4" />
                  {isDeployed ? 'Send Transaction' : 'Confirm & Lock Fees'}
                </>
              )}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
