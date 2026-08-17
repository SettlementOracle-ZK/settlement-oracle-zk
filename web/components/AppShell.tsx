'use client';

import dynamic from 'next/dynamic';
import Link from 'next/link';
import type { ReactNode } from 'react';

import { Nav } from './Nav';
import { OracleTicker } from './OracleTicker';

const WalletButton = dynamic(
  () => import('./WalletButton').then((mod) => mod.WalletButton),
  { ssr: false },
);

export function AppShell({ children, rail }: { children: ReactNode; rail?: ReactNode }) {
  return (
    <div className="shell">
      <header className="topbar">
        <Link className="brand" href="/">
          <strong>
            Settlement<span>Oracle</span>
          </strong>
          <em>ZK · on-chain vault</em>
        </Link>
        <Nav />
        <div className="wallet-slot">
          <OracleTicker />
          <span className="network-chip">devnet</span>
          <WalletButton />
        </div>
      </header>
      <div className="workspace">
        <main>{children}</main>
        {rail}
      </div>
    </div>
  );
}
