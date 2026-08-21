'use client';

import { useState } from 'react';

import { shortHash } from '@/lib/format';

export function HashChip({
  value,
  size = 4,
  label,
}: {
  value: string;
  size?: number;
  label?: string;
}) {
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
    <button
      type="button"
      className="hash-chip"
      onClick={copy}
      title={value}
      data-copied={copied}
      aria-label={label ? `Copy ${label}` : 'Copy'}
    >
      {label ? <span className="hash-chip-label">{label}</span> : null}
      <span className="hash">{shortHash(value, size)}</span>
      <span className="hash-chip-hint">{copied ? 'copied' : 'copy'}</span>
    </button>
  );
}
