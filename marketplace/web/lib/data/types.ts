// Domain types mirroring ARCHITECTURE.md §5. The mock data layer (seed.ts)
// and the future Supabase layer both satisfy these shapes.

export type Specialty =
  | "openers"
  | "bio"
  | "photos"
  | "conversations"
  | "dates"
  | "custom";

export type FulfillmentMode = "chat" | "deliverable" | "live";

export type PricingScheme = "per_unit" | "flat" | "pack" | "subscription" | "quote";

export interface Offering {
  id: string;
  coachHandle: string;
  title: string;
  description: string;
  specialty: Specialty;
  fulfillment: FulfillmentMode;
  scheme: PricingScheme;
  unitLabel?: string;
  unitPriceCents?: number;
  flatPriceCents?: number;
  packUnits?: number;
  packPriceCents?: number;
  sessionMinutes?: number;
  firstUnitFree?: boolean;
  ratingAvg?: number;
  ratingCount: number;
}

export interface Review {
  id: string;
  coachHandle: string;
  offeringId: string | null; // null = ad-hoc quote work
  offeringTitle: string;
  clientName: string;
  stars: number;
  outcomeTags: string[];
  body: string;
  daysAgo: number;
}

export interface Coach {
  handle: string;
  displayName: string;
  pronouns?: string;
  headline: string;
  bio: string;
  specialties: Specialty[];
  responseTimeMins: number;
  ratingAvg: number;
  ratingCount: number;
  clientsHelped: number;
  avatarGradient: [string, string];
  verified: boolean;
  badges: string[]; // e.g. "First reply free"
}

export interface DemoUser {
  email: string;
  displayName: string;
  isCoach: boolean;
}

export const SPECIALTY_LABELS: Record<Specialty, string> = {
  openers: "Openers",
  bio: "Bio",
  photos: "Photos",
  conversations: "Convos",
  dates: "Dates",
  custom: "Custom",
};

export function fromPrice(o: Offering): { amountCents: number; suffix: string } {
  switch (o.scheme) {
    case "per_unit":
      return { amountCents: o.unitPriceCents ?? 0, suffix: `per ${o.unitLabel ?? "unit"}` };
    case "flat":
      return { amountCents: o.flatPriceCents ?? 0, suffix: "flat" };
    case "pack":
      return { amountCents: o.packPriceCents ?? 0, suffix: `${o.packUnits} ${o.unitLabel ?? "unit"}s` };
    case "subscription":
      return { amountCents: o.flatPriceCents ?? 0, suffix: "per month" };
    case "quote":
      return { amountCents: 0, suffix: "ask" };
  }
}

export function formatDollars(cents: number): string {
  if (cents % 100 === 0) return `$${cents / 100}`;
  return `$${(cents / 100).toFixed(2)}`;
}
