'use client';

import { useState } from 'react';

import { shortHash } from '@/lib/format';

export function HashChip({ value, size = 4 }: { value: string; size?: number }) {
  const [copied, setCopied] = useState(false);

  async function copy() {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1200);
    } catch {
      setCopied(false);
    }
  }

  return (
    <button type="button" className="hash-chip" onClick={copy} title={value}>
      <span className="hash">{shortHash(value, size)}</span>
      <span className="hash-chip-hint">{copied ? 'copied' : 'copy'}</span>
    </button>
  );
}
