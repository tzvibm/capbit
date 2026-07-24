// Seed personas — ARCHITECTURE.md §13. In DATA_MODE=mock these back the whole
// app; the Supabase seed.sql mirrors this data.

import type { Coach, Offering, Review, DemoUser } from "./types";

export const coaches: Coach[] = [
  {
    handle: "maya",
    displayName: "Maya R.",
    pronouns: "she/her",
    headline: "I fix dead conversations.",
    bio: "Eight years of turning ghosted threads into second dates. I read the conversation you're stuck in and tell you exactly what to send — and why it works.",
    specialties: ["openers", "conversations"],
    responseTimeMins: 5,
    ratingAvg: 4.9,
    ratingCount: 212,
    clientsHelped: 1408,
    avatarGradient: ["#E4506F", "#B23A6E"],
    verified: true,
    badges: ["First reply free"],
  },
  {
    handle: "dev",
    displayName: "Dev K.",
    headline: "Bio doctor · ex-copywriter",
    bio: "I spent ten years writing ads people couldn't ignore. Now I write bios that get right-swipes from the people you actually want to meet.",
    specialties: ["bio"],
    responseTimeMins: 120,
    ratingAvg: 4.8,
    ratingCount: 167,
    clientsHelped: 640,
    avatarGradient: ["#2F5D62", "#4E8C7F"],
    verified: true,
    badges: [],
  },
  {
    handle: "sam",
    displayName: "Sam T.",
    headline: "Photo lineup strategist",
    bio: "Your photos are 80% of your profile. I tell you which to keep, which to cut, and what to reshoot — bluntly, with reasons.",
    specialties: ["photos"],
    responseTimeMins: 60,
    ratingAvg: 4.7,
    ratingCount: 94,
    clientsHelped: 410,
    avatarGradient: ["#B77410", "#D6A03F"],
    verified: true,
    badges: ["Pack deals"],
  },
  {
    handle: "elena",
    displayName: "Elena V.",
    headline: "Strategy for women — screening & standards",
    bio: "I help women date with a screening mindset: spot low effort early, hold standards without games, and invest only in matches worth your time.",
    specialties: ["conversations", "dates"],
    responseTimeMins: 10,
    ratingAvg: 4.9,
    ratingCount: 318,
    clientsHelped: 1920,
    avatarGradient: ["#2F5D62", "#7FA8A0"],
    verified: true,
    badges: ["Free intro call"],
  },
  {
    handle: "marcus",
    displayName: "Marcus H.",
    headline: "Dating again after divorce · 40+",
    bio: "Divorced at 44, remarried at 49, coached hundreds through the restart. Apps changed while you were away — I'll catch you up without the cringe.",
    specialties: ["bio", "photos"],
    responseTimeMins: 180,
    ratingAvg: 4.8,
    ratingCount: 141,
    clientsHelped: 380,
    avatarGradient: ["#B77410", "#8A5A2B"],
    verified: true,
    badges: [],
  },
  {
    handle: "priya",
    displayName: "Priya N.",
    headline: "Date planner · knows every city",
    bio: "First dates die in loud restaurants. Tell me the city and the person, and I'll plan something they'll talk about — with a rain backup.",
    specialties: ["dates", "custom"],
    responseTimeMins: 240,
    ratingAvg: 5.0,
    ratingCount: 58,
    clientsHelped: 210,
    avatarGradient: ["#E4506F", "#D6A03F"],
    verified: true,
    badges: ["Quote anything"],
  },
  {
    handle: "jo",
    displayName: "Jo A.",
    pronouns: "they/them",
    headline: "Queer dating specialist",
    bio: "Apps and IRL, first messages to first dates. Dating while queer has its own physics — I coach from lived experience, not recycled hetero advice.",
    specialties: ["openers", "conversations", "dates"],
    responseTimeMins: 15,
    ratingAvg: 5.0,
    ratingCount: 86,
    clientsHelped: 340,
    avatarGradient: ["#E4506F", "#7C4DBE"],
    verified: true,
    badges: [],
  },
];

export const offerings: Offering[] = [
  // Maya
  { id: "maya-opener", coachHandle: "maya", title: "Opener rescue", description: "Send me the profile, I write the first message that gets a reply.", specialty: "openers", fulfillment: "chat", scheme: "per_unit", unitLabel: "reply", unitPriceCents: 400, firstUnitFree: true, ratingAvg: 4.9, ratingCount: 120 },
  { id: "maya-pack", coachHandle: "maya", title: "Convo rescue pack", description: "10 replies from me, use anytime a conversation stalls.", specialty: "conversations", fulfillment: "chat", scheme: "pack", unitLabel: "reply", packUnits: 10, packPriceCents: 2500, ratingAvg: 4.9, ratingCount: 64 },
  { id: "maya-openers5", coachHandle: "maya", title: "5 custom openers", description: "Five openers written for a specific match, ranked by risk.", specialty: "openers", fulfillment: "deliverable", scheme: "flat", flatPriceCents: 1800, ratingAvg: 4.8, ratingCount: 41 },
  { id: "maya-call", coachHandle: "maya", title: "Live strategy call", description: "30 minutes on video — bring your worst conversation.", specialty: "conversations", fulfillment: "live", scheme: "flat", flatPriceCents: 5000, sessionMinutes: 30, ratingAvg: 5.0, ratingCount: 28 },
  // Dev
  { id: "dev-bio", coachHandle: "dev", title: "Full bio makeover", description: "Two rewritten versions — one playful, one grounded — plus prompt picks.", specialty: "bio", fulfillment: "deliverable", scheme: "flat", flatPriceCents: 3000, ratingAvg: 4.8, ratingCount: 129 },
  { id: "dev-prompts", coachHandle: "dev", title: "Prompt punch-up", description: "Your three app prompts, rewritten to start conversations.", specialty: "bio", fulfillment: "deliverable", scheme: "flat", flatPriceCents: 1500, ratingAvg: 4.7, ratingCount: 38 },
  // Sam
  { id: "sam-photo", coachHandle: "sam", title: "Photo verdicts", description: "Per photo: keep, cut, or reshoot — with the reason.", specialty: "photos", fulfillment: "chat", scheme: "per_unit", unitLabel: "photo", unitPriceCents: 500, ratingAvg: 4.7, ratingCount: 71 },
  { id: "sam-lineup", coachHandle: "sam", title: "Full lineup review", description: "Up to 8 photos → your best 5, ordered, with swap suggestions.", specialty: "photos", fulfillment: "deliverable", scheme: "flat", flatPriceCents: 2000, ratingAvg: 4.8, ratingCount: 23 },
  // Elena
  { id: "elena-reply", coachHandle: "elena", title: "Screening second opinion", description: "Show me the conversation — I'll tell you if he's worth your time.", specialty: "conversations", fulfillment: "chat", scheme: "per_unit", unitLabel: "reply", unitPriceCents: 600, ratingAvg: 4.9, ratingCount: 204 },
  { id: "elena-call", coachHandle: "elena", title: "Free intro call", description: "15 minutes — see if my approach fits before you spend anything.", specialty: "dates", fulfillment: "live", scheme: "flat", flatPriceCents: 0, sessionMinutes: 15, ratingAvg: 4.9, ratingCount: 88 },
  // Marcus
  { id: "marcus-restart", coachHandle: "marcus", title: "Profile restart", description: "Bio + photo strategy + app choice for dating again after a long break.", specialty: "bio", fulfillment: "deliverable", scheme: "flat", flatPriceCents: 4500, ratingAvg: 4.8, ratingCount: 96 },
  // Priya
  { id: "priya-quote", coachHandle: "priya", title: "Plan me a date", description: "Describe the person and the city; I quote a price and build the plan.", specialty: "dates", fulfillment: "deliverable", scheme: "quote", ratingCount: 0 },
  // Jo
  { id: "jo-reply", coachHandle: "jo", title: "Conversation help", description: "Apps or texting — send the thread, get the next move.", specialty: "conversations", fulfillment: "chat", scheme: "per_unit", unitLabel: "reply", unitPriceCents: 500, ratingAvg: 5.0, ratingCount: 62 },
  { id: "jo-firstdate", coachHandle: "jo", title: "First-date gameplan", description: "Venue ideas, talking points, and an exit plan — tailored, not generic.", specialty: "dates", fulfillment: "deliverable", scheme: "flat", flatPriceCents: 2200, ratingAvg: 5.0, ratingCount: 18 },
];

export const reviews: Review[] = [
  { id: "r1", coachHandle: "maya", offeringId: "maya-pack", offeringTitle: "Convo rescue pack", clientName: "J.", stars: 5, outcomeTags: ["got_reply", "date_set"], body: "Sent her a convo that was dead for a week. Her two lines got a reply in an hour. Date's on Friday.", daysAgo: 2 },
  { id: "r2", coachHandle: "maya", offeringId: "maya-opener", offeringTitle: "Opener rescue", clientName: "T.", stars: 5, outcomeTags: ["got_reply"], body: "The pigeon line worked. I don't know how she does it.", daysAgo: 5 },
  { id: "r3", coachHandle: "maya", offeringId: "maya-call", offeringTitle: "Live strategy call", clientName: "R.", stars: 5, outcomeTags: ["faster"], body: "30 minutes that fixed a year of bad habits. Worth 10x the price.", daysAgo: 9 },
  { id: "r4", coachHandle: "dev", offeringId: "dev-bio", offeringTitle: "Full bio makeover", clientName: "A.", stars: 5, outcomeTags: ["better_profile"], body: "Matches tripled in a week. The pottery hook was genius.", daysAgo: 3 },
  { id: "r5", coachHandle: "dev", offeringId: "dev-bio", offeringTitle: "Full bio makeover", clientName: "M.", stars: 4, outcomeTags: ["better_profile"], body: "Both versions were strong. Wanted one more revision round, but the result speaks for itself.", daysAgo: 12 },
  { id: "r6", coachHandle: "sam", offeringId: "sam-photo", offeringTitle: "Photo verdicts", clientName: "K.", stars: 5, outcomeTags: ["better_profile"], body: "Brutal and correct. Cut my gym selfie, matches went up.", daysAgo: 4 },
  { id: "r7", coachHandle: "elena", offeringId: "elena-reply", offeringTitle: "Screening second opinion", clientName: "S.", stars: 5, outcomeTags: ["faster"], body: "She spotted the breadcrumbing in one screenshot. Saved me a month.", daysAgo: 1 },
  { id: "r8", coachHandle: "marcus", offeringId: "marcus-restart", offeringTitle: "Profile restart", clientName: "D.", stars: 5, outcomeTags: ["better_profile", "date_set"], body: "First date in six years, two weeks after his rebuild. Felt like myself, not a costume.", daysAgo: 7 },
  { id: "r9", coachHandle: "priya", offeringId: null, offeringTitle: "Custom date plan", clientName: "L.", stars: 5, outcomeTags: ["date_set"], body: "The museum-then-pinball plan was perfect. Second date already booked.", daysAgo: 6 },
  { id: "r10", coachHandle: "jo", offeringId: "jo-reply", offeringTitle: "Conversation help", clientName: "C.", stars: 5, outcomeTags: ["got_reply"], body: "Finally advice that gets how queer dating apps actually work.", daysAgo: 2 },
];

export const demoUsers: DemoUser[] = [
  { email: "jordan@demo.wing", displayName: "Jordan", isCoach: false },
  { email: "alex@demo.wing", displayName: "Alex", isCoach: false },
  { email: "riley@demo.wing", displayName: "Riley", isCoach: false },
  { email: "maya@demo.wing", displayName: "Maya R.", isCoach: true },
];
