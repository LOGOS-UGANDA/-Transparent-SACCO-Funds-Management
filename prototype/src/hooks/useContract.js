import { useState, useCallback, useMemo } from 'react';
import { Contract, parseEther } from 'ethers';
import { CONTRACT_ADDRESS, FEE_TRANSPARENCY_ABI } from '../utils/contractABI';

export default function useContract(provider, signer) {
  const [txLoading, setTxLoading] = useState(false);
  const [txHash, setTxHash] = useState(null);
  const [txError, setTxError] = useState(null);

  const isDeployed = CONTRACT_ADDRESS !== '0x0000000000000000000000000000000000000000';

  const readContract = useMemo(() => {
    if (!provider || !isDeployed) return null;
    try {
      return new Contract(CONTRACT_ADDRESS, FEE_TRANSPARENCY_ABI, provider);
    } catch {
      return null;
    }
  }, [provider, isDeployed]);

  const writeContract = useMemo(() => {
    if (!signer || !isDeployed) return null;
    try {
      return new Contract(CONTRACT_ADDRESS, FEE_TRANSPARENCY_ABI, signer);
    } catch {
      return null;
    }
  }, [signer, isDeployed]);

  const resetTx = useCallback(() => {
    setTxHash(null);
    setTxError(null);
    setTxLoading(false);
  }, []);

  // ─── Write operations ───

  const payFees = useCallback(async (institutionId, payerName, referenceNumber, amountEth) => {
    if (!writeContract) throw new Error('Contract not available — connect wallet first');
    setTxLoading(true);
    setTxError(null);
    setTxHash(null);
    try {
      const tx = await writeContract.payFees(
        institutionId,
        payerName,
        referenceNumber,
        { value: parseEther(amountEth) }
      );
      setTxHash(tx.hash);
      const receipt = await tx.wait();
      setTxLoading(false);
      return { tx, receipt };
    } catch (err) {
      const message = err?.reason || err?.message || 'Transaction failed';
      setTxError(message);
      setTxLoading(false);
      throw err;
    }
  }, [writeContract]);

  const registerInstitution = useCallback(async (name, regNumber, type, treasury) => {
    if (!writeContract) throw new Error('Contract not available');
    setTxLoading(true);
    setTxError(null);
    try {
      const tx = await writeContract.registerInstitution(name, regNumber, type, treasury);
      setTxHash(tx.hash);
      const receipt = await tx.wait();
      setTxLoading(false);
      return { tx, receipt };
    } catch (err) {
      setTxError(err?.reason || err?.message || 'Transaction failed');
      setTxLoading(false);
      throw err;
    }
  }, [writeContract]);

  const createFeeCategory = useCallback(async (institutionId, name, description, percentBps) => {
    if (!writeContract) throw new Error('Contract not available');
    setTxLoading(true);
    setTxError(null);
    try {
      const tx = await writeContract.createFeeCategory(institutionId, name, description, percentBps);
      setTxHash(tx.hash);
      const receipt = await tx.wait();
      setTxLoading(false);
      return { tx, receipt };
    } catch (err) {
      setTxError(err?.reason || err?.message || 'Transaction failed');
      setTxLoading(false);
      throw err;
    }
  }, [writeContract]);

  const requestRelease = useCallback(async (institutionId, categoryId, amount, purpose, evidenceHash) => {
    if (!writeContract) throw new Error('Contract not available');
    setTxLoading(true);
    setTxError(null);
    try {
      const tx = await writeContract.requestRelease(institutionId, categoryId, amount, purpose, evidenceHash);
      setTxHash(tx.hash);
      const receipt = await tx.wait();
      setTxLoading(false);
      return { tx, receipt };
    } catch (err) {
      setTxError(err?.reason || err?.message || 'Transaction failed');
      setTxLoading(false);
      throw err;
    }
  }, [writeContract]);

  const approveRelease = useCallback(async (releaseId) => {
    if (!writeContract) throw new Error('Contract not available');
    setTxLoading(true);
    setTxError(null);
    try {
      const tx = await writeContract.approveRelease(releaseId);
      setTxHash(tx.hash);
      const receipt = await tx.wait();
      setTxLoading(false);
      return { tx, receipt };
    } catch (err) {
      setTxError(err?.reason || err?.message || 'Transaction failed');
      setTxLoading(false);
      throw err;
    }
  }, [writeContract]);

  // ─── Read operations ───

  const getInstitution = useCallback(async (id) => {
    if (!readContract) return null;
    try { return await readContract.getInstitution(id); } catch { return null; }
  }, [readContract]);

  const getPayment = useCallback(async (id) => {
    if (!readContract) return null;
    try { return await readContract.getPayment(id); } catch { return null; }
  }, [readContract]);

  const verifyPaymentOnChain = useCallback(async (paymentId) => {
    if (!readContract) return null;
    try { return await readContract.verifyPaymentByRef(paymentId); } catch { return null; }
  }, [readContract]);

  const getPaymentAllocations = useCallback(async (paymentId) => {
    if (!readContract) return null;
    try { return await readContract.getPaymentAllocations(paymentId); } catch { return null; }
  }, [readContract]);

  const getInstitutionCategories = useCallback(async (institutionId) => {
    if (!readContract) return null;
    try { return await readContract.getInstitutionCategories(institutionId); } catch { return null; }
  }, [readContract]);

  const getFeeCategory = useCallback(async (categoryId) => {
    if (!readContract) return null;
    try { return await readContract.feeCategories(categoryId); } catch { return null; }
  }, [readContract]);

  const getContractBalance = useCallback(async () => {
    if (!readContract) return null;
    try { return await readContract.getContractBalance(); } catch { return null; }
  }, [readContract]);

  const getTotalInstitutions = useCallback(async () => {
    if (!readContract) return null;
    try { return await readContract.getTotalInstitutions(); } catch { return null; }
  }, [readContract]);

  const getTotalPayments = useCallback(async () => {
    if (!readContract) return null;
    try { return await readContract.getTotalPayments(); } catch { return null; }
  }, [readContract]);

  const getPayerPayments = useCallback(async (payerAddress) => {
    if (!readContract) return null;
    try { return await readContract.getPayerPayments(payerAddress); } catch { return null; }
  }, [readContract]);

  return {
    isDeployed,
    readContract,
    writeContract,
    txLoading,
    txHash,
    txError,
    resetTx,

    payFees,
    registerInstitution,
    createFeeCategory,
    requestRelease,
    approveRelease,

    getInstitution,
    getPayment,
    verifyPaymentOnChain,
    getPaymentAllocations,
    getInstitutionCategories,
    getFeeCategory,
    getContractBalance,
    getTotalInstitutions,
    getTotalPayments,
    getPayerPayments,
  };
}
