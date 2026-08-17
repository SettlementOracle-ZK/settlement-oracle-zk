'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';

const LINKS = [
  { href: '/', label: 'Overview' },
  { href: '/policies', label: 'Policies' },
  { href: '/explorer', label: 'Explorer' },
  { href: '/monitor', label: 'Monitor' },
];

export function Nav() {
  const pathname = usePathname();
  return (
    <nav className="nav" aria-label="Primary">
      {LINKS.map((link) => (
        <Link
          key={link.href}
          href={link.href}
          data-active={pathname === link.href ? 'true' : 'false'}
        >
          {link.label}
        </Link>
      ))}
    </nav>
  );
}
