import { writable } from "svelte/store";

// "dashboard" | "people" | "person" | "tasks" | "conversations" | "today" | "settings"
export const screen = writable("dashboard");
export const sidebarCollapsed = writable(false);
export const spellCheck = writable(true);

export function toggleSidebar() {
  sidebarCollapsed.update((v) => {
    const next = !v;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("sideeye.sidebarCollapsed", next ? "1" : "0");
    }
    return next;
  });
}

export function expandSidebar() {
  sidebarCollapsed.set(false);
  if (typeof localStorage !== "undefined") {
    localStorage.setItem("sideeye.sidebarCollapsed", "0");
  }
}
// slug of the open person, or null
export const selectedSlug = writable(null);

// title of the task to highlight when navigating to tasks screen
export const selectedTaskTitle = writable("");

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

export const GROUP_COLORS = [
  "#6b7d9c",
  "#a86b5a",
  "#8d6480",
  "#a8824f",
  "#5d8a8a",
  "#7a8b5a",
  "#6f8f72",
  "#9c7b6b",
];

export function colorForPerson(person, folderList = []) {
  const groupColor = folderList.find((folder) => folder.name === person?.group)?.color;
  return groupColor || person?.color || GROUP_COLORS[0];
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

function parseDateValue(value) {
  if (!value) return null;
  const trimmed = String(value).trim();
  if (!trimmed) return null;

  if (/^\d{4}-\d{2}-\d{2}$/.test(trimmed)) {
    const parsed = new Date(`${trimmed}T00:00:00`);
    return Number.isNaN(parsed.getTime()) ? null : parsed;
  }

  if (/^\d{4}-\d{2}-\d{2}[ T]\d{2}:\d{2}(:\d{2})?$/.test(trimmed)) {
    const normalized = trimmed.includes("T") ? trimmed : trimmed.replace(" ", "T");
    const parsed = new Date(normalized.length === 16 ? `${normalized}:00` : normalized);
    return Number.isNaN(parsed.getTime()) ? null : parsed;
  }

  const parsed = new Date(trimmed);
  return Number.isNaN(parsed.getTime()) ? null : parsed;
}

function hasExplicitTime(value) {
  return /\d{2}:\d{2}/.test(String(value || ""));
}

function startOfDay(date) {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

export function formatDate(iso) {
  if (!iso) return "";
  const d = parseDateValue(iso);
  if (!d) return iso;

  const dateLabel = d.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });

  if (!hasExplicitTime(iso)) return dateLabel;

  const timeLabel = d.toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
  });

  return `${dateLabel} at ${timeLabel}`;
}

// relative label like "Today", "2d ago", "3 weeks ago"
export function relative(iso) {
  if (!iso) return "never";
  const d = parseDateValue(iso);
  if (!d) return iso;
  const days = Math.round((startOfDay(new Date()) - startOfDay(d)) / 86400000);
  if (days <= 0) return "Today";
  if (days === 1) return "Yesterday";
  if (days < 7) return `${days} days ago`;
  if (days < 30) return `${Math.round(days / 7)} weeks ago`;
  return formatDate(iso);
}

export function daysSince(iso) {
  if (!iso) return Number.POSITIVE_INFINITY;
  const d = parseDateValue(iso);
  if (!d) return Number.POSITIVE_INFINITY;
  return Math.round((startOfDay(new Date()) - startOfDay(d)) / 86400000);
}

export function dayLabel(iso) {
  const days = daysSince(iso);
  if (!Number.isFinite(days)) return iso;
  if (days <= 0) return "Today";
  if (days === 1) return "Yesterday";
  if (days < 7) return `${days} days ago`;
  const d = parseDateValue(iso);
  if (!d) return iso;
  return d.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
  });
}
