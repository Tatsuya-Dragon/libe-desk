export type TabId = string;
export type ServiceId = "city" | "library" | "skill" | "ichiba" | "custom";

export interface TabInfo {
  id: TabId;
  serviceId: ServiceId;
  title: string;
  url: string;
  active: boolean;
  loading: boolean;
  canGoBack: boolean;
  canGoForward: boolean;
}

export interface PageInfo {
  tabId: TabId;
  title: string;
  url: string;
  loading: boolean;
  canGoBack: boolean;
  canGoForward: boolean;
}

export type AppErrorCode =
  | "TAB_NOT_FOUND"
  | "TAB_LIMIT_REACHED"
  | "INVALID_URL"
  | "NAVIGATION_UNAVAILABLE"
  | "WEBVIEW_FAILURE"
  | "EXTERNAL_OPEN_FAILURE";

export interface AppOperations {
  openUrl(input: { url: string; disposition?: "current" | "new-tab" }): Promise<TabInfo>;
  openExternalUrl(url: string): Promise<void>;
  listTabs(): Promise<TabInfo[]>;
  activateTab(tabId: TabId): Promise<void>;
  closeTab(tabId: TabId): Promise<void>;
  goBack(tabId: TabId): Promise<void>;
  goForward(tabId: TabId): Promise<void>;
  reload(tabId: TabId): Promise<void>;
  getPageInfo(tabId?: TabId): Promise<PageInfo>;
}
