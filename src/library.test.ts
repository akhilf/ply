// ai-generated
import { describe, expect, it } from "vitest";
import { renderLibrary } from "./library";

describe("renderLibrary", () => {
  it("renders bookmark titles and opens on click", () => {
    const root = document.createElement("div");
    const opened: string[] = [];
    renderLibrary(
      root,
      {
        bookmarks: [{ url: "https://a.test/", title: "Alpha", created_at: 1 }],
        visits: [{ id: 1, url: "https://b.test/", title: "Beta", visited_at: 2 }],
      },
      (url) => opened.push(url),
    );
    expect(root.textContent).toContain("Alpha");
    expect(root.textContent).toContain("Beta");
    root.querySelector<HTMLElement>('[data-test=bookmark-row][data-url="https://a.test/"]')!.click();
    expect(opened).toEqual(["https://a.test/"]);
  });
});
// ai-generated end
