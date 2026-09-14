// ai-generated
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { renderChrome } from "./chrome";
import { renderLibrary, type LibraryData } from "./library";
import "./chrome.css";
import type {
  BookmarkRow,
  ChromeHandlers,
  PersistNotice,
  SessionSnapshot,
  VisitRow,
} from "./types";

const el = document.getElementById("chrome-root");
if (!el) {
  throw new Error("chrome-root missing");
}
const root = el;

let lastSnap: SessionSnapshot | null = null;
let toastText = "";
let toastTimer: number | undefined;

function showToast(message: string): void {
  toastText = message;
  const toast = root.querySelector("[data-test=toast]");
  if (toast) {
    toast.textContent = message;
  }
  window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => {
    toastText = "";
    const node = root.querySelector("[data-test=toast]");
    if (node) {
      node.textContent = "";
    }
  }, 4000);
}

function paint(snapshot: SessionSnapshot): void {
  lastSnap = snapshot;
  renderChrome(root, snapshot, handlers);
  if (toastText) {
    const toast = root.querySelector("[data-test=toast]");
    if (toast) {
      toast.textContent = toastText;
    }
  }
}

async function fillLibrary(data: LibraryData): Promise<void> {
  const panel = root.querySelector<HTMLElement>("[data-test=library-panel]");
  if (!panel) {
    return;
  }
  renderLibrary(
    panel,
    data,
    (url) => {
      const tab_id = lastSnap?.active_tab_id;
      if (tab_id) {
        void invoke("navigate", { tab_id, input: url });
      }
      panel.setAttribute("hidden", "");
    },
    async (query) => {
      const trimmed = query.trim();
      const visits = trimmed
        ? await invoke<VisitRow[]>("search_history", { query: trimmed })
        : await invoke<VisitRow[]>("list_history", { limit: 100 });
      const bookmarks = await invoke<BookmarkRow[]>("list_bookmarks");
      await fillLibrary({ bookmarks, visits });
    },
  );
}

async function openLibrary(): Promise<void> {
  const bookmarks = await invoke<BookmarkRow[]>("list_bookmarks");
  const visits = await invoke<VisitRow[]>("list_history", { limit: 100 });
  await fillLibrary({ bookmarks, visits });
}

const handlers: ChromeHandlers = {
  onNavigate: (tab_id, input) => {
    void invoke("navigate", { tab_id, input });
  },
  onNewTab: () => {
    void invoke("new_tab");
  },
  onCloseTab: (tab_id) => {
    void invoke("close_tab", { tab_id });
  },
  onSwitchTab: (tab_id) => {
    void invoke("switch_tab", { tab_id });
  },
  onBack: (tab_id) => {
    void invoke("go_back", { tab_id });
  },
  onForward: (tab_id) => {
    void invoke("go_forward", { tab_id });
  },
  onReload: (tab_id) => {
    void invoke("reload", { tab_id });
  },
  onToggleBookmark: (tab_id) => {
    void invoke("toggle_bookmark", { tab_id });
  },
  onOpenLibrary: () => {
    void openLibrary();
  },
};

const snap = await invoke<SessionSnapshot>("get_snapshot");
paint(snap);

await listen<SessionSnapshot>("session_changed", (event) => {
  paint(event.payload);
});
await listen<PersistNotice>("persist_notice", (event) => {
  showToast(event.payload.message);
});
// ai-generated end
