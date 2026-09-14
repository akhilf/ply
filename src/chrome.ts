// ai-generated
import { LOAD_ERROR_COPY, MEMORY_NOTICE } from "./notices";
import type { ChromeHandlers, SessionSnapshot } from "./types";

export function renderChrome(
  root: HTMLElement,
  snapshot: SessionSnapshot,
  handlers: ChromeHandlers,
): void {
  root.replaceChildren();
  root.classList.add("ply-chrome");

  const memory = document.createElement("div");
  memory.setAttribute("data-test", "memory-notice");
  memory.textContent = MEMORY_NOTICE;
  if (snapshot.persist_mode !== "memory") {
    memory.setAttribute("hidden", "");
  }
  root.append(memory);

  const strip = document.createElement("div");
  strip.id = "tab-strip";
  strip.className = "tab-strip";

  const brand = document.createElement("div");
  brand.className = "ply-brand";
  brand.setAttribute("data-test", "brand");
  brand.setAttribute("aria-label", "Ply");
  const mark = document.createElement("span");
  mark.className = "ply-mark";
  mark.setAttribute("aria-hidden", "true");
  const backLayer = document.createElement("span");
  backLayer.className = "ply-mark-layer ply-mark-layer--back";
  const frontLayer = document.createElement("span");
  frontLayer.className = "ply-mark-layer ply-mark-layer--front";
  mark.append(backLayer, frontLayer);
  const word = document.createElement("span");
  word.className = "ply-wordmark";
  word.textContent = "Ply";
  brand.append(mark, word);
  strip.append(brand);

  for (const tab of snapshot.tabs) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "tab";
    btn.setAttribute("data-test", `tab-${tab.id}`);
    btn.setAttribute("aria-selected", String(tab.id === snapshot.active_tab_id));
    const label = document.createElement("span");
    label.className = "tab-title";
    label.textContent = tab.title || tab.url;
    btn.append(label);
    btn.addEventListener("click", () => handlers.onSwitchTab(tab.id));
    const close = document.createElement("button");
    close.type = "button";
    close.className = "tab-close";
    close.setAttribute("data-test", `close-${tab.id}`);
    close.setAttribute("aria-label", "Close tab");
    close.textContent = "×";
    close.addEventListener("click", (event) => {
      event.stopPropagation();
      handlers.onCloseTab(tab.id);
    });
    btn.append(close);
    strip.append(btn);
  }
  const newTab = document.createElement("button");
  newTab.type = "button";
  newTab.className = "new-tab";
  newTab.setAttribute("data-test", "new-tab");
  newTab.setAttribute("aria-label", "New tab");
  newTab.textContent = "+";
  newTab.addEventListener("click", () => handlers.onNewTab());
  strip.append(newTab);
  root.append(strip);

  const toolbar = document.createElement("div");
  toolbar.className = "toolbar";

  const nav = document.createElement("div");
  nav.className = "nav-cluster";
  const back = navButton("back", "Back", !snapshot.can_go_back, () =>
    handlers.onBack(snapshot.active_tab_id),
    ICON_BACK,
  );
  const forward = navButton("forward", "Forward", !snapshot.can_go_forward, () =>
    handlers.onForward(snapshot.active_tab_id),
    ICON_FORWARD,
  );
  const reload = navButton("reload", "Reload", false, () =>
    handlers.onReload(snapshot.active_tab_id),
    ICON_RELOAD,
  );
  nav.append(back, forward, reload);
  toolbar.append(nav);

  const form = document.createElement("form");
  form.className = "url-form";
  const url = document.createElement("input");
  url.type = "text";
  url.setAttribute("data-test", "url");
  url.value = snapshot.current_url;
  url.spellcheck = false;
  form.append(url);
  form.addEventListener("submit", (event) => {
    event.preventDefault();
    if (url.value.trim() === "") {
      return;
    }
    handlers.onNavigate(snapshot.active_tab_id, url.value);
  });
  toolbar.append(form);

  const bookmark = document.createElement("button");
  bookmark.type = "button";
  bookmark.className = "icon-btn";
  bookmark.setAttribute("data-test", "bookmark");
  bookmark.setAttribute("aria-label", snapshot.is_bookmarked ? "Remove bookmark" : "Bookmark this page");
  bookmark.textContent = snapshot.is_bookmarked ? "★" : "☆";
  bookmark.addEventListener("click", () =>
    handlers.onToggleBookmark(snapshot.active_tab_id),
  );
  toolbar.append(bookmark);

  const library = document.createElement("button");
  library.type = "button";
  library.className = "library-btn";
  library.setAttribute("data-test", "library");
  library.textContent = "Library";
  library.addEventListener("click", () => handlers.onOpenLibrary());
  toolbar.append(library);

  const error = document.createElement("div");
  error.setAttribute("data-test", "load-error");
  if (snapshot.load_error) {
    error.textContent = LOAD_ERROR_COPY[snapshot.load_error];
  } else {
    error.setAttribute("hidden", "");
  }
  toolbar.append(error);
  root.append(toolbar);

  const toast = document.createElement("div");
  toast.setAttribute("data-test", "toast");
  root.append(toast);

  const panel = document.createElement("div");
  panel.setAttribute("data-test", "library-panel");
  panel.setAttribute("hidden", "");
  root.append(panel);
}

const ICON_BACK =
  '<svg viewBox="0 0 16 16" aria-hidden="true"><path fill="currentColor" d="M10.2 2.7 4.9 8l5.3 5.3 1.1-1.1L7.1 8l4.2-4.2z"/></svg>';
const ICON_FORWARD =
  '<svg viewBox="0 0 16 16" aria-hidden="true"><path fill="currentColor" d="M5.8 2.7 4.7 3.8 8.9 8l-4.2 4.2 1.1 1.1L11.1 8z"/></svg>';
const ICON_RELOAD =
  '<svg viewBox="0 0 16 16" aria-hidden="true"><path fill="currentColor" d="M8 2.2a5.8 5.8 0 1 0 5.5 4h-1.6A4.2 4.2 0 1 1 8 3.8V6l3-2.2L8 1.6z"/></svg>';

function navButton(
  test: string,
  label: string,
  disabled: boolean,
  onClick: () => void,
  icon: string,
): HTMLButtonElement {
  const btn = document.createElement("button");
  btn.type = "button";
  btn.className = "nav-btn";
  btn.setAttribute("data-test", test);
  btn.setAttribute("aria-label", label);
  btn.title = label;
  btn.innerHTML = icon;
  btn.disabled = disabled;
  btn.addEventListener("click", onClick);
  return btn;
}
// ai-generated end
