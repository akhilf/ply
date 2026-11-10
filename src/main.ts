// ai-generated
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { renderChrome } from "./chrome";
import { renderDownloads, type DownloadRow } from "./downloads";
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
let downloadsOpen = false;
let downloads: DownloadRow[] = [];
let findOpen = false;
let findQuery = "";
let findShouldFocus = false;
let findTimer: number | undefined;
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
  const downloadSaving = downloads.filter((item) => item.status === "saving").length;
  renderChrome(root, snapshot, handlers, {
    downloadSaving,
    findOpen,
    findQuery,
  });
  if (toastText) {
    const toast = root.querySelector("[data-test=toast]");
    if (toast) {
      toast.textContent = toastText;
    }
  }
  if (libraryOpen && libraryData) {
    void fillLibrary(libraryData);
  }
  if (downloadsOpen) {
    fillDownloads();
  }
  if (findOpen && findShouldFocus) {
    findShouldFocus = false;
    root.querySelector<HTMLInputElement>("[data-test=find-query]")?.focus();
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
  downloadsOpen = false;
  libraryOpen = true;
  root.querySelector<HTMLElement>("[data-test=downloads-panel]")?.setAttribute("hidden", "");
  await invoke("set_library_open", { open: true });
  const bookmarks = await invoke<BookmarkRow[]>("list_bookmarks");
  const visits = await invoke<VisitRow[]>("list_history", { limit: 100 });
  await fillLibrary({ bookmarks, visits });
}

async function closeDownloads(): Promise<void> {
  downloadsOpen = false;
  const panel = root.querySelector<HTMLElement>("[data-test=downloads-panel]");
  panel?.setAttribute("hidden", "");
  await invoke("set_downloads_open", { open: false });
}

function fillDownloads(): void {
  const panel = root.querySelector<HTMLElement>("[data-test=downloads-panel]");
  if (!panel) {
    return;
  }
  renderDownloads(panel, downloads, {
    onOpen: (id) => {
      void commandVoid("open_download", { id });
    },
    onReveal: (id) => {
      void commandVoid("reveal_download", { id });
    },
    onClear: () => {
      void (async () => {
        downloads = await invoke<DownloadRow[]>("clear_downloads");
        if (lastSnap) {
          paint(lastSnap);
        }
      })();
    },
    onClose: () => {
      void closeDownloads();
    },
  });
}

async function openDownloads(): Promise<void> {
  libraryOpen = false;
  downloadsOpen = true;
  root.querySelector<HTMLElement>("[data-test=library-panel]")?.setAttribute("hidden", "");
  await invoke("set_downloads_open", { open: true });
  downloads = await invoke<DownloadRow[]>("list_downloads");
  fillDownloads();
}

function rememberDownloads(items: DownloadRow[]): void {
  for (const item of items) {
    const prev = downloads.find((row) => row.id === item.id);
    if (!prev && item.status === "saving") {
      showToast(`Saving ${item.filename}`);
    } else if (prev?.status === "saving" && item.status === "done") {
      showToast(`Saved ${item.filename}`);
    } else if (prev?.status === "saving" && item.status === "failed") {
      showToast(`Couldn't save ${item.filename}`);
    }
  }
  downloads = items;
  if (lastSnap) {
    paint(lastSnap);
  }
}

async function command(name: string, args?: Record<string, unknown>): Promise<void> {
  try {
    const snap = await invoke<SessionSnapshot>(name, args);
    paint(snap);
  } catch (err) {
    showToast(err instanceof Error ? err.message : String(err));
  }
}

async function commandVoid(name: string, args?: Record<string, unknown>): Promise<void> {
  try {
    await invoke(name, args);
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
  onOpenDownloads: () => {
    void openDownloads();
  },
  onOpenFind: () => {
    void openFind();
  },
  onFindQuery: (query) => {
    findQuery = query;
    window.clearTimeout(findTimer);
    findTimer = window.setTimeout(() => runFind(false), 160);
  },
  onFindNext: () => {
    runFind(false);
  },
  onFindPrev: () => {
    runFind(true);
  },
  onCloseFind: () => {
    void closeFind();
  },
};

async function openFind(): Promise<void> {
  findOpen = true;
  findShouldFocus = true;
  await invoke("set_find_open", { open: true });
  if (lastSnap) {
    paint(lastSnap);
  }
}

async function closeFind(): Promise<void> {
  findOpen = false;
  const tabId = lastSnap?.active_tab_id;
  if (tabId) {
    await invoke("find_in_page", { tabId, query: "", backwards: false });
  }
  await invoke("set_find_open", { open: false });
  if (lastSnap) {
    paint(lastSnap);
  }
}

function runFind(backwards: boolean): void {
  const tabId = lastSnap?.active_tab_id;
  const query = findQuery.trim();
  if (!tabId || query === "") {
    return;
  }
  void invoke("find_in_page", { tabId, query, backwards });
}

const snap = await invoke<SessionSnapshot>("get_snapshot");
paint(snap);

await listen<SessionSnapshot>("session_changed", (event) => {
  paint(event.payload);
});
await listen<PersistNotice>("persist_notice", (event) => {
  showToast(event.payload.message);
});
await listen<DownloadRow[]>("downloads_changed", (event) => {
  rememberDownloads(event.payload);
});

window.addEventListener("keydown", (event) => {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "f") {
    event.preventDefault();
    void openFind();
  }
  if (event.key === "Escape" && findOpen) {
    event.preventDefault();
    void closeFind();
  }
});
// ai-generated end
