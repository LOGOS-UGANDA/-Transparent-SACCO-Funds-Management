import React, { createContext, useContext, useState, useCallback } from 'react';
import useWallet from '../hooks/useWallet';
import useContract from '../hooks/useContract';
import {
  DEMO_INSTITUTIONS,
  DEMO_CATEGORIES,
  DEMO_PAYMENTS,
  DEMO_RELEASES,
  PLATFORM_STATS,
} from '../utils/demoData';

const AppContext = createContext(null);

export function AppProvider({ children }) {
  const wallet = useWallet();
  const contract = useContract(wallet.provider, wallet.signer);

  const [institutions] = useState(DEMO_INSTITUTIONS);
  const [categories] = useState(DEMO_CATEGORIES);
  const [payments, setPayments] = useState(DEMO_PAYMENTS);
  const [releases] = useState(DEMO_RELEASES);
  const [stats] = useState(PLATFORM_STATS);
  const [notification, setNotification] = useState(null);

  const showNotification = useCallback((message, type = 'info') => {
    setNotification({ message, type });
    setTimeout(() => setNotification(null), 5000);
  }, []);

  const connectWallet = useCallback(async () => {
    if (!wallet.hasEthereum) {
      showNotification(
        'No Web3 wallet detected. Install MetaMask to interact with the blockchain. Running in demo mode.',
        'warning'
      );
      return false;
    }
    const ok = await wallet.connect();
    if (ok) {
      showNotification('Wallet connected successfully', 'success');
    } else if (wallet.error === 'rejected') {
      showNotification('Connection rejected — please approve in your wallet', 'warning');
    } else {
      showNotification('Failed to connect wallet', 'error');
    }
    return ok;
  }, [wallet, showNotification]);

  const disconnectWallet = useCallback(() => {
    wallet.disconnect();
    showNotification('Wallet disconnected', 'info');
  }, [wallet, showNotification]);

  const getInstitution = useCallback(
    (id) => institutions.find((i) => i.id === id),
    [institutions]
  );

  const getCategories = useCallback(
    (institutionId) => categories[institutionId] || [],
    [categories]
  );

  const verifyPayment = useCallback(
    (refOrId) => {
      const payment = payments.find(
        (p) => p.referenceNumber === refOrId || p.id === Number(refOrId)
      );
      if (!payment) return null;
      const institution = getInstitution(payment.institutionId);
      const cats = getCategories(payment.institutionId);
      const allocations = cats.map((cat) => ({
        categoryId: cat.id,
        categoryName: cat.name,
        percentage: cat.allocatedPercentage,
        amount: (BigInt(payment.amount) * BigInt(cat.allocatedPercentage)) / 10000n,
        released: false,
      }));
      return { payment, institution, allocations };
    },
    [payments, getInstitution, getCategories]
  );

  const addPayment = useCallback(
    (payment) => {
      setPayments((prev) => [...prev, { ...payment, id: prev.length + 1 }]);
    },
    []
  );

  return (
    <AppContext.Provider
      value={{
        // wallet state
        wallet,
        walletAddress: wallet.address,
        isConnected: wallet.isConnected,
        isConnecting: wallet.isConnecting,
        balance: wallet.balance,
        chainId: wallet.chainId,
        networkName: wallet.networkName,
        hasEthereum: wallet.hasEthereum,
        signer: wallet.signer,
        provider: wallet.provider,

        // contract
        contract,
        isDeployed: contract.isDeployed,
        txLoading: contract.txLoading,
        txHash: contract.txHash,
        txError: contract.txError,

        // demo data
        institutions,
        categories,
        payments,
        releases,
        stats,
        notification,

        // actions
        connectWallet,
        disconnectWallet,
        switchToLogos: wallet.switchToLogos,
        showNotification,
        getInstitution,
        getCategories,
        verifyPayment,
        addPayment,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error('useApp must be used within AppProvider');
  return ctx;
}
