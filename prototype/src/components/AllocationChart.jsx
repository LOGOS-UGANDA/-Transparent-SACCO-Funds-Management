import React from 'react';
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip, Legend } from 'recharts';
import { bpsToPercent } from '../utils/helpers';

const COLORS = [
  '#2563eb', '#3b82f6', '#60a5fa', '#93c5fd',
  '#1d4ed8', '#1e40af', '#bfdbfe',
];

export default function AllocationChart({ categories, title }) {
  const data = categories.map((cat, i) => ({
    name: cat.name,
    value: Number(bpsToPercent(cat.allocatedPercentage)),
    fill: COLORS[i % COLORS.length],
  }));

  return (
    <div className="glass-card rounded-2xl p-6">
      {title && <h3 className="text-lg font-bold text-gray-900 mb-4">{title}</h3>}
      <ResponsiveContainer width="100%" height={300}>
        <PieChart>
          <Pie
            data={data}
            cx="50%"
            cy="50%"
            innerRadius={60}
            outerRadius={100}
            paddingAngle={3}
            dataKey="value"
            animationBegin={0}
            animationDuration={800}
          >
            {data.map((entry, i) => (
              <Cell key={i} fill={entry.fill} stroke="white" strokeWidth={2} />
            ))}
          </Pie>
          <Tooltip
            formatter={(val) => `${val}%`}
            contentStyle={{
              background: 'rgba(255,255,255,0.95)',
              border: '1px solid #dbeafe',
              borderRadius: '12px',
              boxShadow: '0 4px 20px rgba(0,0,0,0.08)',
            }}
          />
          <Legend
            verticalAlign="bottom"
            height={36}
            formatter={(value) => <span className="text-xs text-gray-600">{value}</span>}
          />
        </PieChart>
      </ResponsiveContainer>
    </div>
  );
}
