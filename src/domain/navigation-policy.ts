export type NavigationDecision = "internal" | "external" | "reject";

export function decideNavigation(rawUrl: string): NavigationDecision {
  let url: URL;
  try {
    url = new URL(rawUrl);
  } catch {
    return "reject";
  }

  if (["mailto:", "tel:"].includes(url.protocol)) return "external";
  if (!["https:", "http:"].includes(url.protocol)) return "reject";

  const host = url.hostname.toLowerCase();
  return host === "libecity.com" || host.endsWith(".libecity.com") ? "internal" : "external";
}
