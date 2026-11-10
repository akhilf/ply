// ai-generated
export type DownloadRow = {
  id: number;
  url: string;
  filename: string;
  path: string;
  status: "saving" | "done" | "failed";
  started_at: number;
};

export type DownloadHandlers = {
  onOpen: (id: number) => void;
  onReveal: (id: number) => void;
  onClear: () => void;
  onClose: () => void;
};

export function renderDownloads(
  root: HTMLElement,
  items: DownloadRow[],
  handlers: DownloadHandlers,
): void {
  root.replaceChildren();
  root.removeAttribute("hidden");
  root.classList.add("overlay-panel");

  const head = document.createElement("div");
  head.className = "library-head";
  const title = document.createElement("strong");
  title.textContent = "Downloads";
  const actions = document.createElement("div");
  actions.className = "overlay-actions";
  const clear = document.createElement("button");
  clear.type = "button";
  clear.setAttribute("data-test", "downloads-clear");
  clear.textContent = "Clear";
  clear.addEventListener("click", () => handlers.onClear());
  const close = document.createElement("button");
  close.type = "button";
  close.setAttribute("data-test", "downloads-close");
  close.textContent = "Close";
  close.addEventListener("click", () => {
    root.setAttribute("hidden", "");
    handlers.onClose();
  });
  actions.append(clear, close);
  head.append(title, actions);
  root.append(head);

  if (items.length === 0) {
    const empty = document.createElement("p");
    empty.setAttribute("data-test", "downloads-empty");
    empty.textContent = "Files you save from a page will show up here.";
    root.append(empty);
    return;
  }

  for (const item of items) {
    const row = document.createElement("div");
    row.className = "download-row";
    row.setAttribute("data-test", "download-row");
    row.setAttribute("data-id", String(item.id));
    const name = document.createElement("div");
    name.className = "download-name";
    name.textContent = item.filename;
    const status = document.createElement("div");
    status.className = "download-status";
    status.textContent = statusLabel(item.status);
    const buttons = document.createElement("div");
    buttons.className = "download-actions";
    const open = document.createElement("button");
    open.type = "button";
    open.setAttribute("data-test", "download-open");
    open.textContent = "Open";
    open.disabled = item.status !== "done";
    open.addEventListener("click", () => handlers.onOpen(item.id));
    const reveal = document.createElement("button");
    reveal.type = "button";
    reveal.setAttribute("data-test", "download-reveal");
    reveal.textContent = "Show";
    reveal.disabled = item.status !== "done";
    reveal.addEventListener("click", () => handlers.onReveal(item.id));
    buttons.append(open, reveal);
    row.append(name, status, buttons);
    root.append(row);
  }
}

function statusLabel(status: DownloadRow["status"]): string {
  switch (status) {
    case "saving":
      return "Saving…";
    case "done":
      return "Saved";
    case "failed":
      return "Couldn’t save";
  }
}
// ai-generated end
