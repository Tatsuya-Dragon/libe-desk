import { For, Show, createMemo, createSignal, onMount } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import { SERVICES } from "./domain/services";
import { decideNavigation } from "./domain/navigation-policy";
import type { TabInfo } from "./domain/types";
import { tauriOperations } from "./operations/tauri-operations";

export function App() {
  const [tabs, setTabs] = createSignal<TabInfo[]>([]);
  const [busy, setBusy] = createSignal(false);
  const [message, setMessage] = createSignal("非公式アプリです");
  const [urlInput, setUrlInput] = createSignal("");
  const activeTab = createMemo(() => tabs().find((tab) => tab.active));

  const openEnteredUrl = async () => {
    const raw = urlInput().trim();
    if (!raw) return;
    const normalized = /^[a-z][a-z0-9+.-]*:/i.test(raw) ? raw : `https://${raw}`;
    const decision = decideNavigation(normalized);
    if (decision === "reject") {
      setMessage("このURLは安全上の理由で開けません");
      return;
    }
    await run(() => decision === "internal"
      ? tauriOperations.openUrl({ url: normalized, disposition: "new-tab" })
      : tauriOperations.openExternalUrl(normalized));
    setUrlInput("");
  };

  const refreshTabs = async () => setTabs(await tauriOperations.listTabs());
  const run = async (action: () => Promise<unknown>) => {
    setBusy(true);
    try {
      await action();
      await refreshTabs();
      setMessage("非公式アプリです");
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  };

  onMount(async () => {
    await refreshTabs();
    await listen("tabs-changed", refreshTabs);
    await listen<string>("open-internal-url", (event) => {
      void run(() => tauriOperations.openUrl({ url: event.payload, disposition: "new-tab" }));
    });
  });

  return (
    <main class="app-shell">
      <aside class="sidebar" aria-label="サービス">
        <div class="brand"><span class="brand-mark">L</span><strong>Libe Desk</strong></div>
        <nav>
          <For each={SERVICES}>{(service) => (
            <button class="service-button" onClick={() => run(() => tauriOperations.openUrl({ url: service.url, disposition: "new-tab" }))}>
              <span aria-hidden="true">{service.icon}</span><span>{service.name}</span>
            </button>
          )}</For>
        </nav>
        <div class="sidebar-footer"><span>⚙️</span><span>設定</span></div>
      </aside>

      <section class="workspace">
        <div class="tabs" role="tablist" aria-label="開いているページ">
          <For each={tabs()}>{(tab) => (
            <button classList={{ tab: true, active: tab.active }} role="tab" aria-selected={tab.active} onClick={() => run(() => tauriOperations.activateTab(tab.id))}>
              <span class="tab-title">{tab.title || "読み込み中…"}</span>
              <span class="tab-close" role="button" aria-label={`${tab.title}を閉じる`} onClick={(event) => { event.stopPropagation(); void run(() => tauriOperations.closeTab(tab.id)); }}>×</span>
            </button>
          )}</For>
          <button class="new-tab" aria-label="リベシティを新しいタブで開く" onClick={() => run(() => tauriOperations.openUrl({ url: SERVICES[0].url, disposition: "new-tab" }))}>＋</button>
        </div>

        <div class="toolbar">
          <button disabled={!activeTab()?.canGoBack || busy()} aria-label="戻る" onClick={() => run(() => tauriOperations.goBack(activeTab()!.id))}>←</button>
          <button disabled={!activeTab()?.canGoForward || busy()} aria-label="進む" onClick={() => run(() => tauriOperations.goForward(activeTab()!.id))}>→</button>
          <button disabled={!activeTab() || busy()} aria-label="再読み込み" onClick={() => run(() => tauriOperations.reload(activeTab()!.id))}>↻</button>
          <form class="address-form" onSubmit={(event) => { event.preventDefault(); void openEnteredUrl(); }}>
            <input
              class="address"
              aria-label="開くURL"
              placeholder={activeTab()?.url ?? "URLを入力してください"}
              value={urlInput()}
              onInput={(event) => setUrlInput(event.currentTarget.value)}
            />
            <button type="submit" disabled={!urlInput().trim() || busy()}>開く</button>
          </form>
        </div>

        <div class="webview-space" aria-hidden={!!activeTab()}>
          <Show when={!activeTab()}>
            <div class="empty-state"><div class="empty-icon">L</div><h1>サービスを開く</h1><p>左のメニューから利用するサービスを選んでください。</p></div>
          </Show>
        </div>
        <div class="status" role="status"><span classList={{ pulse: busy() }} />{message()}</div>
      </section>
    </main>
  );
}
