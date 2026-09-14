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
  for (const tab of snapshot.tabs) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.setAttribute("data-test", `tab-${tab.id}`);
    btn.setAttribute("aria-selected", String(tab.id === snapshot.active_tab_id));
    const label = document.createElement("span");
    label.textContent = tab.title || tab.url;
    btn.append(label);
    btn.addEventListener("click", () => handlers.onSwitchTab(tab.id));
    const close = document.createElement("button");
    close.type = "button";
    close.setAttribute("data-test", `close-${tab.id}`);
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
  newTab.setAttribute("data-test", "new-tab");
  newTab.textContent = "+";
  newTab.addEventListener("click", () => handlers.onNewTab());
  strip.append(newTab);
  root.append(strip);

  const toolbar = document.createElement("div");
  toolbar.className = "toolbar";

  const back = navButton("back", "Back", !snapshot.can_go_back, () =>
    handlers.onBack(snapshot.active_tab_id),
  );
  const forward = navButton("forward", "Forward", !snapshot.can_go_forward, () =>
    handlers.onForward(snapshot.active_tab_id),
  );
  const reload = navButton("reload", "Reload", false, () =>
    handlers.onReload(snapshot.active_tab_id),
  );
  toolbar.append(back, forward, reload);

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
  bookmark.setAttribute("data-test", "bookmark");
  bookmark.textContent = snapshot.is_bookmarked ? "★" : "☆";
  bookmark.addEventListener("click", () =>
    handlers.onToggleBookmark(snapshot.active_tab_id),
  );
  toolbar.append(bookmark);

  const library = document.createElement("button");
  library.type = "button";
  library.setAttribute("data-test", "library");
  library.textContent = "Library";
  library.addEventListener("click", () => handlers.onOpenLibrary());
  toolbar.append(library);
  root.append(toolbar);

  const error = document.createElement("div");
  error.setAttribute("data-test", "load-error");
  if (snapshot.load_error) {
    error.textContent = LOAD_ERROR_COPY[snapshot.load_error];
  } else {
    error.setAttribute("hidden", "");
  }
  root.append(error);

  const toast = document.createElement("div");
  toast.setAttribute("data-test", "toast");
  root.append(toast);

  const panel = document.createElement("div");
  panel.setAttribute("data-test", "library-panel");
  panel.setAttribute("hidden", "");
  root.append(panel);
}

function navButton(
  test: string,
  label: string,
  disabled: boolean,
  onClick: () => void,
): HTMLButtonElement {
  const btn = document.createElement("button");
  btn.type = "button";
  btn.setAttribute("data-test", test);
  btn.textContent = label;
  btn.disabled = disabled;
  btn.addEventListener("click", onClick);
  return btn;
}
// ai-generated end
