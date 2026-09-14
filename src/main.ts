// ai-generated
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { renderChrome } from "./chrome";
import "./chrome.css";
import type { ChromeHandlers, PersistNotice, SessionSnapshot } from "./types";

const el = document.getElementById("chrome-root");
if (!el) {
  throw new Error("chrome-root missing");
}
const root = el;

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
    const el = root.querySelector("[data-test=toast]");
    if (el) {
      el.textContent = "";
    }
  }, 4000);
}

function paint(snapshot: SessionSnapshot): void {
  renderChrome(root, snapshot, handlers);
  if (toastText) {
    const toast = root.querySelector("[data-test=toast]");
    if (toast) {
      toast.textContent = toastText;
    }
  }
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
  onOpenLibrary: () => {},
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
