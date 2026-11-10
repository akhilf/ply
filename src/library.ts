// ai-generated
import type { BookmarkRow, VisitRow } from "./types";

export type LibraryData = {
  bookmarks: BookmarkRow[];
  visits: VisitRow[];
};

export type HistoryGroup = {
  label: string;
  visits: VisitRow[];
};

export function groupVisitsByDay(visits: VisitRow[], nowMs = Date.now()): HistoryGroup[] {
  const groups = new Map<string, VisitRow[]>();
  const order: string[] = [];
  for (const visit of visits) {
    const label = dayLabel(visit.visited_at, nowMs);
    if (!groups.has(label)) {
      order.push(label);
      groups.set(label, []);
    }
    groups.get(label)!.push(visit);
  }
  return order.map((label) => ({ label, visits: groups.get(label)! }));
}

export function renderLibrary(
  root: HTMLElement,
  data: LibraryData,
  onOpenUrl: (url: string) => void,
  onSearch?: (query: string) => void,
  onClose?: () => void,
): void {
  const prior = root.querySelector<HTMLInputElement>("[data-test=history-search]")?.value ?? "";
  root.replaceChildren();
  root.removeAttribute("hidden");

  const head = document.createElement("div");
  head.className = "library-head";
  const title = document.createElement("strong");
  title.textContent = "Library";
  const close = document.createElement("button");
  close.type = "button";
  close.setAttribute("data-test", "library-close");
  close.textContent = "Close";
  close.addEventListener("click", () => {
    root.setAttribute("hidden", "");
    onClose?.();
  });
  head.append(title, close);
  root.append(head);

  const historyHeading = document.createElement("h2");
  historyHeading.textContent = "History";
  root.append(historyHeading);
  const search = document.createElement("input");
  search.type = "search";
  search.setAttribute("data-test", "history-search");
  search.placeholder = "Search history";
  search.value = prior;
  search.addEventListener("input", () => {
    onSearch?.(search.value);
  });
  root.append(search);

  if (data.visits.length === 0) {
    const empty = document.createElement("p");
    empty.setAttribute("data-test", "history-empty");
    empty.textContent = "Pages you visit will show up here.";
    root.append(empty);
  } else {
    for (const group of groupVisitsByDay(data.visits)) {
      const day = document.createElement("h3");
      day.className = "history-day";
      day.textContent = group.label;
      root.append(day);
      for (const visit of group.visits) {
        const row = document.createElement("button");
        row.type = "button";
        row.className = "visit-row";
        row.setAttribute("data-test", "visit-row");
        row.setAttribute("data-url", visit.url);
        const name = document.createElement("span");
        name.className = "visit-title";
        name.textContent = visit.title || visit.url;
        const meta = document.createElement("span");
        meta.className = "visit-meta";
        const count =
          visit.visit_count > 1 ? ` · ${visit.visit_count} visits` : "";
        meta.textContent = `${hostOf(visit.url)} · ${formatTime(visit.visited_at)}${count}`;
        row.append(name, meta);
        row.addEventListener("click", () => onOpenUrl(visit.url));
        root.append(row);
      }
    }
  }

  const bookmarksHeading = document.createElement("h2");
  bookmarksHeading.textContent = "Bookmarks";
  root.append(bookmarksHeading);
  if (data.bookmarks.length === 0) {
    const empty = document.createElement("p");
    empty.setAttribute("data-test", "bookmarks-empty");
    empty.textContent = "Star a page to keep it here.";
    root.append(empty);
  }
  for (const bookmark of data.bookmarks) {
    const row = document.createElement("button");
    row.type = "button";
    row.setAttribute("data-test", "bookmark-row");
    row.setAttribute("data-url", bookmark.url);
    row.textContent = bookmark.title || bookmark.url;
    row.addEventListener("click", () => onOpenUrl(bookmark.url));
    root.append(row);
  }
}

function dayLabel(visitedAt: number, nowMs: number): string {
  const day = startOfDay(visitedAt * 1000);
  const today = startOfDay(nowMs);
  const yesterdayDate = new Date(nowMs);
  yesterdayDate.setDate(yesterdayDate.getDate() - 1);
  const yesterday = startOfDay(yesterdayDate.getTime());
  if (day === today) {
    return "Today";
  }
  if (day === yesterday) {
    return "Yesterday";
  }
  return new Date(visitedAt * 1000).toLocaleDateString(undefined, {
    weekday: "long",
    month: "short",
    day: "numeric",
  });
}

function startOfDay(ms: number): number {
  const d = new Date(ms);
  d.setHours(0, 0, 0, 0);
  return d.getTime();
}

function formatTime(visitedAt: number): string {
  return new Date(visitedAt * 1000).toLocaleTimeString(undefined, {
    hour: "numeric",
    minute: "2-digit",
  });
}

function hostOf(url: string): string {
  try {
    return new URL(url).host || url;
  } catch {
    return url;
  }
}
// ai-generated end
