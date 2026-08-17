'use client';

import dynamic from 'next/dynamic';
import type { ReactNode } from 'react';

const WalletProviders = dynamic(
  () => import('./WalletProviders').then((mod) => mod.WalletProviders),
  { ssr: false },
);

export function Providers({ children }: { children: ReactNode }) {
  return <WalletProviders>{children}</WalletProviders>;
}
