import React, { useState } from 'react';
import { Outlet, Link, useLocation } from 'react-router-dom';
import { useApp } from '../context/AppContext';
import {
  LayoutDashboard,
  Building2,
  CreditCard,
  Search,
  ArrowRightLeft,
  Info,
  Wallet,
  Menu,
  X,
  Shield,
  ChevronRight,
  Loader2,
  Globe,
  CircleDollarSign,
  LogOut,
  ExternalLink,
} from 'lucide-react';
import Notification from './Notification';

const NAV_ITEMS = [
  { path: '/', label: 'Dashboard', icon: LayoutDashboard },
  { path: '/institutions', label: 'Institutions', icon: Building2 },
  { path: '/pay', label: 'Pay Fees', icon: CreditCard },
  { path: '/verify', label: 'Verify Payment', icon: Search },
  { path: '/releases', label: 'Fund Releases', icon: ArrowRightLeft },
  { path: '/about', label: 'About', icon: Info },
];

function WalletDropdown({ address, balance, networkName, chainId, onDisconnect, onSwitchNetwork }) {
  const [open, setOpen] = useState(false);

  return (
    <div className="relative">
      <button
        onClick={() => setOpen(!open)}
        className="hidden sm:flex items-center gap-2 px-4 py-2 rounded-xl bg-logos-50 border border-logos-200 text-logos-700 text-sm font-medium hover:bg-logos-100 transition-all"
      >
        <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
        <span className="font-mono">{address?.slice(0, 6)}...{address?.slice(-4)}</span>
      </button>

      {open && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />
          <div className="absolute right-0 top-full mt-2 w-72 z-50 bg-white rounded-2xl border border-gray-200 shadow-2xl shadow-logos-900/10 overflow-hidden">
            <div className="p-4 bg-gradient-to-r from-logos-600 to-logos-700 text-white">
              <p className="text-xs text-blue-200 mb-1">Connected Wallet</p>
              <p className="font-mono text-sm font-semibold break-all">{address}</p>
            </div>

            <div className="p-4 space-y-3">
              <div className="flex items-center justify-between">
                <span className="flex items-center gap-2 text-sm text-gray-500">
                  <CircleDollarSign className="w-4 h-4" />
                  Balance
                </span>
                <span className="text-sm font-bold text-gray-900">
                  {balance ? `${parseFloat(balance).toFixed(4)} ETH` : '—'}
                </span>
              </div>

              <div className="flex items-center justify-between">
                <span className="flex items-center gap-2 text-sm text-gray-500">
                  <Globe className="w-4 h-4" />
                  Network
                </span>
                <span className="text-sm font-semibold text-gray-900 flex items-center gap-1">
                  <div className="w-2 h-2 rounded-full bg-emerald-500" />
                  {networkName || 'Unknown'}
                </span>
              </div>

              <div className="flex items-center justify-between">
                <span className="flex items-center gap-2 text-sm text-gray-500">
                  <Shield className="w-4 h-4" />
                  Chain ID
                </span>
                <span className="text-xs font-mono text-gray-600 bg-gray-100 px-2 py-0.5 rounded">
                  {chainId || '—'}
                </span>
              </div>
            </div>

            <div className="border-t border-gray-100 p-3 space-y-1">
              <button
                onClick={() => { onSwitchNetwork(); setOpen(false); }}
                className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-logos-700 hover:bg-logos-50 transition-colors"
              >
                <Globe className="w-4 h-4" />
                Switch to Logos Network
              </button>
              <button
                onClick={() => { onDisconnect(); setOpen(false); }}
                className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm text-red-600 hover:bg-red-50 transition-colors"
              >
                <LogOut className="w-4 h-4" />
                Disconnect Wallet
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  );
}

export default function Layout() {
  const {
    walletAddress, isConnected, isConnecting,
    balance, networkName, chainId,
    connectWallet, disconnectWallet, switchToLogos,
    notification, isDeployed,
  } = useApp();
  const location = useLocation();
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <div className="min-h-screen">
      {/* Contract Status Banner */}
      {isConnected && !isDeployed && (
        <div className="bg-amber-50 border-b border-amber-200 px-4 py-2 text-center text-sm text-amber-700">
          <Shield className="w-4 h-4 inline mr-1" />
          Smart contract not deployed — running with demo data.
          Deploy the contract and set <code className="bg-amber-100 px-1 rounded text-xs">VITE_CONTRACT_ADDRESS</code> to enable live on-chain features.
        </div>
      )}

      {/* Top Navigation */}
      <nav className="sticky top-0 z-50 glass-card border-b border-blue-100/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            {/* Logo */}
            <Link to="/" className="flex items-center gap-3 group">
              <div className="w-9 h-9 bg-gradient-to-br from-logos-600 to-logos-800 rounded-xl flex items-center justify-center shadow-lg shadow-logos-600/30 group-hover:shadow-logos-600/50 transition-shadow">
                <Shield className="w-5 h-5 text-white" />
              </div>
              <div>
                <span className="text-lg font-bold text-gradient">LOGOS</span>
                <span className="text-[10px] block -mt-1 text-logos-500 font-medium tracking-wider">FEE TRANSPARENCY</span>
              </div>
            </Link>

            {/* Desktop Nav */}
            <div className="hidden lg:flex items-center gap-1">
              {NAV_ITEMS.map((item) => {
                const Icon = item.icon;
                const isActive = location.pathname === item.path;
                return (
                  <Link
                    key={item.path}
                    to={item.path}
                    className={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
                      isActive
                        ? 'bg-logos-100 text-logos-700 shadow-sm'
                        : 'text-gray-600 hover:text-logos-700 hover:bg-logos-50'
                    }`}
                  >
                    <Icon className="w-4 h-4" />
                    {item.label}
                  </Link>
                );
              })}
            </div>

            {/* Wallet + Mobile Toggle */}
            <div className="flex items-center gap-3">
              {isConnected ? (
                <WalletDropdown
                  address={walletAddress}
                  balance={balance}
                  networkName={networkName}
                  chainId={chainId}
                  onDisconnect={disconnectWallet}
                  onSwitchNetwork={switchToLogos}
                />
              ) : (
                <button
                  onClick={connectWallet}
                  disabled={isConnecting}
                  className="hidden sm:flex btn-primary text-sm py-2 items-center gap-2 disabled:opacity-70"
                >
                  {isConnecting ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : (
                    <Wallet className="w-4 h-4" />
                  )}
                  {isConnecting ? 'Connecting...' : 'Connect Wallet'}
                </button>
              )}
              <button
                onClick={() => setMobileOpen(!mobileOpen)}
                className="lg:hidden p-2 rounded-lg hover:bg-logos-50 text-gray-600"
              >
                {mobileOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
              </button>
            </div>
          </div>
        </div>
      </nav>

      {/* Mobile Nav */}
      {mobileOpen && (
        <div className="lg:hidden fixed inset-0 top-16 z-40 bg-white/95 backdrop-blur-xl">
          <div className="p-4 space-y-1">
            {NAV_ITEMS.map((item) => {
              const Icon = item.icon;
              const isActive = location.pathname === item.path;
              return (
                <Link
                  key={item.path}
                  to={item.path}
                  onClick={() => setMobileOpen(false)}
                  className={`flex items-center justify-between px-4 py-3 rounded-xl text-base font-medium transition-all ${
                    isActive
                      ? 'bg-logos-100 text-logos-700'
                      : 'text-gray-600 hover:bg-logos-50'
                  }`}
                >
                  <span className="flex items-center gap-3">
                    <Icon className="w-5 h-5" />
                    {item.label}
                  </span>
                  <ChevronRight className="w-4 h-4 opacity-40" />
                </Link>
              );
            })}

            {/* Mobile wallet section */}
            <div className="pt-4 space-y-2">
              {isConnected && (
                <div className="p-3 rounded-xl bg-logos-50 border border-logos-200 mb-2">
                  <p className="text-xs text-logos-500 mb-1">Connected</p>
                  <p className="text-sm font-mono font-semibold text-logos-800 break-all">{walletAddress}</p>
                  <div className="flex items-center gap-4 mt-2 text-xs text-logos-600">
                    <span>{balance ? `${parseFloat(balance).toFixed(4)} ETH` : '—'}</span>
                    <span className="flex items-center gap-1">
                      <div className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
                      {networkName || 'Unknown'}
                    </span>
                  </div>
                </div>
              )}
              {isConnected ? (
                <button onClick={() => { disconnectWallet(); setMobileOpen(false); }} className="btn-secondary w-full text-center">
                  <LogOut className="w-4 h-4 mr-2 inline" />
                  Disconnect Wallet
                </button>
              ) : (
                <button
                  onClick={() => { connectWallet(); setMobileOpen(false); }}
                  disabled={isConnecting}
                  className="btn-primary w-full text-center"
                >
                  {isConnecting ? (
                    <Loader2 className="w-4 h-4 mr-2 inline animate-spin" />
                  ) : (
                    <Wallet className="w-4 h-4 mr-2 inline" />
                  )}
                  {isConnecting ? 'Connecting...' : 'Connect Wallet'}
                </button>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <Outlet />
      </main>

      {/* Notification */}
      {notification && <Notification message={notification.message} type={notification.type} />}

      {/* Footer */}
      <footer className="border-t border-blue-100/50 mt-16">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="flex flex-col md:flex-row items-center justify-between gap-4">
            <div className="flex items-center gap-2">
              <Shield className="w-5 h-5 text-logos-600" />
              <span className="font-bold text-gradient">LOGOS</span>
              <span className="text-gray-400 text-sm">Fee Transparency Platform</span>
            </div>
            <p className="text-sm text-gray-400">
              Powered by Logos Blockchain — Every fee accounted for, every shilling transparent.
            </p>
          </div>
        </div>
      </footer>
    </div>
  );
}
