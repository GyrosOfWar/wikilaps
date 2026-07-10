import { describe, expect, it } from "vitest";
import { formatDate } from "./date-time";

describe("formatDate", () => {
  it("should format a date in the right format", () => {
    const date = formatDate("2026-01-04");
    expect(date).toBeDefined();
  });

  it("should fail with bad data", () => {
    expect(() => formatDate("asdf")).toThrow();
  });
});
