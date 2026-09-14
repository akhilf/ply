// ai-generated
import type { BookmarkRow, VisitRow } from "./types";

export type LibraryData = {
  bookmarks: BookmarkRow[];
  visits: VisitRow[];
};

export function renderLibrary(
  root: HTMLElement,
  data: LibraryData,
  onOpenUrl: (url: string) => void,
  onSearch?: (query: string) => void,
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
  });
  head.append(title, close);
  root.append(head);

  const bookmarksHeading = document.createElement("h2");
  bookmarksHeading.textContent = "Bookmarks";
  root.append(bookmarksHeading);
  for (const bookmark of data.bookmarks) {
    const row = document.createElement("button");
    row.type = "button";
    row.setAttribute("data-test", "bookmark-row");
    row.setAttribute("data-url", bookmark.url);
    row.textContent = bookmark.title || bookmark.url;
    row.addEventListener("click", () => onOpenUrl(bookmark.url));
    root.append(row);
  }

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
  for (const visit of data.visits) {
    const row = document.createElement("button");
    row.type = "button";
    row.setAttribute("data-test", "visit-row");
    row.setAttribute("data-url", visit.url);
    row.textContent = visit.title || visit.url;
    row.addEventListener("click", () => onOpenUrl(visit.url));
    root.append(row);
  }
}
// ai-generated end
