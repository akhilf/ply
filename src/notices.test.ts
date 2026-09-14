// ai-generated
import { describe, expect, it } from "vitest";
import { LOAD_ERROR_COPY, MEMORY_NOTICE } from "./notices";

describe("notices", () => {
  it("matches spec copy", () => {
    expect(LOAD_ERROR_COPY.invalid_url).toBe("This isn’t a valid address.");
    expect(LOAD_ERROR_COPY.unreachable).toBe("This site can’t be reached.");
    expect(LOAD_ERROR_COPY.load_failed).toBe("This page failed to load.");
    expect(MEMORY_NOTICE).toBe(
      "Bookmarks and history will not persist this launch.",
    );
  });
});
// ai-generated end
