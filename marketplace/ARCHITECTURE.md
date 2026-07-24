# Wing — Build Plan & Architecture Specification

**Version 1.0 · July 2026 · Status: ready to implement**

This document is a complete, self-contained specification for building **Wing** (working title): a mobile-responsive multiuser web marketplace where dating coaches sell help — per-reply chat coaching, deliverables (bios, photo reviews, opener bundles), live sessions, packs, and custom-quoted work — and clients pay **plain dollars per item at the moment of purchase**.

> **How to use this document (instructions to the coding agent):**
> Build milestones **M0 → M4 in order** (§15). Do not skip ahead; each milestone has acceptance criteria that must pass before the next begins. Where this spec is silent, prefer the simplest implementation consistent with the invariants in §6. Anything listed in §16 (Out of scope) must NOT be built, even if it seems easy. All money logic must go through the single ledger code path (§6.2) — no exceptions. Use the design tokens in §17 exactly; the UI mockup reference describes every key screen (§10–§11).

---

## 1. Product overview & core concepts

Two roles on one account model:

- **Client** — browses coaches, buys items, gets help in a chat thread, leaves reviews.
- **Coach** — publishes a menu of offerings, fulfills purchases in threads, gets paid out via Stripe Connect.

A user can be both. There is also a minimal **admin** surface (§12.5).

### Glossary (use these exact names in code)

| Term | Meaning |
|---|---|
| **Offering** | An item on a coach's menu: what + how delivered + how charged. |
| **Engagement** | A funded purchase of an offering by a client, with frozen terms. The unit of consent. All fulfillment and consumption happens inside an engagement. |
| **Thread** | The single conversation between one client and one coach. Holds messages and cards. Engagements attach to a thread. |
| **Consumption** | The coach delivering a unit of value against an engagement (a reply, a photo verdict, session minutes). Recorded as a `usage_event`. |
| **Ledger** | Append-only double-entry record of all money movement (§6). Balances are always derived, never stored as mutable columns. |
| **Proposal** | A coach-initiated offer card (custom quote or upsell). Becomes an engagement only when the client accepts and pays. |

### The one product invariant (enforce in code, state in comments)

> **A coach's action can cost the client exactly what they already agreed to — or less. Never more.**
> The client picks the item and pays; the coach fulfills within those terms. Anything beyond the terms is a proposal requiring explicit client acceptance + payment.

---

## 2. Tech stack (pinned decisions)

| Layer | Choice | Notes |
|---|---|---|
| Framework | **Next.js 15+ (App Router) + TypeScript** | One repo, server actions for mutations, route handlers for webhooks. |
| Styling / UI | **Tailwind CSS + shadcn/ui** | Tokens from §17. Mobile-first responsive. |
| Database / Auth / Storage / Realtime | **Supabase** (Postgres 15+, Supabase Auth, Storage, Realtime) | Local dev via `supabase start` (Docker). Migrations in-repo. |
| Payments | **Stripe** (PaymentIntents, saved payment methods, Stripe Connect **Express**) | Behind a provider interface with a **mock mode** (§6.5) so the whole app runs with no Stripe keys. |
| Validation | **zod** everywhere; **react-hook-form** on forms | One schema validates client & server. |
| State machine | Plain TypeScript enum + guarded transition map (§7) | No XState dependency needed. |
| Testing | **Vitest** (unit; all money paths) + **Playwright** (e2e happy paths) | |
| Deploy target | Vercel + Supabase cloud | Nothing may depend on long-lived server processes. |

No other significant dependencies without need. **Do not** add Redux/Zustand (server components + a small thread store suffice), tRPC, or an ORM other than the Supabase client + typed SQL (generate types via `supabase gen types`).

---

## 3. Repository layout

```
wing/
  app/
    (marketing)/page.tsx            # landing
    (auth)/sign-in, sign-up, callback
    coaches/page.tsx                # discover (filters via searchParams)
    coaches/[handle]/page.tsx       # coach profile + menu
    threads/page.tsx                # inbox
    threads/[id]/page.tsx           # the thread (hero surface)
    purchases/page.tsx              # client purchase history
    reviews/new/page.tsx            # ?engagement=<id>
    coach/apply/page.tsx            # become-a-coach application
    coach/menu/page.tsx             # offering composer + list
    coach/queue/page.tsx            # fulfillment queue
    coach/earnings/page.tsx
    settings/page.tsx
    admin/page.tsx                  # applications review, disputes, reports
    api/stripe/webhook/route.ts
    api/uploads/sign/route.ts
  components/
    ui/                             # shadcn primitives
    marketplace/                    # CoachCard, MenuOfferingRow, FilterChips...
    thread/                         # MessageBubble, EngagementStrip, OfferCard,
                                    # OrderCard, ConsumptionBadge, Composer,
                                    # PurchaseSheet, VerdictRow, OpenerRow...
    coach/                          # ComposerWizard, QueueRow, EarningsTile...
  lib/
    db/                             # typed queries; ledger.ts is THE money path
    payments/                       # provider.ts (interface), stripe.ts, mock.ts
    engagements/                    # state machine + consumption logic
    validators/                     # zod schemas
    realtime/                       # thread channel helpers
  supabase/
    migrations/*.sql
    seed.sql                        # personas & demo data (§13)
  tests/
    unit/   e2e/
```

---

## 4. Environment & configuration

```
NEXT_PUBLIC_SUPABASE_URL=
NEXT_PUBLIC_SUPABASE_ANON_KEY=
SUPABASE_SERVICE_ROLE_KEY=          # server only
PAYMENTS_MODE=mock|stripe           # default: mock
STRIPE_SECRET_KEY=                  # only when PAYMENTS_MODE=stripe
STRIPE_WEBHOOK_SECRET=
STRIPE_CONNECT_CLIENT_ID=
PLATFORM_FEE_BPS=2000               # 20.00% in basis points
AUTO_APPROVE_DAYS=3
ATTACHMENT_TTL_DAYS=30
APP_URL=
```

The app must boot and be fully demoable with `PAYMENTS_MODE=mock` and only Supabase local running.

---

## 5. Database schema

All money is **integer cents**. All tables get `created_at timestamptz default now()`. Use these exact enums/tables (trim or extend columns only if a milestone requires it).

```sql
-- ENUMS
create type specialty as enum
  ('openers','bio','photos','conversations','dates','custom');
create type fulfillment_mode as enum ('chat','deliverable','live');
create type pricing_scheme as enum ('per_unit','flat','pack','subscription','quote');
create type engagement_status as enum
  ('pending_payment','active','delivered','approved','completed',
   'cancelled','refunded','disputed');
create type message_kind as enum
  ('text','attachment','proposal','engagement_update','system');
create type ledger_reason as enum
  ('purchase','consumption','escrow_release','refund','platform_fee','payout');
create type coach_status as enum ('none','applied','approved','suspended');

-- PROFILES (1 row per auth user; created by trigger on auth.users insert)
create table profiles (
  id uuid primary key references auth.users(id),
  handle text unique not null,
  display_name text not null,
  avatar_url text,
  pronouns text,
  is_coach boolean not null default false,
  coach_status coach_status not null default 'none',
  headline text,               -- coach fields below
  bio text,
  specialties specialty[] not null default '{}',
  response_time_mins int,      -- rolling median, denormalized by job
  rating_avg numeric(3,2),     -- denormalized from reviews
  rating_count int not null default 0,
  stripe_account_id text,      -- Connect Express
  stripe_customer_id text,     -- client-side saved cards
  created_at timestamptz not null default now()
);

-- OFFERINGS (a coach's menu items)
create table offerings (
  id uuid primary key default gen_random_uuid(),
  coach_id uuid not null references profiles(id),
  title text not null,
  description text,
  specialty specialty not null,
  fulfillment fulfillment_mode not null,
  scheme pricing_scheme not null,
  unit_label text,             -- 'reply' | 'photo' | 'minute' | ...
  unit_price_cents int,        -- per_unit
  flat_price_cents int,        -- flat / quote(filled at accept)
  pack_units int,              -- pack
  pack_price_cents int,        -- pack
  session_minutes int,         -- live
  required_inputs jsonb not null default '[]',  -- [{key,label,type:'text'|'images',required}]
  first_unit_free boolean not null default false,
  active boolean not null default true,
  sort int not null default 0,
  rating_avg numeric(3,2),          -- denormalized per-offering rating
  rating_count int not null default 0
);

-- THREADS (unique client↔coach conversation)
create table threads (
  id uuid primary key default gen_random_uuid(),
  client_id uuid not null references profiles(id),
  coach_id  uuid not null references profiles(id),
  last_message_at timestamptz,
  unique (client_id, coach_id)
);

-- ENGAGEMENTS (funded purchases; terms FROZEN at purchase time)
create table engagements (
  id uuid primary key default gen_random_uuid(),
  thread_id uuid not null references threads(id),
  offering_id uuid references offerings(id),     -- null for ad-hoc quotes
  client_id uuid not null references profiles(id),
  coach_id  uuid not null references profiles(id),
  status engagement_status not null default 'pending_payment',
  title text not null,                 -- snapshot
  scheme pricing_scheme not null,      -- snapshot
  fulfillment fulfillment_mode not null,
  unit_label text,
  unit_price_cents int,
  units_total int,                     -- pack: N; per_unit: purchased qty; flat/live: 1
  units_used int not null default 0,
  amount_cents int not null,           -- total client paid
  fee_cents int not null,              -- platform fee at PLATFORM_FEE_BPS
  payment_intent_id text,
  client_inputs jsonb,
  delivered_at timestamptz,
  auto_approve_at timestamptz,
  completed_at timestamptz
);

-- A thread holds MANY engagements (active pack + delivered bio + booked call
-- simultaneously). Exactly one chat-metered engagement may be "armed" per
-- thread; fulfilling replies consume the armed one only. (Added here because
-- engagements is defined after threads.)
alter table threads add column armed_engagement_id uuid references engagements(id);

-- MESSAGES
create table messages (
  id uuid primary key default gen_random_uuid(),
  thread_id uuid not null references threads(id),
  sender_id uuid not null references profiles(id),
  kind message_kind not null default 'text',
  body text,
  payload jsonb,               -- proposal terms / engagement_update snapshot / verdicts
  attachment_path text,        -- Supabase Storage key
  engagement_id uuid references engagements(id),  -- set when this message consumed
  units_consumed int not null default 0,
  created_at timestamptz not null default now()
);
create index on messages (thread_id, created_at);

-- USAGE EVENTS (uniform consumption record)
create table usage_events (
  id uuid primary key default gen_random_uuid(),
  engagement_id uuid not null references engagements(id),
  message_id uuid references messages(id),
  qty int not null,
  unit text not null,
  created_at timestamptz not null default now()
);

-- MONEY LEDGER (append-only, double-entry: every txn balances to zero)
create table ledger_entries (
  id bigint generated always as identity primary key,
  txn_id uuid not null,                -- groups the balanced legs of one transaction
  account text not null,               -- 'client:<uuid>' | 'escrow:<engagement_uuid>'
                                       -- | 'coach:<uuid>' | 'platform:fees' | 'external:stripe'
  delta_cents int not null,
  reason ledger_reason not null,
  ref_id uuid,                         -- engagement / payout id
  created_at timestamptz not null default now()
);
create index on ledger_entries (account);
create index on ledger_entries (txn_id);

-- REVIEWS (one per completed engagement — reviews are PER ITEM)
create table reviews (
  id uuid primary key default gen_random_uuid(),
  engagement_id uuid unique not null references engagements(id),
  offering_id uuid references offerings(id),   -- snapshot from engagement; null for ad-hoc quotes
  client_id uuid not null references profiles(id),
  coach_id uuid not null references profiles(id),
  stars int not null check (stars between 1 and 5),
  outcome_tags text[] not null default '{}',   -- 'got_reply','date_set','faster','better_profile'
  body text
);
create index on reviews (offering_id);
create index on reviews (coach_id);

-- DISPUTES / HELP REQUESTS
create table help_requests (
  id uuid primary key default gen_random_uuid(),
  engagement_id uuid not null references engagements(id),
  opened_by uuid not null references profiles(id),
  type text not null check (type in ('changes','refund','report')),
  body text,
  status text not null default 'open' check (status in ('open','resolved','escalated')),
  resolved_at timestamptz
);

-- COACH APPLICATIONS
create table coach_applications (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references profiles(id),
  specialties specialty[] not null,
  proof text,
  audition_answer text not null,
  status text not null default 'pending' check (status in ('pending','approved','rejected')),
  reviewed_at timestamptz
);

-- NOTIFICATIONS (in-app; email later)
create table notifications (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references profiles(id),
  type text not null,        -- 'new_message','engagement_update','payout','review'
  payload jsonb not null,
  read_at timestamptz
);
```

### Row-Level Security (RLS) — enable on every table

| Table | Policy summary |
|---|---|
| profiles | Read: public columns of coaches (approved only) + own row. Write: own row (never `coach_status`, `rating_*`, `stripe_*` — server-only via service role). |
| offerings | Read: `active` of approved coaches + own. Write: own (coach). |
| threads / messages | Read/write: participants only (`client_id` or `coach_id` = `auth.uid()`). Message **inserts of kind `text`/`attachment` only**; all other kinds and any row with `units_consumed > 0` are written exclusively by server actions using the service role. |
| engagements / usage_events / ledger_entries | Read: own (participant). **No client-side writes at all** — service role only. |
| reviews | Read: public. Insert: client of a `completed`/`approved` engagement without an existing review. |
| help_requests, coach_applications, notifications | Own rows; admin reads all (via service role + admin check). |

Admin = `profiles.id` present in an `admins` table (simple allowlist).

---

## 6. Money architecture

### 6.1 Model: dollars per item at purchase time

- Every purchase = one Stripe PaymentIntent charged immediately (saved card / Apple Pay / Google Pay via Payment Element).
- Client money sits in **escrow** (a ledger account per engagement) until released by delivery + approval.
- Coach earnings accumulate in their ledger account; payouts transfer to their Stripe Connect Express account (mock mode: simulated).
- **No stored-balance wallet. No credits.** (Explicitly out of scope, §16.)
- Charge pattern: **separate charges & transfers** — charge to the platform account, `Transfer` to the connected account on escrow release.

### 6.2 The single money code path

`lib/db/ledger.ts` exposes exactly these functions; **nothing else in the codebase writes `ledger_entries` or mutates engagement money fields**:

```ts
recordPurchase(engagement, paymentIntentId)     // external:stripe → escrow:<eng>
recordConsumption(engagement, usageEvent)       // marks units_used; no money moves yet
releaseEscrow(engagement)                       // escrow → coach:<id> (net) + platform:fees
recordRefund(engagement, amountCents)           // escrow → external:stripe (reverse)
recordPayout(coachId, amountCents, transferId)  // coach:<id> → external:stripe
```

Each runs in **one Postgres transaction**, inserts balanced legs sharing a `txn_id` (sum of `delta_cents` per txn = 0), and enforces guards (e.g. release requires status `approved`; refund requires escrow balance ≥ amount). Unit-test all of these exhaustively, including double-call idempotency (pass an idempotency key = txn_id derived from `(engagement_id, reason)` where applicable).

### 6.3 Flow: purchase → fulfill → release

1. Client taps an offering → server action `createEngagement` snapshots terms, computes `amount_cents` + `fee_cents`, creates PaymentIntent (`metadata.engagement_id`), returns client secret → PurchaseSheet confirms payment.
2. Webhook `payment_intent.succeeded` → verify signature → `recordPurchase` → status `active` → system message `engagement_update` into the thread.
3. Coach fulfills. For chat schemes: the composer's **Fulfill toggle** calls `sendFulfillingMessage(threadId, engagementId, …)` (server action): single transaction verifies the engagement is **the thread's armed engagement** (`threads.armed_engagement_id`), is `active`, and has `units_used < units_total` → inserts message with `units_consumed` → inserts `usage_event` → increments `units_used`. Free messages skip all of this. See §6.7 for arming rules.
4. Deliverable/live: coach calls `markDelivered` (attaches files/payload) → status `delivered`, `auto_approve_at = now() + AUTO_APPROVE_DAYS`.
5. Client `approve` (or the auto-approve job fires) → status `approved` → `releaseEscrow` → status `completed` → review prompt.
   Chat engagements auto-complete when `units_used = units_total` (release then too); partially-used engagements auto-release **consumed value only** after `AUTO_APPROVE_DAYS` of thread inactivity, refunding the remainder — implement as one scheduled job (§12.4).
6. `openHelpRequest('refund')` → status `disputed`, freezes release; admin resolves → `recordRefund` and/or partial `releaseEscrow`.

### 6.4 Proposals (custom quotes & upsells)

`proposeEngagement` (coach): inserts a `proposal` message with `payload = {title, price_cents, description, scheme:'quote'|'flat', ...}`. Client accept → same `createEngagement` path with `offering_id = null` and the proposal payload as the snapshot. Declining just marks the message payload declined. Proposals expire after 7 days.

### 6.5 Payments provider interface + mock

```ts
interface PaymentsProvider {
  createPaymentIntent(i: {amountCents; customerId?; metadata}): Promise<{id; clientSecret}>
  createCustomer(u): Promise<{id}>
  createConnectAccount(u): Promise<{id; onboardingUrl}>
  transfer(t: {destinationAccountId; amountCents; metadata}): Promise<{id}>
  refund(r: {paymentIntentId; amountCents}): Promise<{id}>
}
```

`mock.ts`: succeeds instantly, and the PurchaseSheet in mock mode shows a fake "Pay" that directly invokes the same webhook-handler logic (call the handler function, not the HTTP route). **All demos, tests, and seed flows run in mock mode.**

### 6.6 Multiple engagements per thread — the "armed" rule

A thread holds any number of engagements over its lifetime, concurrently and sequentially. Disambiguation:

- **Card-scoped engagements** (deliverable, live, quote): always addressed by their own card/id — never ambiguous; any number may coexist.
- **Chat-metered engagements** (per_unit, pack): at most **one is armed** per thread (`threads.armed_engagement_id`). Fulfilling replies consume the armed engagement only.
- **Arming**: a newly funded chat-metered engagement auto-arms itself (in the purchase webhook path). The client switches via `armEngagement(threadId, engagementId)` (validates: caller is the thread's client, engagement is theirs, chat-metered, `active` with units remaining). **The coach can never arm.** If nothing is armed or the armed item is exhausted, `sendFulfillingMessage` fails and the UI disables Fulfill — the coach can still send free messages and proposals.
- **Attribution**: consumption badges, `usage_events`, and receipts always reference their engagement, and the UI renders the name ("1 used · Convo pack · 5 left") so thread history doubles as a receipt trail.
- On engagement completion/refund, if it was armed, null out `armed_engagement_id` (same transaction).

### 6.7 Per-reply purchases (MVP simplification)

MVP sells per-unit chat help as **small prepaid quantities**: a per_unit offering's purchase sheet lets the client pick qty 1 / 3 / 5 (one charge, e.g. 3 replies = $12) — creating a normal engagement with `units_total = qty`. One-tap **"Buy another reply"** re-purchase (off-session PaymentIntent on the saved card) when units run out. Daily micro-charge batching is **phase 2** — do not build now.

---

## 7. Engagement state machine

Implement as a transition map checked by every server action; illegal transitions throw.

| From | Event (actor) | To | Side effects |
|---|---|---|---|
| pending_payment | payment_succeeded (webhook) | active | recordPurchase; system msg |
| pending_payment | cancel (client) / expire 24h (job) | cancelled | cancel PI |
| active | consume (coach) — chat schemes | active | usage_event; units_used++ |
| active | last unit consumed (system) | approved | auto: releaseEscrow → completed |
| active | mark_delivered (coach) — deliverable/live | delivered | set auto_approve_at; system msg |
| delivered | approve (client) / auto_approve (job) | approved | releaseEscrow |
| approved | (immediate, system) | completed | review prompt notification |
| active, delivered | open_help:refund (client) | disputed | freeze releases |
| disputed | resolve (admin) | completed / refunded | releaseEscrow and/or recordRefund |

`subscription` scheme: **phase 2** (§16) — the enum exists, the UI hides it.

---

## 8. API surface (server actions unless noted)

All actions validate with zod, check auth + role, and return typed results. Names are canonical:

**Marketplace** — `listCoaches({specialty?, priceMaxCents?, minRating?, query?, cursor?})`, `getCoachProfile(handle)`.
**Offerings (coach)** — `createOffering`, `updateOffering`, `setOfferingActive`, `reorderOfferings`.
**Purchases** — `createEngagement({offeringId | proposalMessageId, qty?, clientInputs})`, `cancelPendingEngagement`, `rebuyUnits(engagementId, qty)`.
**Thread** — `getOrCreateThread(coachId)`, `listThreads()`, `listMessages(threadId, cursor)`, `sendMessage(threadId, {body?, attachmentPath?})` (free), `sendFulfillingMessage(threadId, engagementId, {body, payload?})`, `armEngagement(threadId, engagementId)` (client only; §6.6), `proposeEngagement(threadId, terms)`, `respondToProposal(messageId, accept)`.
**Deliverables** — `markDelivered(engagementId, {payload, attachmentPaths})`, `approveEngagement(engagementId)`, `requestChanges(engagementId, note)`.
**Reviews** — `submitReview(engagementId, {stars, outcomeTags, body})`.
**Help** — `openHelpRequest(engagementId, type, body)`, `resolveHelpRequest(id, resolution)` (admin).
**Coach lifecycle** — `submitCoachApplication`, `reviewApplication(id, approve)` (admin), `startConnectOnboarding()`, `getEarnings()` (tiles + ledger rows), `requestPayout()` (or weekly auto-payout job).
**Route handlers** — `POST /api/stripe/webhook` (signature-verified; idempotent by event id), `POST /api/uploads/sign` (returns signed upload URL; enforces size/type: png/jpg/webp/txt/pdf ≤ 10 MB).

---

## 9. Realtime

- One Supabase Realtime channel per thread (`thread:<id>`): postgres_changes on `messages` (insert) and `engagements` (update) scoped by id; presence for the "● online now" indicator.
- Inbox subscribes to the user's threads for unread badges (`last_message_at` vs a `thread_reads` table — add it: `(thread_id, user_id, last_read_at)`).
- Fallback: SWR polling every 10 s if the socket drops. No message sending over the socket — all writes go through server actions.

---

## 10. Frontend: routes, components, responsive rules

**The UI reference is the mockup document ("Wing — UI/UX Design Pitch v2"); replicate its layouts and components.** Key rules:

- **Mobile-first**: every screen designed at 390 px, then adapted up. Desktop: discover becomes a grid; the thread becomes a two-pane inbox→thread layout for coaches; max content width 1100 px.
- **Money component language** (build once in `components/thread/`, reuse everywhere):
  `EngagementTray` (pinned bar: armed item's meter — "$4/reply · $8 so far" | "7 of 10 left" | timer — plus "+N items ▾"; expands to a bottom sheet listing every engagement in the thread with status/meters, an arming radio on chat-metered items, and "Add from menu"), `ConsumptionBadge` (rose, mono, names its engagement: "1 USED · CONVO PACK · 6 LEFT"), `FreeBadge` (sage: FREE / INCLUDED), `ProposalCard` (Accept·$X / Not now), `OrderCard` (PAID → IN PROGRESS → DELIVERED → APPROVED timeline, file chips, approve/request-changes, auto-approve note), `PurchaseSheet` (bottom sheet: item, price, payment method, one Pay button, "Charged now, once" note), `VerdictRow` (KEEP/CUT/RESHOOT), `OpenerRow` (numbered + copy-to-clipboard), `GroupedAnswer` (rose spine, one consumption for multiple bubbles).
- Composer (coach side) has a **Free / Fulfill toggle**, defaulting per engagement state; client composer is always free.
- Dark mode: token-level via CSS custom properties (§17), `prefers-color-scheme` + `data-theme` override; ships in M4.
- Accessibility: money semantics never color-only (badges carry words); focus states; `prefers-reduced-motion`; 4.5:1 contrast.

---

## 11. Key implementation notes per surface

- **Discover**: server component; filters in searchParams; Postgres full-text (`to_tsvector` on display_name+headline+bio) + array-contains on specialties; order by rating_avg desc, rating_count desc. No search service.
- **Thread**: virtualize only if needed (start simple). Message payloads render by `kind`; `engagement_update` renders OrderCard/state chips from snapshot payload so history stays correct even after later changes.
- **Uploads**: client requests signed URL → uploads to Storage bucket `attachments/` → sends message with path. Images render blurred until tapped (CSS blur + click-to-reveal). A scheduled job deletes attachment objects older than `ATTACHMENT_TTL_DAYS` and nulls `attachment_path` (§12.4).
- **Reviews are per item**: prompt appears in-thread on completion + on the purchases page, and names the completed item; only `approved/completed` engagements without a review; outcome tags are the four canonical strings (§5). `submitReview` copies `offering_id` from the engagement (null for ad-hoc quotes) and, in the same transaction, recomputes the denormalized aggregates at **both levels**: `offerings.rating_avg/rating_count` and `profiles.rating_avg/rating_count` (the coach roll-up = across all their reviews, including quote reviews). Menu rows, discover cards, and the coach profile render the per-offering rating next to price; reviews on the profile are filterable by offering.
- **Coach queue**: one query across the coach's engagements: active chat engagements with client-last message (needs reply), `delivered=false` deliverables with `auto_approve_at` deadlines, open proposals, open help requests — sorted by urgency.
- **Earnings**: tiles (post-fee week total, rating, repeat rate) + ledger rows for `coach:<id>`; payout row shows next scheduled payout (weekly job) and Connect status.

---

## 12. Trust, safety, privacy, ops

1. **Privacy defaults**: screenshots auto-expire (TTL job); no message content in notification payloads (only "New message from Maya"); self-serve account deletion (soft-delete profile, purge attachments, keep anonymized ledger for accounting).
2. **The coaching line**: static ToS page + coach application checkbox: coaches advise and draft; they never operate a client's dating account. `report` help-request type feeds the admin queue.
3. **Rate limits** (middleware, per-user): messages 30/min, purchases 10/hour, applications 3/day.
4. **Scheduled jobs** (Vercel cron or Supabase pg_cron; both acceptable): auto-approve deliverables past `auto_approve_at`; auto-settle inactive chat engagements (§6.3.5); expire pending_payment > 24 h; expire proposals > 7 d; attachment TTL purge; weekly payouts; recompute `response_time_mins`.
5. **Admin page**: tables for coach applications (approve/reject), open help requests (resolve with refund/release), reports. Plain, functional, service-role backed, allowlist-gated.
6. **Audit**: ledger is the money audit; add `engagement_events(engagement_id, event, actor_id, at)` written by the state machine for a full lifecycle trail.
7. **Review-fraud defenses** (build the storage in M3, the scoring in M4):
   - Store Stripe's card fingerprint on profiles (`payment_fingerprints text[]`, server-only). A fingerprint shared across multiple client accounts reviewing the same coach links those reviews; reviews from linked accounts are voided together. Prepaid/virtual card purchases mark the review low-trust.
   - **Ranking score ≠ displayed average.** Display the true avg/count, but rank with a Bayesian-shrunk, credibility-weighted score: weight each review by account age, payment-method uniqueness, breadth (has the reviewer bought from ≥2 coaches?), and engagement depth (messages exchanged, days active, units consumed). Shrink toward the marketplace prior until count is meaningful — small batches of manufactured 5★s must not outrank an established 4.8★.
   - One review per client per offering (already enforced by `engagement_id unique`); a repeat purchase lets the client *update* their review, never stack a new one.
   - Tripwire flags into `help_requests`-style admin queue (`fraud_flags(coach_id, rule, evidence jsonb, created_at)`): review-rate burst vs. trailing baseline; zero/minimal-consumption engagements completed then reviewed; review submitted < N minutes after purchase; text-similarity cluster across a coach's reviews; same-device/IP correlation between coach and reviewer sessions. Flagged reviews are de-weighted immediately, removed only by admin decision.
   - Enforcement path: weekly payout delay is the clawback window — confirmed fraud reverses escrow via `recordRefund`/withheld `releaseEscrow` before payout, purges the reviews, recomputes aggregates, and suspends the coach (`coach_status='suspended'`). Coaches are KYC'd via Connect, so a ban is loss of a verified identity, not a throwaway profile.

---

## 13. Seed data (`supabase/seed.sql` + `pnpm seed`)

Create demo users (password `demo1234`): clients `jordan@demo.wing`, `alex@demo.wing`, `riley@demo.wing`; coaches matching the mockups — **Maya R.** (openers/conversations: $4 per-reply first-free, $25 pack of 10, $18 five-opener bundle, $50 live 30-min), **Dev K.** (bio: $30 makeover flat), **Sam T.** (photos: $5 per-photo, $20 four-photo review), **Elena V.** (strategy for women: $6 per-reply, free intro), **Marcus H.** (40+ restart: $45 flat), **Priya N.** (date planning: quote-only), **Jo A.** (queer dating: $5 per-reply). Seed: ~30 reviews with outcome tags, each linked to its offering so per-item ratings render on menus; one thread per demo pairing in varied states (active pack partially consumed, delivered bio awaiting approval, accepted quote with delivered itinerary, open refund request); matching ledger entries so earnings pages are non-empty. Seed must run in mock payments mode and leave the ledger balanced.

---

## 14. Testing & CI

- **Unit (Vitest)**: every `ledger.ts` function (balanced legs, guards, idempotency); state machine transition table (legal + illegal); consumption race (two concurrent `sendFulfillingMessage` on 1 remaining unit → exactly one succeeds — use `select … for update`).
- **e2e (Playwright, mock payments)**: (1) browse → buy pack → coach fulfills 2 replies (1 free clarifier) → meter correct → review; (2) bio makeover: buy → deliver → approve → coach earnings reflect net; (3) custom quote: request → propose → accept → deliver → auto-approve via time-travel (inject clock).
- CI: typecheck, lint, unit, e2e against local Supabase. Single-threaded e2e (shared DB).

---

## 15. Build milestones (execute in order)

**M0 — Foundation & browsable marketplace.** Repo scaffold, Supabase local, migrations, auth (email + Google), profiles, seeded coaches, discover + coach profile pages (menu rendered, CTA disabled), responsive shell + tokens. ✔ *Accept: sign up, browse seeded coaches with filters on mobile viewport.*

**M1 — Money rails.** Ledger + tests, engagements + state machine, PurchaseSheet with mock provider, webhook handler, purchases page, Connect onboarding stub, admin allowlist. ✔ *Accept: buy a flat item end-to-end in mock mode; ledger balanced; unit tests green.*

**M2 — The thread.** Threads/messages/realtime, EngagementTray + money components, free vs fulfilling send with consumption transaction against the armed engagement, arming (auto-arm on purchase + client switch), packs & per-unit (qty picker + rebuy), deliverable flow with OrderCard + approve/auto-approve job, proposals/quotes. ✔ *Accept: e2e #1 and #3 pass, plus: two concurrent chat engagements in one thread — consumption follows the armed one, switching re-routes it, badges name their source.*

**M3 — Reputation & queue.** Reviews (gating, outcome tags, per-offering + per-coach aggregates rendered on menu rows and profiles), coach queue, earnings + weekly payout job, help requests + admin resolution (refund path), coach application + audition + admin approval. ✔ *Accept: e2e #2; refund resolution reverses escrow correctly.*

**M4 — Polish & launch readiness.** Dark mode, notifications + unread badges, report/block, rate limits, attachment TTL + privacy jobs, landing page, empty states, a11y pass, real-Stripe smoke test behind `PAYMENTS_MODE=stripe`. ✔ *Accept: full demo script on a phone-sized viewport, both themes, no dead ends.*

---

## 16. Explicitly OUT of scope (do not build)

Stored-balance wallet / credits · subscriptions (enum exists; UI hidden) · daily micro-charge batching · native apps (responsive web only) · in-app video infra (live sessions link out to a video URL field the coach provides; embed later) · intros/matchmaking · AI features of any kind · multi-coach group threads · i18n (structure copy in one `strings.ts` for later) · email digests (in-app notifications only).

---

## 17. Design tokens (implement as CSS custom properties)

| Token | Light | Dark | Use |
|---|---|---|---|
| `--bg` | `#FFFBF8` | `#17131A` | app ground |
| `--surface` | `#FFFFFF` | `#201A22` | cards |
| `--ink` | `#2A2230` | `#F1EAEE` | text; client bubbles bg (light) |
| `--ink-soft` | `#6E6068` | `#C4B8C0` | secondary text |
| `--line` | `#F0E4DD` | `#362E3B` | borders |
| `--accent` | `#D6456A` | `#F2718F` | money, primary CTA — **only** these |
| `--accent-soft` | `#FBEEF1` | `#3A2530` | charge badges bg |
| `--positive` | `#2F5D62` | `#7FC0C3` | FREE/INCLUDED, success — never for CTAs |
| `--positive-soft` | `#E9F2F0` | `#1E2E30` | free badge bg |
| `--gold` | `#E0A03C` | `#E0A64E` | star ratings only |

Type: serif display (`Iowan Old Style, Palatino, Georgia, serif`) for voice moments (greetings, section titles); system sans for UI; monospace for **all** dollar amounts, meters, and status badges. Radii: cards 14–16 px, sheets 22 px top, pills 999. One signature motion: consumption badge tick on fulfillment; all else instant; respect reduced-motion.

---

*End of specification. Begin with M0.*
