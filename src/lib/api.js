// Thin wrapper around Tauri commands. When running outside Tauri (plain
// `npm run dev` in a browser) it falls back to in-memory sample data so the
// UI is still fully browsable during frontend work.

import { SAMPLE_PEOPLE, SAMPLE_TASKS } from "./sample.js";

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

// Lazy-load Tauri's invoke to avoid top-level await (incompatible with older build targets)
let _tauriInvoke = null;
async function invoke(cmd, args) {
  if (!inTauri) throw new Error("Tauri not available");
  if (!_tauriInvoke) {
    const mod = await import("@tauri-apps/api/core");
    _tauriInvoke = mod.invoke;
  }
  return _tauriInvoke(cmd, args);
}

// --- browser fallback state ---
let mockPeople = structuredClone(SAMPLE_PEOPLE);
let mockTasks = structuredClone(SAMPLE_TASKS);
let mockAppSettings = {
  auto_archive_done: true,
  auto_archive_days: 7,
  stale_1on1_days: 14,
};

export async function listPeople() {
  if (!inTauri) return structuredClone(mockPeople);
  return invoke("list_people");
}

export async function getPerson(slug) {
  if (!inTauri) return structuredClone(mockPeople.find((p) => p.slug === slug));
  return invoke("get_person", { slug });
}

export async function addConversation(slug, conv) {
  if (!inTauri) {
    const p = mockPeople.find((x) => x.slug === slug);
    p.conversations.unshift(conv);
    p.last_met = conv.date;
    return structuredClone(p);
  }
  return invoke("add_conversation", { slug, conv });
}

function normalizeTasks(raw) {
  return raw.map((t) => ({
    ...t,
    due_time: t.due_time || "",
    people: Array.isArray(t.people)
      ? t.people
      : t.person && t.person !== "—"
      ? [t.person]
      : [],
  }));
}

export async function listTasks() {
  if (!inTauri) return normalizeTasks(structuredClone(mockTasks));
  return normalizeTasks(await invoke("list_tasks"));
}

export async function saveTasks(tasks) {
  if (!inTauri) {
    mockTasks = structuredClone(tasks).map((task) => {
      if (
        mockAppSettings.auto_archive_done &&
        task.column === "done" &&
        !task.archived &&
        task.completed_at
      ) {
        const done = new Date(`${task.completed_at}T00:00:00`);
        const days = Math.round((Date.now() - done.getTime()) / 86400000);
        if (days >= Math.max(1, mockAppSettings.auto_archive_days)) {
          return { ...task, archived: true };
        }
      }
      return task;
    });
    return;
  }
  return invoke("save_tasks", { tasks });
}

export async function getVaultPath() {
  if (!inTauri) return "(browser preview — sample data)";
  return invoke("get_vault_path");
}

export async function setVaultPath(path) {
  if (!inTauri) return;
  return invoke("set_vault_path", { path });
}

export async function pickVault() {
  if (!inTauri) return null;
  return invoke("pick_vault");
}

export async function createVaultBackup() {
  if (!inTauri) {
    return {
      path: "(browser preview — backup unavailable)",
    };
  }
  return invoke("create_vault_backup");
}

export async function getAppSettings() {
  if (!inTauri) return structuredClone(mockAppSettings);
  return invoke("get_app_settings");
}

export async function saveAppSettings(settings) {
  if (!inTauri) {
    mockAppSettings = structuredClone(settings);
    return structuredClone(mockAppSettings);
  }
  return invoke("save_app_settings", { settings });
}

export async function createPerson({ name, role, color, group }) {
  if (!inTauri) {
    const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
    const p = { slug, name, role: role || "", bio: "", color: color || "#6b7d9c", group: group || "", last_met: null, conversations: [] };
    mockPeople.push(p);
    return structuredClone(p);
  }
  return invoke("create_person", { name, role, color, group: group || "" });
}

export async function deletePerson(slug) {
  if (!inTauri) {
    mockPeople = mockPeople.filter((p) => p.slug !== slug);
    return;
  }
  return invoke("delete_person", { slug });
}

export async function updatePerson(slug, { name, role, bio, color, group, birthday = "" }) {
  if (!inTauri) {
    const idx = mockPeople.findIndex((p) => p.slug === slug);
    if (idx >= 0) {
      mockPeople[idx] = { ...mockPeople[idx], name, role, bio, color, group: group || "", birthday: birthday || "" };
    }
    return structuredClone(mockPeople[idx]);
  }
  return invoke("update_person", { slug, name, role, bio, color, group: group || "", birthday: birthday || "" });
}

export async function deleteConversation(slug, date, title) {
  if (!inTauri) {
    const p = mockPeople.find((x) => x.slug === slug);
    if (p) p.conversations = p.conversations.filter((c) => !(c.date === date && c.title === title));
    return;
  }
  return invoke("delete_conversation", { slug, date, title });
}

export async function updateConversation(slug, origDate, origTitle, conv) {
  if (!inTauri) {
    const p = mockPeople.find((x) => x.slug === slug);
    if (p) {
      const idx = p.conversations.findIndex((c) => c.date === origDate && c.title === origTitle);
      if (idx >= 0) p.conversations[idx] = conv;
      if (idx === 0) p.last_met = conv.date;
    }
    return structuredClone(p);
  }
  return invoke("update_conversation", { slug, origDate, origTitle, conv });
}

export async function listFolders() {
  if (!inTauri) return [];
  return invoke("list_folders");
}

export async function createFolder(name, color, excludeCheckin = false) {
  if (!inTauri) return;
  return invoke("create_folder", { name, color, excludeCheckin });
}

export async function updateFolder(name, nextName, color, excludeCheckin = false) {
  if (!inTauri) return;
  return invoke("update_folder", { name, nextName, color, excludeCheckin });
}

export async function reorderFolders(names) {
  if (!inTauri) return;
  return invoke("reorder_folders", { names });
}

export async function deleteFolder(name) {
  if (!inTauri) return;
  return invoke("delete_folder", { name });
}

export async function getUpdaterStatus() {
  if (!inTauri) {
    return {
      currentVersion: "browser-preview",
      configured: false,
      endpoint: null,
    };
  }
  return invoke("get_updater_status");
}

export async function checkForUpdate() {
  if (!inTauri) return null;
  return invoke("check_for_update");
}

export async function installUpdate() {
  if (!inTauri) return;
  return invoke("install_update");
}

export async function listenToMenuActions(handler) {
  if (!inTauri) return () => {};
  const mod = await import("@tauri-apps/api/event");
  return mod.listen("menu-action", (event) => handler(event.payload));
}
