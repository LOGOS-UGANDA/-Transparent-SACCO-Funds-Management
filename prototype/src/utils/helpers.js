export const INSTITUTION_TYPES = ['University', 'SACCO', 'Cooperative', 'School'];

export const FEE_STATUS = ['Locked', 'Partially Released', 'Fully Released', 'Refunded'];

export const RELEASE_STATUS = ['Pending', 'Approved', 'Rejected', 'Executed'];

export function formatAddress(address) {
  if (!address) return '';
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

export function formatCurrency(wei, decimals = 4) {
  if (!wei) return '0';
  const eth = Number(wei) / 1e18;
  return eth.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: decimals,
  });
}

export function formatDate(timestamp) {
  if (!timestamp) return '';
  const date = new Date(Number(timestamp) * 1000);
  return date.toLocaleDateString('en-UG', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function bpsToPercent(bps) {
  return (Number(bps) / 100).toFixed(1);
}

export function generateRefNumber() {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
  let result = 'LFT-';
  for (let i = 0; i < 8; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}
