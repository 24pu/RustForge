use leptos::*;
use leptos::html::Input;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use uuid::Uuid;

// ---------- 模型 ----------
#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: String,
    email: String,
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ContentItem {
    id: Uuid,
    slug: String,
    title: String,
    body: String,
    published: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct CreateContentRequest {
    slug: String,
    title: String,
    body: String,
}

#[derive(Serialize, Deserialize)]
struct UpdateContentRequest {
    title: String,
    body: String,
    published: Option<bool>,
}

// ---------- API 基础 ----------
async fn api_post<T: Serialize>(path: &str, body: &T) -> Result<String, String> {
    let resp = Request::post(&format!("/api{}", path))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(body).unwrap())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if resp.ok() {
        resp.text().await.map_err(|e| e.to_string())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(format!("{}: {}", status, text))
    }
}

async fn api_get<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T, String> {
    let resp = Request::get(&format!("/api{}", path)).send().await.map_err(|e| e.to_string())?;
    if resp.ok() {
        resp.json().await.map_err(|e| e.to_string())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(format!("{}: {}", status, text))
    }
}

async fn api_delete(path: &str) -> Result<(), String> {
    let resp = Request::delete(&format!("/api{}", path)).send().await.map_err(|e| e.to_string())?;
    if resp.ok() {
        Ok(())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(format!("{}: {}", status, text))
    }
}

// ---------- 全局状态 ----------
#[derive(Clone)]
struct AuthState {
    logged_in: RwSignal<bool>,
    user_email: RwSignal<String>,
}

// ---------- 组件 ----------
#[component]
fn App() -> impl IntoView {
    let (logged_in, set_logged_in) = create_signal(false);
    let (user_email, set_user_email) = create_signal(String::new());
    provide_context(AuthState { logged_in, user_email: set_user_email });

    view! {
        <div class="container mx-auto p-4">
            <Show
                when=move || logged_in.get()
                fallback=move || view! { <Login on_success=move |email| { set_logged_in(true); set_user_email(email); } /> }
            >
                <AdminDashboard on_logout=move || set_logged_in(false) />
            </Show>
        </div>
    }
}

#[component]
fn Login(on_success: impl Fn(String) + 'static) -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(None::<String>);
    let login_action = create_action(move |(email, password): &(String, String)| {
        let email = email.clone();
        let password = password.clone();
        async move {
            let req = LoginRequest { email: email.clone(), password };
            match api_post("/login", &req).await {
                Ok(_) => on_success(email),
                Err(e) => set_error(Some(e)),
            }
        }
    });
    view! {
        <div class="max-w-md mx-auto mt-10">
            <h1 class="text-2xl font-bold mb-4">Login</h1>
            <form on:submit=move |ev| {
                ev.prevent_default();
                login_action.dispatch((email.get(), password.get()));
            }>
                <input class="border p-2 w-full mb-2" type="email" placeholder="Email"
                    on:input=move |ev| set_email(event_target_value(&ev)) />
                <input class="border p-2 w-full mb-2" type="password" placeholder="Password"
                    on:input=move |ev| set_password(event_target_value(&ev)) />
                <button type="submit" class="bg-blue-500 text-white p-2 rounded">Login</button>
            </form>
            {move || if let Some(err) = error.get() {
                view! { <p class="text-red-500 mt-2">{err}</p> }.into_view()
            } else {
                view! {}.into_view()
            }}
        </div>
    }
}

#[component]
fn AdminDashboard(on_logout: impl Fn() + 'static) -> impl IntoView {
    let (contents, set_contents) = create_signal(Vec::<ContentItem>::new());
    let load_contents = create_action(move |_| {
        async move {
            let items: Vec<ContentItem> = api_get("/admin/contents").await.unwrap_or_default();
            set_contents(items);
        }
    });
    create_effect(move |_| {
        load_contents.dispatch(());
    });
    let delete_content = create_action(move |id: &Uuid| {
        let id = *id;
        async move {
            let _ = api_delete(&format!("/admin/contents/{}", id)).await;
            load_contents.dispatch(());
        }
    });
    view! {
        <div>
            <div class="flex justify-between items-center mb-4">
                <h1 class="text-2xl font-bold">Admin Dashboard</h1>
                <button on:click=move |_| on_logout() class="bg-red-500 text-white p-2 rounded">Logout</button>
            </div>
            <div class="mb-6">
                <CreateContent on_created=move || { load_contents.dispatch(()); } />
            </div>
            <div>
                <h2 class="text-xl font-semibold mb-2">Contents</h2>
                <ul>
                    {move || contents.get().iter().map(|c| view! {
                        <li class="border p-2 mb-2">
                            <div><strong>{&c.title}</strong> (slug: {&c.slug})</div>
                            <p>{&c.body}</p>
                            <button on:click=move |_| { delete_content.dispatch(c.id); } class="bg-red-500 text-white p-1 rounded">Delete</button>
                        </li>
                    }).collect_view()}
                </ul>
            </div>
        </div>
    }
}

#[component]
fn CreateContent(on_created: impl Fn() + 'static) -> impl IntoView {
    let (slug, set_slug) = create_signal(String::new());
    let (title, set_title) = create_signal(String::new());
    let (body, set_body) = create_signal(String::new());
    let create_action = create_action(move |_| {
        let slug = slug.get();
        let title = title.get();
        let body = body.get();
        async move {
            let req = CreateContentRequest { slug, title, body };
            let _ = api_post("/admin/contents", &req).await;
            on_created();
            set_slug.set("".to_string());
            set_title.set("".to_string());
            set_body.set("".to_string());
        }
    });
    view! {
        <div class="border p-4 rounded">
            <h3 class="text-lg font-bold mb-2">Create Content</h3>
            <input class="border p-2 w-full mb-2" placeholder="Slug" on:input=move |ev| set_slug(event_target_value(&ev)) />
            <input class="border p-2 w-full mb-2" placeholder="Title" on:input=move |ev| set_title(event_target_value(&ev)) />
            <textarea class="border p-2 w-full mb-2" placeholder="Body" rows=3 on:input=move |ev| set_body(event_target_value(&ev)) />
            <button on:click=move |_| create_action.dispatch(()) class="bg-green-500 text-white p-2 rounded">Create</button>
        </div>
    }
}

// 启动
pub fn main() {
    mount_to_body(|| view! { <App /> })
}