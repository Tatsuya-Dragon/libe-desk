import { describe, expect, it } from "vitest";
import { decideNavigation } from "./navigation-policy";

describe("decideNavigation", () => {
  it.each([
    "https://libecity.com/",
    "https://library.libecity.com/articles/1",
    "https://skill.libecity.com/",
  ])("allows a Libe City host: %s", (url) => expect(decideNavigation(url)).toBe("internal"));

  it("does not accept a deceptive suffix", () => {
    expect(decideNavigation("https://libecity.com.example.org/")).toBe("external");
  });

  it.each(["javascript:alert(1)", "data:text/html,test", "not a url"])(
    "rejects an unsafe or invalid URL: %s",
    (url) => expect(decideNavigation(url)).toBe("reject"),
  );

  it.each(["https://example.com/", "mailto:hello@example.com", "tel:0123456789"])(
    "delegates an external URL: %s",
    (url) => expect(decideNavigation(url)).toBe("external"),
  );
});
