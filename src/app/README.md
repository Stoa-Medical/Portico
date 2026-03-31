# Web App (`app/`)

Next.js 15 App Router — the Portico dashboard and API layer.

## Structure

```
app/
├── layout.tsx              # Root layout (ClerkProvider, Geist fonts, dark mode)
├── globals.css             # Tailwind CSS global styles
│
├── (auth)/                 # Auth route group (public)
│   ├── layout.tsx          #   Centered layout for auth pages
│   ├── login/page.tsx
│   └── register/page.tsx
│
├── (dashboard)/            # Dashboard route group (protected by Clerk)
│   ├── layout.tsx          #   Sidebar nav + header with UserButton
│   ├── page.tsx            #   System overview (agent count, pending signals, recent sessions)
│   ├── agents/             #   Agent list + detail ([id])
│   ├── workflows/          #   Workflow list + detail ([id])
│   ├── mappings/           #   Data mapping list + detail ([id])
│   ├── integrations/       #   Integration connections
│   └── analytics/          #   Analytics dashboard
│
└── api/                    # API routes
    ├── chat/route.ts       #   AI chat endpoint (AI SDK + AI Gateway)
    ├── signals/route.ts    #   Signal ingestion API
    ├── webhooks/
    │   ├── hl7/route.ts    #   HL7v2 message webhook
    │   ├── fhir/route.ts   #   FHIR resource webhook
    │   └── medplum/route.ts#   Medplum subscription webhook
    └── cron/
        └── cleanup/route.ts#   Scheduled cleanup (verified with CRON_SECRET)
```

## Related Root-Level Files

These files live at the project root (not inside `app/`) per Next.js convention:

| File | Purpose |
|------|---------|
| `middleware.ts` | Clerk auth — protects `/(dashboard)(.*)` routes |
| `lib/` | Shared modules: database client, agent logic, Medplum client, analytics queries |
| `components/` | Shared UI components (shadcn/ui — added via `npx shadcn@latest add`) |
| `hooks/` | Custom React hooks |
| `next.config.ts` | Next.js configuration |
| `drizzle.config.ts` | Drizzle ORM / Neon Postgres configuration |
| `components.json` | shadcn/ui configuration |

## Running

```bash
npm install
npm run dev     # http://localhost:3000
```

Requires environment variables for Clerk, Neon Postgres, and optionally Medplum/Redis/AI Gateway. See the root README for setup.
