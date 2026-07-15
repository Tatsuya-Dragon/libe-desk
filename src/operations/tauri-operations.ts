import { invoke } from "@tauri-apps/api/core";
import type { AppOperations, PageInfo, TabId, TabInfo } from "../domain/types";

export const tauriOperations: AppOperations = {
  openUrl: (input) => invoke<TabInfo>("open_url", { input }),
  openExternalUrl: (url) => invoke("open_external_url", { url }),
  listTabs: () => invoke<TabInfo[]>("list_tabs"),
  activateTab: (tabId) => invoke("activate_tab", { tabId }),
  closeTab: (tabId) => invoke("close_tab", { tabId }),
  goBack: (tabId) => invoke("go_back", { tabId }),
  goForward: (tabId) => invoke("go_forward", { tabId }),
  reload: (tabId) => invoke("reload", { tabId }),
  getPageInfo: (tabId?: TabId) => invoke<PageInfo>("get_page_info", { tabId }),
};
