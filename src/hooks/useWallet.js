import { useState, useCallback, useEffect, useRef } from 'react';
import { BrowserProvider, formatEther } from 'ethers';

const LOGOS_CHAIN = {
  chainId: import.meta.env.VITE_CHAIN_ID || '0x7A69',
  chainName: import.meta.env.VITE_CHAIN_NAME || 'Logos Blockchain',
  rpcUrls: [import.meta.env.VITE_RPC_URL || 'http://127.0.0.1:8545'],
  nativeCurrency: { name: 'ETH', symbol: 'ETH', decimals: 18 },
};

export default function useWallet() {
  const [address, setAddress] = useState(null);
  const [balance, setBalance] = useState(null);
  const [chainId, setChainId] = useState(null);
  const [isConnected, setIsConnected] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);
  const [provider, setProvider] = useState(null);
  const [signer, setSigner] = useState(null);
  const [error, setError] = useState(null);
  const [networkName, setNetworkName] = useState(null);
  const providerRef = useRef(null);

  const hasEthereum = typeof window !== 'undefined' && typeof window.ethereum !== 'undefined';

  const fetchBalance = useCallback(async (addr, prov) => {
    try {
      const bal = await prov.getBalance(addr);
      setBalance(formatEther(bal));
    } catch {
      setBalance(null);
    }
  }, []);

  const fetchChainInfo = useCallback(async (prov) => {
    try {
      const network = await prov.getNetwork();
      setChainId('0x' + network.chainId.toString(16));
      setNetworkName(network.name === 'unknown' ? 'Logos' : network.name);
    } catch {
      setChainId(null);
      setNetworkName(null);
    }
  }, []);

  const connect = useCallback(async () => {
    setError(null);

    if (!hasEthereum) {
      setError('no_wallet');
      return false;
    }

    setIsConnecting(true);
    try {
      const browserProvider = new BrowserProvider(window.ethereum);
      const accounts = await browserProvider.send('eth_requestAccounts', []);

      if (accounts.length === 0) {
        setError('no_accounts');
        setIsConnecting(false);
        return false;
      }

      const walletSigner = await browserProvider.getSigner();
      const addr = await walletSigner.getAddress();

      providerRef.current = browserProvider;
      setProvider(browserProvider);
      setSigner(walletSigner);
      setAddress(addr);
      setIsConnected(true);

      await fetchBalance(addr, browserProvider);
      await fetchChainInfo(browserProvider);

      setIsConnecting(false);
      return true;
    } catch (err) {
      if (err.code === 4001) {
        setError('rejected');
      } else {
        setError('connection_failed');
      }
      setIsConnecting(false);
      return false;
    }
  }, [hasEthereum, fetchBalance, fetchChainInfo]);

  const disconnect = useCallback(() => {
    setAddress(null);
    setBalance(null);
    setChainId(null);
    setIsConnected(false);
    setProvider(null);
    setSigner(null);
    setNetworkName(null);
    setError(null);
    providerRef.current = null;
  }, []);

  const switchToLogos = useCallback(async () => {
    if (!hasEthereum) return false;
    try {
      await window.ethereum.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: LOGOS_CHAIN.chainId }],
      });
      return true;
    } catch (switchErr) {
      if (switchErr.code === 4902) {
        try {
          await window.ethereum.request({
            method: 'wallet_addEthereumChain',
            params: [LOGOS_CHAIN],
          });
          return true;
        } catch {
          return false;
        }
      }
      return false;
    }
  }, [hasEthereum]);

  useEffect(() => {
    if (!hasEthereum) return;

    const handleAccountsChanged = async (accounts) => {
      if (accounts.length === 0) {
        disconnect();
      } else if (providerRef.current) {
        const newSigner = await providerRef.current.getSigner();
        const addr = await newSigner.getAddress();
        setSigner(newSigner);
        setAddress(addr);
        await fetchBalance(addr, providerRef.current);
      }
    };

    const handleChainChanged = () => {
      window.location.reload();
    };

    window.ethereum.on('accountsChanged', handleAccountsChanged);
    window.ethereum.on('chainChanged', handleChainChanged);

    return () => {
      window.ethereum.removeListener('accountsChanged', handleAccountsChanged);
      window.ethereum.removeListener('chainChanged', handleChainChanged);
    };
  }, [hasEthereum, disconnect, fetchBalance]);

  // Auto-reconnect if already authorized
  useEffect(() => {
    if (!hasEthereum) return;
    (async () => {
      try {
        const accounts = await window.ethereum.request({ method: 'eth_accounts' });
        if (accounts.length > 0) {
          await connect();
        }
      } catch {
        // silent
      }
    })();
  }, [hasEthereum]); // eslint-disable-line react-hooks/exhaustive-deps

  return {
    address,
    balance,
    chainId,
    networkName,
    isConnected,
    isConnecting,
    provider,
    signer,
    error,
    hasEthereum,
    connect,
    disconnect,
    switchToLogos,
    LOGOS_CHAIN,
  };
}
