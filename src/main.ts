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
      const tab_id = lastSnap?.active_tab_id;
      void closeLibrary();
      if (tab_id) {
        void invoke("navigate", { tab_id, input: url });
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
  onNavigate: (tab_id, input) => {
    void command("navigate", { tab_id, input });
  },
  onNewTab: () => {
    void command("new_tab");
  },
  onCloseTab: (tab_id) => {
    void command("close_tab", { tab_id });
  },
  onSwitchTab: (tab_id) => {
    void command("switch_tab", { tab_id });
  },
  onBack: (tab_id) => {
    void command("go_back", { tab_id });
  },
  onForward: (tab_id) => {
    void command("go_forward", { tab_id });
  },
  onReload: (tab_id) => {
    void command("reload", { tab_id });
  },
  onToggleBookmark: (tab_id) => {
    void command("toggle_bookmark", { tab_id });
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
