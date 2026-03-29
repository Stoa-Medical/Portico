import Link from 'next/link'
import { UserButton } from '@clerk/nextjs'

const navLinks = [
  { href: '/agents', label: 'Agents' },
  { href: '/workflows', label: 'Workflows' },
  { href: '/mappings', label: 'Mappings' },
  { href: '/integrations', label: 'Integrations' },
  { href: '/analytics', label: 'Analytics' },
]

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex min-h-screen bg-zinc-950 text-zinc-100">
      {/* Sidebar */}
      <aside className="flex w-60 flex-col border-r border-zinc-800 bg-zinc-950">
        <div className="flex h-14 items-center border-b border-zinc-800 px-4">
          <Link href="/" className="text-lg font-semibold tracking-tight text-white">
            Portico
          </Link>
        </div>
        <nav className="flex-1 space-y-1 p-3">
          {navLinks.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className="block rounded-md px-3 py-2 text-sm text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-100"
            >
              {link.label}
            </Link>
          ))}
        </nav>
      </aside>

      {/* Main area */}
      <div className="flex flex-1 flex-col">
        {/* Header */}
        <header className="flex h-14 items-center justify-end border-b border-zinc-800 px-6">
          <UserButton />
        </header>

        {/* Content */}
        <main className="flex-1 p-6">{children}</main>
      </div>
    </div>
  )
}
