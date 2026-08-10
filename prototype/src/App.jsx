import React from 'react';
import { Routes, Route } from 'react-router-dom';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Institutions from './pages/Institutions';
import InstitutionDetail from './pages/InstitutionDetail';
import PayFees from './pages/PayFees';
import VerifyPayment from './pages/VerifyPayment';
import Releases from './pages/Releases';
import About from './pages/About';

export default function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/" element={<Dashboard />} />
        <Route path="/institutions" element={<Institutions />} />
        <Route path="/institutions/:id" element={<InstitutionDetail />} />
        <Route path="/pay" element={<PayFees />} />
        <Route path="/verify" element={<VerifyPayment />} />
        <Route path="/releases" element={<Releases />} />
        <Route path="/about" element={<About />} />
      </Route>
    </Routes>
  );
}
