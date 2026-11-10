// ai-generated
import { describe, expect, it } from "vitest";
import { renderChrome } from "./chrome";
import type { ChromeHandlers, SessionSnapshot } from "./types";

const base: SessionSnapshot = {
  tabs: [
    {
      id: "tab-1",
      title: "Example",
      url: "https://example.com/",
      favicon_url: "",
      engine: "native",
      load_error: null,
    },
  ],
  active_tab_id: "tab-1",
  current_url: "https://example.com/",
  can_go_back: false,
  can_go_forward: false,
  is_bookmarked: false,
  load_error: null,
  persist_mode: "disk",
};

function noopHandlers(): ChromeHandlers {
  return {
    onNavigate: () => {},
    onNewTab: () => {},
    onCloseTab: () => {},
    onSwitchTab: () => {},
    onBack: () => {},
    onForward: () => {},
    onReload: () => {},
    onToggleBookmark: () => {},
    onOpenLibrary: () => {},
    onOpenDownloads: () => {},
    onOpenFind: () => {},
    onFindQuery: () => {},
    onFindNext: () => {},
    onFindPrev: () => {},
    onCloseFind: () => {},
  };
}

describe("renderChrome", () => {
  it("shows the active URL and disables back", () => {
    const root = document.createElement("div");
    renderChrome(root, base, noopHandlers());
    const url = root.querySelector<HTMLInputElement>("[data-test=url]")!;
    expect(url.value).toBe("https://example.com/");
    const back = root.querySelector<HTMLButtonElement>("[data-test=back]")!;
    expect(back.disabled).toBe(true);
    expect(root.querySelector("[data-test=tab-tab-1]")!.getAttribute("aria-selected")).toBe(
      "true",
    );
  });

  it("shows the error line only when load_error is set", () => {
    const root = document.createElement("div");
    renderChrome(root, { ...base, load_error: null }, noopHandlers());
    expect(root.querySelector("[data-test=load-error]")!.hasAttribute("hidden")).toBe(true);
    renderChrome(root, { ...base, load_error: "invalid_url" }, noopHandlers());
    const line = root.querySelector("[data-test=load-error]")!;
    expect(line.hasAttribute("hidden")).toBe(false);
    expect(line.textContent).toContain("This isn’t a valid address.");
  });

  it("does not navigate on empty submit", () => {
    const calls: string[] = [];
    const root = document.createElement("div");
    const handlers = {
      ...noopHandlers(),
      onNavigate: (_id: string, input: string) => calls.push(input),
    };
    renderChrome(root, base, handlers);
    const form = root.querySelector("form")!;
    root.querySelector<HTMLInputElement>("[data-test=url]")!.value = "   ";
    form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
    expect(calls).toEqual([]);
  });

  it("highlights the Ply wordmark in chrome", () => {
    const root = document.createElement("div");
    renderChrome(root, base, noopHandlers());
    const brand = root.querySelector("[data-test=brand]")!;
    expect(brand.textContent).toContain("Ply");
    expect(brand.querySelector(".ply-wordmark")!.textContent).toBe("Ply");
  });

  it("does not show the Toy engine switch", () => {
    const root = document.createElement("div");
    renderChrome(root, base, noopHandlers());
    expect(root.querySelector("[data-test=engine-toy]")).toBeNull();
    expect(root.querySelector("[data-test=engine-native]")).toBeNull();
  });

  it("opens find in page from the toolbar", () => {
    const root = document.createElement("div");
    const calls: string[] = [];
    const handlers = {
      ...noopHandlers(),
      onOpenFind: () => calls.push("open"),
      onFindNext: () => calls.push("next"),
    };
    renderChrome(root, base, handlers, { findOpen: true, findQuery: "hello" });
    expect(root.querySelector<HTMLInputElement>("[data-test=find-query]")!.value).toBe("hello");
    root.querySelector<HTMLButtonElement>("[data-test=find]")!.click();
    root.querySelector("form.find-bar")!.dispatchEvent(
      new Event("submit", { bubbles: true, cancelable: true }),
    );
    expect(calls).toEqual(["open", "next"]);
  });

  it("shows the memory persist banner", () => {
    const root = document.createElement("div");
    renderChrome(root, { ...base, persist_mode: "memory" }, noopHandlers());
    const banner = root.querySelector("[data-test=memory-notice]")!;
    expect(banner.hasAttribute("hidden")).toBe(false);
    expect(banner.textContent).toContain(
      "Bookmarks and history will not persist this launch.",
    );
  });

  it("opens downloads and badges in-progress saves", () => {
    const root = document.createElement("div");
    const calls: string[] = [];
    const handlers = {
      ...noopHandlers(),
      onOpenDownloads: () => calls.push("open"),
    };
    renderChrome(root, base, handlers, { downloadSaving: 2 });
    const btn = root.querySelector<HTMLButtonElement>("[data-test=downloads]")!;
    expect(root.querySelector("[data-test=downloads-badge]")!.textContent).toBe("2");
    btn.click();
    expect(calls).toEqual(["open"]);
  });
});
// ai-generated end
