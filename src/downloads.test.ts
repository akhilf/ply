// ai-generated
import { describe, expect, it } from "vitest";
import { renderDownloads, type DownloadRow } from "./downloads";

function row(partial: Partial<DownloadRow>): DownloadRow {
  return {
    id: 1,
    url: "https://a.test/note.txt",
    filename: "note.txt",
    path: "/tmp/note.txt",
    status: "done",
    started_at: 1,
    ...partial,
  };
}

describe("renderDownloads", () => {
  it("shows empty copy when there are no files", () => {
    const root = document.createElement("div");
    renderDownloads(root, [], {
      onOpen: () => {},
      onReveal: () => {},
      onClear: () => {},
      onClose: () => {},
    });
    expect(root.querySelector("[data-test=downloads-empty]")!.textContent).toContain(
      "Files you save from a page",
    );
  });

  it("opens a finished file and keeps saving rows disabled", () => {
    const root = document.createElement("div");
    const opened: number[] = [];
    renderDownloads(
      root,
      [row({ id: 2, filename: "done.txt", status: "done" }), row({ id: 1, filename: "part.bin", status: "saving" })],
      {
        onOpen: (id) => opened.push(id),
        onReveal: () => {},
        onClear: () => {},
        onClose: () => {},
      },
    );
    const rows = root.querySelectorAll("[data-test=download-row]");
    expect(rows).toHaveLength(2);
    expect(root.textContent).toContain("done.txt");
    expect(root.textContent).toContain("Saving…");
    const openButtons = root.querySelectorAll<HTMLButtonElement>("[data-test=download-open]");
    expect(openButtons[0]!.disabled).toBe(false);
    expect(openButtons[1]!.disabled).toBe(true);
    openButtons[0]!.click();
    expect(opened).toEqual([2]);
  });

  it("clears and closes from the header", () => {
    const root = document.createElement("div");
    const calls: string[] = [];
    renderDownloads(root, [row({})], {
      onOpen: () => {},
      onReveal: () => {},
      onClear: () => calls.push("clear"),
      onClose: () => calls.push("close"),
    });
    root.querySelector<HTMLButtonElement>("[data-test=downloads-clear]")!.click();
    root.querySelector<HTMLButtonElement>("[data-test=downloads-close]")!.click();
    expect(calls).toEqual(["clear", "close"]);
    expect(root.hasAttribute("hidden")).toBe(true);
  });
});
// ai-generated end
