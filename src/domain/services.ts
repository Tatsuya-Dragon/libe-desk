import type { ServiceId } from "./types";

export interface ServiceDefinition {
  id: Exclude<ServiceId, "custom">;
  name: string;
  shortName: string;
  url: string;
  icon: string;
}

export const SERVICES: readonly ServiceDefinition[] = [
  { id: "city", name: "リベシティ", shortName: "シティ", url: "https://libecity.com/", icon: "🏠" },
  { id: "library", name: "ノウハウ図書館", shortName: "図書館", url: "https://library.libecity.com/", icon: "📚" },
  { id: "skill", name: "スキルマーケット", shortName: "スキル", url: "https://skill.libecity.com/", icon: "🛠️" },
  { id: "ichiba", name: "リベシティ市場", shortName: "市場", url: "https://ichiba.libecity.com/", icon: "🛍️" },
] as const;

export function serviceForUrl(url: string): ServiceId {
  const parsed = new URL(url);
  return SERVICES.find((service) => new URL(service.url).hostname === parsed.hostname)?.id ?? "custom";
}
