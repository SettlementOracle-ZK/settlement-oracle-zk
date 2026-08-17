import type { Metadata } from 'next';
import { IBM_Plex_Mono, Outfit, Syne } from 'next/font/google';

import { Providers } from '@/components/Providers';

import './globals.css';

const display = Syne({
  subsets: ['latin'],
  variable: '--font-display',
  display: 'swap',
});

const body = Outfit({
  subsets: ['latin'],
  variable: '--font-body',
  display: 'swap',
});

const mono = IBM_Plex_Mono({
  subsets: ['latin'],
  weight: ['400', '500'],
  variable: '--font-mono',
  display: 'swap',
});

export const metadata: Metadata = {
  title: 'SettlementOracle ZK',
  description: 'On-chain parametric settlement — escrow vaults, Pyth triggers, ZK attestation.',
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body className={`${display.variable} ${body.variable} ${mono.variable}`}>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
