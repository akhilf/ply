// ai-generated
import { describe, expect, it } from "vitest";
import { groupVisitsByDay, renderLibrary } from "./library";

describe("renderLibrary", () => {
  it("renders bookmark titles and opens on click", () => {
    const root = document.createElement("div");
    const opened: string[] = [];
    renderLibrary(
      root,
      {
        bookmarks: [{ url: "https://a.test/", title: "Alpha", created_at: 1 }],
        visits: [{ id: 1, url: "https://b.test/", title: "Beta", visited_at: 2, visit_count: 1 }],
      },
      (url) => opened.push(url),
    );
    expect(root.textContent).toContain("Alpha");
    expect(root.textContent).toContain("Beta");
    root.querySelector<HTMLElement>('[data-test=bookmark-row][data-url="https://a.test/"]')!.click();
    expect(opened).toEqual(["https://a.test/"]);
  });

  it("groups history by day and shows visit counts", () => {
    const now = Date.parse("2026-11-10T18:00:00Z");
    const today = Math.floor(now / 1000);
    const yesterday = Math.floor(Date.parse("2026-11-09T18:00:00Z") / 1000);
    const groups = groupVisitsByDay(
      [
        { id: 2, url: "https://b.test/", title: "Beta", visited_at: today, visit_count: 3 },
        { id: 1, url: "https://a.test/", title: "Alpha", visited_at: yesterday, visit_count: 1 },
      ],
      now,
    );
    expect(groups.map((g) => g.label)).toEqual(["Today", "Yesterday"]);
    const root = document.createElement("div");
    renderLibrary(
      root,
      {
        bookmarks: [],
        visits: [
          { id: 2, url: "https://b.test/", title: "Beta", visited_at: today, visit_count: 3 },
        ],
      },
      () => {},
    );
    expect(root.textContent).toContain("3 visits");
    expect(root.textContent).toContain("Today");
  });
});
// ai-generated end
