import { writable } from "svelte/store";

// "people" | "person" | "tasks" | "conversations" | "today" | "settings"
export const screen = writable("people");
// slug of the open person, or null
export const selectedSlug = writable(null);

export const people = writable([]);
export const tasks = writable([]);
export const folders = writable([]);
export const vaultPath = writable("");
export const appAction = writable({ type: "", detail: null, token: 0 });
export const conversationSearchQuery = writable("");
export const conversationsViewQuery = writable("");
export const conversationsViewPage = writable(1);
export const targetConversation = writable(null);

export function fireAppAction(type, detail = null) {
  appAction.set({
    type,
    detail,
    token: Date.now() + Math.random(),
  });
}

export function clearAppAction() {
  appAction.set({ type: "", detail: null, token: 0 });
}

const PRIORITY = {
  high: "var(--over)",
  med: "var(--due)",
  low: "#8a9b8c",
};

export function priorityColor(p) {
  return PRIORITY[p] || "#8a9b8c";
}

export function initials(name) {
  if (!name || name === "—") return "·";
  return name
    .split(" ")
    .map((w) => w[0])
    .slice(0, 2)
    .join("")
    .toUpperCase();
}

// "2026-06-16" -> "Jun 16, 2026"
export function formatDate(iso) {
  if (!iso) return "";
  const d = new Date(iso + "T00:00:00");
  if (isNaN(d)) return iso;
  return d.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
}

// relative label like "Today", "2d ago", "3 weeks ago"
export function relative(iso) {
  if (!iso) return "never";
  const d = new Date(iso + "T00:00:00");
  if (isNaN(d)) return iso;
  const days = Math.round((Date.now() - d.getTime()) / 86400000);
  if (days <= 0) return "Today";
  if (days === 1) return "Yesterday";
  if (days < 7) return `${days} days ago`;
  if (days < 30) return `${Math.round(days / 7)} weeks ago`;
  return formatDate(iso);
}

export function daysSince(iso) {
  if (!iso) return Number.POSITIVE_INFINITY;
  const d = new Date(iso + "T00:00:00");
  if (isNaN(d)) return Number.POSITIVE_INFINITY;
  return Math.round((Date.now() - d.getTime()) / 86400000);
}

export function dayLabel(iso) {
  const days = daysSince(iso);
  if (!Number.isFinite(days)) return iso;
  if (days <= 0) return "Today";
  if (days === 1) return "Yesterday";
  if (days < 7) return `${days} days ago`;
  return new Date(iso + "T00:00:00").toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
  });
}
