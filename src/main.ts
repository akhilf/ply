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
let libraryOpen = false;
let libraryData: LibraryData | null = null;
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
  if (libraryOpen && libraryData) {
    void fillLibrary(libraryData);
  }
}

async function closeLibrary(): Promise<void> {
  libraryOpen = false;
  const panel = root.querySelector<HTMLElement>("[data-test=library-panel]");
  panel?.setAttribute("hidden", "");
  await invoke("set_library_open", { open: false });
}

async function fillLibrary(data: LibraryData): Promise<void> {
  libraryData = data;
  const panel = root.querySelector<HTMLElement>("[data-test=library-panel]");
  if (!panel) {
    return;
  }
  renderLibrary(
    panel,
    data,
    (url) => {
      const tabId = lastSnap?.active_tab_id;
      void closeLibrary();
      if (tabId) {
        void command("navigate", { tabId, input: url });
      }
    },
    async (query) => {
      const trimmed = query.trim();
      const visits = trimmed
        ? await invoke<VisitRow[]>("search_history", { query: trimmed })
        : await invoke<VisitRow[]>("list_history", { limit: 100 });
      const bookmarks = await invoke<BookmarkRow[]>("list_bookmarks");
      await fillLibrary({ bookmarks, visits });
    },
    () => {
      void closeLibrary();
    },
  );
}

async function openLibrary(): Promise<void> {
  libraryOpen = true;
  await invoke("set_library_open", { open: true });
  const bookmarks = await invoke<BookmarkRow[]>("list_bookmarks");
  const visits = await invoke<VisitRow[]>("list_history", { limit: 100 });
  await fillLibrary({ bookmarks, visits });
}

async function command(name: string, args?: Record<string, unknown>): Promise<void> {
  try {
    const snap = await invoke<SessionSnapshot>(name, args);
    paint(snap);
  } catch (err) {
    showToast(err instanceof Error ? err.message : String(err));
  }
}

const handlers: ChromeHandlers = {
  onNavigate: (tabId, input) => {
    void command("navigate", { tabId, input });
  },
  onNewTab: () => {
    void command("new_tab");
  },
  onCloseTab: (tabId) => {
    void command("close_tab", { tabId });
  },
  onSwitchTab: (tabId) => {
    void command("switch_tab", { tabId });
  },
  onBack: (tabId) => {
    void command("go_back", { tabId });
  },
  onForward: (tabId) => {
    void command("go_forward", { tabId });
  },
  onReload: (tabId) => {
    void command("reload", { tabId });
  },
  onToggleBookmark: (tabId) => {
    void command("toggle_bookmark", { tabId });
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
