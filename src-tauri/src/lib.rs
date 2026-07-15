mod navigation_policy;

use navigation_policy::{decide_navigation, NavigationDecision};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager, Runtime, State, WebviewUrl};
use thiserror::Error;
use url::Url;
use uuid::Uuid;

const TAB_LIMIT: usize = 20;
const SIDEBAR_WIDTH: f64 = 184.0;
const CHROME_HEIGHT: f64 = 92.0;
const STATUS_HEIGHT: f64 = 25.0;
const EXTERNAL_LINK_INTERCEPTOR: &str = r#"
document.addEventListener('click', (event) => {
  if (!event.isTrusted || event.defaultPrevented || event.button !== 0) return;

  const target = event.target;
  const anchor = target instanceof Element ? target.closest('a[href]') : null;
  if (!anchor || anchor.target === '_blank') return;

  let destination;
  try {
    destination = new URL(anchor.href, window.location.href);
  } catch (_) {
    return;
  }

  const host = destination.hostname.toLowerCase();
  const isLibeCity = host === 'libecity.com' || host.endsWith('.libecity.com');
  const isExternal = destination.protocol === 'mailto:' ||
    destination.protocol === 'tel:' ||
    ((destination.protocol === 'https:' || destination.protocol === 'http:') && !isLibeCity);

  if (isExternal) {
    event.preventDefault();
    event.stopImmediatePropagation();
    window.open(destination.href, '_blank');
  }
}, true);
"#;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "code", content = "message")]
enum AppError {
    #[error("対象のタブが見つかりません")]
    TabNotFound,
    #[error("開けるタブは最大20件です")]
    TabLimitReached,
    #[error("このURLは開けません")]
    InvalidUrl,
    #[error("ページ表示の操作に失敗しました: {0}")]
    WebviewFailure(String),
}

type Result<T> = std::result::Result<T, AppError>;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenUrlInput {
    url: String,
    disposition: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TabInfo {
    id: String,
    service_id: String,
    title: String,
    url: String,
    active: bool,
    loading: bool,
    can_go_back: bool,
    can_go_forward: bool,
}

#[derive(Default)]
struct TabRegistry {
    active_id: Option<String>,
    order: Vec<String>,
    tabs: HashMap<String, TabInfo>,
}

struct AppState(Mutex<TabRegistry>);

fn service_id(url: &Url) -> &'static str {
    match url.host_str().unwrap_or_default() {
        "libecity.com" => "city",
        "library.libecity.com" => "library",
        "skill.libecity.com" => "skill",
        "ichiba.libecity.com" => "ichiba",
        _ => "custom",
    }
}

fn service_title(id: &str) -> &'static str {
    match id {
        "city" => "リベシティ",
        "library" => "ノウハウ図書館",
        "skill" => "スキルマーケット",
        "ichiba" => "リベシティ市場",
        _ => "リベシティ関連ページ",
    }
}

fn webview_label(tab_id: &str) -> String {
    format!("tab-{tab_id}")
}

fn emit_tabs<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.emit("tabs-changed", ());
}

fn content_bounds<R: Runtime>(app: &AppHandle<R>) -> Result<(f64, f64)> {
    let window = app.get_window("main").ok_or(AppError::WebviewFailure(
        "メインウィンドウがありません".into(),
    ))?;
    let size = window
        .inner_size()
        .map_err(|error| AppError::WebviewFailure(error.to_string()))?;
    let scale = window
        .scale_factor()
        .map_err(|error| AppError::WebviewFailure(error.to_string()))?;
    Ok((
        size.width as f64 / scale - SIDEBAR_WIDTH,
        size.height as f64 / scale - CHROME_HEIGHT - STATUS_HEIGHT,
    ))
}

#[tauri::command]
async fn open_url<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    input: OpenUrlInput,
) -> Result<TabInfo> {
    let _ = input.disposition.as_deref();
    if decide_navigation(&input.url) != NavigationDecision::Internal {
        return Err(AppError::InvalidUrl);
    }
    let parsed = Url::parse(&input.url).map_err(|_| AppError::InvalidUrl)?;

    {
        let registry = state
            .0
            .lock()
            .map_err(|error| AppError::WebviewFailure(error.to_string()))?;
        if registry.tabs.len() >= TAB_LIMIT {
            return Err(AppError::TabLimitReached);
        }
    }

    let id = Uuid::new_v4().simple().to_string();
    let label = webview_label(&id);
    let (width, height) = content_bounds(&app)?;
    let window = app.get_window("main").ok_or(AppError::WebviewFailure(
        "メインウィンドウがありません".into(),
    ))?;
    let new_window_app = app.clone();
    let page_tab_id = id.clone();
    let title_tab_id = id.clone();
    let webview = tauri::webview::WebviewBuilder::new(&label, WebviewUrl::External(parsed.clone()))
        .initialization_script(EXTERNAL_LINK_INTERCEPTOR)
        .on_navigation(move |url| match decide_navigation(url.as_str()) {
            NavigationDecision::Internal => true,
            NavigationDecision::External | NavigationDecision::Reject => false,
        })
        .on_new_window(move |url, _features| {
            match decide_navigation(url.as_str()) {
                NavigationDecision::Internal => {
                    let _ = new_window_app.emit("open-internal-url", url.to_string());
                }
                NavigationDecision::External => {
                    use tauri_plugin_opener::OpenerExt;
                    let _ = new_window_app.opener().open_url(url.as_str(), None::<&str>);
                }
                NavigationDecision::Reject => {}
            }
            tauri::webview::NewWindowResponse::Deny
        })
        .on_page_load(move |webview, payload| {
            let app = webview.app_handle();
            let state = app.state::<AppState>();
            if let Ok(mut registry) = state.0.lock() {
                if let Some(tab) = registry.tabs.get_mut(&page_tab_id) {
                    let next_url = payload.url().to_string();
                    if tab.url != next_url {
                        tab.can_go_back = true;
                        tab.can_go_forward = false;
                    }
                    tab.url = next_url;
                    tab.loading = matches!(payload.event(), tauri::webview::PageLoadEvent::Started);
                }
            }
            emit_tabs(app);
        })
        .on_document_title_changed(move |webview, title| {
            let app = webview.app_handle();
            let state = app.state::<AppState>();
            if let Ok(mut registry) = state.0.lock() {
                if let Some(tab) = registry.tabs.get_mut(&title_tab_id) {
                    if !title.trim().is_empty() {
                        tab.title = title;
                    }
                }
            }
            emit_tabs(app);
        });
    for (_, existing) in app
        .webviews()
        .into_iter()
        .filter(|(_, view)| view.label().starts_with("tab-"))
    {
        existing
            .hide()
            .map_err(|error| AppError::WebviewFailure(error.to_string()))?;
    }
    window
        .add_child(
            webview,
            tauri::LogicalPosition::new(SIDEBAR_WIDTH, CHROME_HEIGHT),
            tauri::LogicalSize::new(width.max(1.0), height.max(1.0)),
        )
        .map_err(|error| AppError::WebviewFailure(error.to_string()))?;

    let sid = service_id(&parsed).to_string();
    let tab = TabInfo {
        id: id.clone(),
        service_id: sid.clone(),
        title: service_title(&sid).into(),
        url: input.url,
        active: true,
        loading: true,
        can_go_back: false,
        can_go_forward: false,
    };
    {
        let mut registry = state
            .0
            .lock()
            .map_err(|error| AppError::WebviewFailure(error.to_string()))?;
        for existing in registry.tabs.values_mut() {
            existing.active = false;
        }
        registry.active_id = Some(id.clone());
        registry.order.push(id.clone());
        registry.tabs.insert(id, tab.clone());
    }
    emit_tabs(&app);
    Ok(tab)
}

#[tauri::command]
fn open_external_url<R: Runtime>(app: AppHandle<R>, url: String) -> Result<()> {
    use tauri_plugin_opener::OpenerExt;
    if decide_navigation(&url) != NavigationDecision::External {
        return Err(AppError::InvalidUrl);
    }
    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|error| AppError::WebviewFailure(error.to_string()))
}

#[tauri::command]
fn list_tabs(state: State<'_, AppState>) -> Result<Vec<TabInfo>> {
    let registry = state
        .0
        .lock()
        .map_err(|error| AppError::WebviewFailure(error.to_string()))?;
    Ok(registry
        .order
        .iter()
        .filter_map(|id| registry.tabs.get(id).cloned())
        .collect())
}

#[tauri::command]
async fn activate_tab<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    tab_id: String,
) -> Result<()> {
    let labels: Vec<(String, bool)> = {
        let mut registry = state
            .0
            .lock()
            .map_err(|error| AppError::WebviewFailure(error.to_string()))?;
        if !registry.tabs.contains_key(&tab_id) {
            return Err(AppError::TabNotFound);
        }
        registry.active_id = Some(tab_id.clone());
        registry
            .tabs
            .values_mut()
            .map(|tab| {
                tab.active = tab.id == tab_id;
                (webview_label(&tab.id), tab.active)
            })
            .collect()
    };
    for (label, active) in labels {
        if let Some(webview) = app.get_webview(&label) {
            let result = if active {
                webview.show()
            } else {
                webview.hide()
            };
            result.map_err(|error| AppError::WebviewFailure(error.to_string()))?;
        }
    }
    emit_tabs(&app);
    Ok(())
}

#[tauri::command]
async fn close_tab<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    tab_id: String,
) -> Result<()> {
    let next_id = {
        let mut registry = state
            .0
            .lock()
            .map_err(|error| AppError::WebviewFailure(error.to_string()))?;
        let removed_index = registry
            .order
            .iter()
            .position(|id| id == &tab_id)
            .ok_or(AppError::TabNotFound)?;
        if registry.tabs.remove(&tab_id).is_none() {
            return Err(AppError::TabNotFound);
        }
        registry.order.retain(|id| id != &tab_id);
        let next = if registry.order.is_empty() {
            None
        } else {
            Some(registry.order[removed_index.min(registry.order.len() - 1)].clone())
        };
        registry.active_id = next.clone();
        for tab in registry.tabs.values_mut() {
            tab.active = Some(&tab.id) == next.as_ref();
        }
        next
    };
    if let Some(webview) = app.get_webview(&webview_label(&tab_id)) {
        webview
            .close()
            .map_err(|error| AppError::WebviewFailure(error.to_string()))?;
    }
    if let Some(next) = next_id {
        activate_tab(app.clone(), state, next).await?;
    }
    emit_tabs(&app);
    Ok(())
}

#[tauri::command]
async fn go_back<R: Runtime>(app: AppHandle<R>, tab_id: String) -> Result<()> {
    let webview = app
        .get_webview(&webview_label(&tab_id))
        .ok_or(AppError::TabNotFound)?;
    webview
        .eval("history.back()")
        .map_err(|error| AppError::WebviewFailure(error.to_string()))?;
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut registry) = state.0.lock() {
            if let Some(tab) = registry.tabs.get_mut(&tab_id) {
                tab.can_go_forward = true;
            }
        }
    }
    emit_tabs(&app);
    Ok(())
}

#[tauri::command]
async fn go_forward<R: Runtime>(app: AppHandle<R>, tab_id: String) -> Result<()> {
    let webview = app
        .get_webview(&webview_label(&tab_id))
        .ok_or(AppError::TabNotFound)?;
    webview
        .eval("history.forward()")
        .map_err(|error| AppError::WebviewFailure(error.to_string()))?;
    emit_tabs(&app);
    Ok(())
}

#[tauri::command]
async fn reload<R: Runtime>(app: AppHandle<R>, tab_id: String) -> Result<()> {
    let webview = app
        .get_webview(&webview_label(&tab_id))
        .ok_or(AppError::TabNotFound)?;
    webview
        .eval("location.reload()")
        .map_err(|error| AppError::WebviewFailure(error.to_string()))
}

#[tauri::command]
fn get_page_info(state: State<'_, AppState>, tab_id: Option<String>) -> Result<TabInfo> {
    let registry = state
        .0
        .lock()
        .map_err(|error| AppError::WebviewFailure(error.to_string()))?;
    let id = tab_id
        .or_else(|| registry.active_id.clone())
        .ok_or(AppError::TabNotFound)?;
    registry.tabs.get(&id).cloned().ok_or(AppError::TabNotFound)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState(Mutex::new(TabRegistry::default())))
        .invoke_handler(tauri::generate_handler![
            open_url,
            open_external_url,
            list_tabs,
            activate_tab,
            close_tab,
            go_back,
            go_forward,
            reload,
            get_page_info
        ])
        .setup(|app| {
            if let Some(window) = app.get_window("main") {
                let resize_app = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Resized(size) = event {
                        let Some(main_window) = resize_app.get_window("main") else {
                            return;
                        };
                        let Ok(scale) = main_window.scale_factor() else {
                            return;
                        };
                        let logical_width = size.width as f64 / scale - SIDEBAR_WIDTH;
                        let logical_height =
                            size.height as f64 / scale - CHROME_HEIGHT - STATUS_HEIGHT;
                        for (_, webview) in resize_app
                            .webviews()
                            .into_iter()
                            .filter(|(_, view)| view.label().starts_with("tab-"))
                        {
                            let _ = webview.set_position(tauri::LogicalPosition::new(
                                SIDEBAR_WIDTH,
                                CHROME_HEIGHT,
                            ));
                            let _ = webview.set_size(tauri::LogicalSize::new(
                                logical_width.max(1.0),
                                logical_height.max(1.0),
                            ));
                        }
                    }
                });
            }
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<AppState>();
                let _ = open_url(
                    handle.clone(),
                    state,
                    OpenUrlInput {
                        url: "https://libecity.com/".into(),
                        disposition: Some("new-tab".into()),
                    },
                )
                .await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run Libe Desk");
}
