//! Production-Ready WARP Application Example
//! Demonstrates all production features in action

use warp::prelude::*;
use std::collections::HashMap;

#[warp_app]
struct ProductionApp {
    title: String,
}

impl WarpApp for ProductionApp {
    fn init() -> Self {
        // Initialize i18n
        init_i18n(
            TranslationCatalog::new()
                .add_locale(Locale::EnUS, TranslationsBuilder::en_us())
                .add_locale(Locale::KoKR, TranslationsBuilder::ko_kr())
        );
        
        // Initialize monitoring
        init_monitoring();
        
        // Initialize router
        init_router();
        
        ProductionApp {
            title: "Production WARP App".to_string(),
        }
    }
    
    fn routes(&self) -> Vec<Route> {
        vec![
            route("/", home_page),
            route("/dashboard", dashboard_page),
            route("/api/posts", api_posts),
            route("/api/metrics", api_metrics),
        ]
    }
}

/// Home page with i18n, auth, and monitoring
#[page("/")]
async fn home_page() -> Element {
    let i18n = use_i18n();
    let auth = use_auth();
    let analytics = use_analytics();
    let perf = use_performance();
    
    // Track page view
    analytics.track_page_view("/", Some("Home"));
    
    // Start performance timing
    let _timing = perf.start_timing("home_page_render");
    
    view! {
        <Layout>
            <Header />
            
            <main class={style![max_w_7xl, mx_auto, px_4]}>
                <h1 class={style![text_4xl, font_bold, mb_8]}>
                    {t!("app.title")}
                </h1>
                
                <LanguageSelector />
                
                {if auth.is_authenticated() {
                    view! {
                        <div>
                            <p>{t!("welcome.message", name = auth.user().name)}</p>
                            <Link href="/dashboard">
                                <Button>{t!("dashboard.link")}</Button>
                            </Link>
                        </div>
                    }
                } else {
                    view! {
                        <LoginForm />
                    }
                }}
            </main>
            
            <Footer />
        </Layout>
    }
}

/// Dashboard with data fetching and caching
#[page("/dashboard")]
#[protected]
async fn dashboard_page() -> Element {
    let cache = use_http_cache();
    let posts_repo = use_repository::<Post>();
    let error_tracker = use_error_tracker();
    
    // Fetch posts with caching
    let posts = match cache.fetch("/api/posts", FetchOptions::default()).await {
        Ok(response) => {
            serde_json::from_str::<Vec<Post>>(&response.text()).unwrap_or_default()
        }
        Err(e) => {
            error_tracker.track_error(&JsValue::from_str(&e));
            vec![]
        }
    };
    
    view! {
        <Layout>
            <DashboardHeader />
            
            <ErrorBoundary fallback={|error| view! {
                <Alert type="error">{error.message}</Alert>
            }}>
                <div class={style![grid, grid_cols_1, md_grid_cols_2, lg_grid_cols_3, gap_6]}>
                    {posts.iter().map(|post| view! {
                        <PostCard post={post.clone()} />
                    }).collect::<Vec<_>>()}
                </div>
            </ErrorBoundary>
            
            <CreatePostForm />
        </Layout>
    }
}

/// Post card component with image optimization
#[component]
fn PostCard(post: Post) -> Element {
    let analytics = use_analytics();
    
    view! {
        <article class={style![bg_white, rounded_lg, shadow_md, overflow_hidden]}>
            {if let Some(image_url) = &post.image_url {
                view! {
                    <Image
                        src={image_url}
                        alt={&post.title}
                        width={400}
                        height={300}
                        loading={ImageLoading::Lazy}
                        placeholder={ImagePlaceholder::Blur("data:image/jpeg;base64,...")}
                    />
                }
            } else {
                view! { <div /> }
            }}
            
            <div class={style![p_6]}>
                <h2 class={style![text_xl, font_semibold, mb_2]}>{&post.title}</h2>
                <p class={style![text_gray_600, mb_4]}>{&post.excerpt}</p>
                
                <Button
                    onclick={move || {
                        analytics.track_click("read_more", Some(&post.title));
                        navigate(&format!("/posts/{}", post.id));
                    }}
                >
                    {t!("post.read_more")}
                </Button>
            </div>
        </article>
    }
}

/// Create post form with validation and CSRF
#[component]
fn CreatePostForm() -> Element {
    let form = use_form(FormConfig {
        validation: |data: &PostFormData| {
            let mut errors = HashMap::new();
            
            if data.title.is_empty() {
                errors.insert("title".to_string(), t!("validation.required"));
            }
            
            if data.content.len() < 10 {
                errors.insert("content".to_string(), t!("validation.min_length", length = 10));
            }
            
            errors
        },
        on_submit: |data| async move {
            create_post(data).await
        },
    });
    
    let csrf_token = use_csrf_token();
    let upload = use_upload(UploadConfig {
        accept: vec!["image/*"],
        max_size: 5 * 1024 * 1024, // 5MB
        on_progress: |progress| {
            web_sys::console::log_1(&format!("Upload progress: {}%", progress).into());
        },
    });
    
    view! {
        <form onsubmit={form.handle_submit}>
            <input type="hidden" name="csrf_token" value={csrf_token} />
            
            <FormField
                label={t!("post.title")}
                error={form.errors.get("title")}
            >
                <Input
                    value={form.values.title}
                    onchange={|e| form.set_field("title", e.target_value())}
                    placeholder={t!("post.title_placeholder")}
                />
            </FormField>
            
            <FormField
                label={t!("post.content")}
                error={form.errors.get("content")}
            >
                <Textarea
                    value={form.values.content}
                    onchange={|e| form.set_field("content", e.target_value())}
                    rows={10}
                />
            </FormField>
            
            <FormField label={t!("post.image")}>
                <FileUpload
                    config={upload}
                    onupload={|files| {
                        form.set_field("image", files[0].url.clone());
                    }}
                />
            </FormField>
            
            <Button
                type="submit"
                loading={form.is_submitting}
                disabled={!form.is_valid}
            >
                {t!("post.create")}
            </Button>
        </form>
    }
}

/// API endpoint with monitoring and security
#[server]
async fn create_post(data: PostFormData) -> Result<Post, ApiError> {
    let security = use_security();
    let metrics = use_metrics();
    let span = TraceSpan::new("create_post", metrics.clone());
    
    // Verify CSRF token
    if !security.verify_csrf(&data.csrf_token) {
        span.set_attribute("error", "invalid_csrf");
        return Err(ApiError::Unauthorized);
    }
    
    // Sanitize input
    let sanitized_content = XssProtection::sanitize_html(&data.content);
    
    // Create post in database
    let db = use_db();
    let post = db.transaction(|tx| async {
        let post = Post {
            id: generate_id(),
            title: data.title,
            content: sanitized_content,
            excerpt: data.content.chars().take(200).collect(),
            image_url: data.image,
            author_id: get_current_user_id()?,
            created_at: current_timestamp(),
        };
        
        tx.execute(
            "INSERT INTO posts (id, title, content, excerpt, image_url, author_id, created_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            vec![
                post.id.clone().into(),
                post.title.clone().into(),
                post.content.clone().into(),
                post.excerpt.clone().into(),
                post.image_url.clone().into(),
                post.author_id.clone().into(),
                post.created_at.into(),
            ]
        ).await?;
        
        Ok(post)
    }).await?;
    
    // Track metrics
    metrics.counter("posts.created", 1, {
        let mut tags = HashMap::new();
        tags.insert("author_id".to_string(), post.author_id.clone());
        tags
    });
    
    // Invalidate cache
    let cache = use_cache();
    cache.invalidate_by_tags(&["posts"]);
    
    span.end();
    Ok(post)
}

/// API endpoint for posts with caching
#[server]
async fn api_posts() -> Result<Vec<Post>, ApiError> {
    let cache = use_cache();
    let posts = cache.get_or_compute_async("all_posts", || async {
        let repo = use_repository::<Post>();
        repo.query()
            .order_by("created_at", "DESC")
            .limit(20)
            .execute(&use_db())
            .await
            .unwrap_or_default()
    }).await;
    
    Ok(posts)
}

/// Metrics endpoint
#[server]
async fn api_metrics() -> Result<MetricsResponse, ApiError> {
    let metrics = use_metrics();
    
    Ok(MetricsResponse {
        uptime: get_uptime(),
        requests_total: get_metric("http.requests.total"),
        errors_total: get_metric("errors.total"),
        active_users: get_metric("users.active"),
    })
}

/// WebSocket handler for real-time updates
fn setup_websocket() {
    let ws = use_websocket("ws://localhost:3001/ws")?;
    
    ws.on_message(|msg| {
        match msg {
            WsMessage::Text(text) => {
                if let Ok(update) = serde_json::from_str::<PostUpdate>(&text) {
                    // Update UI with new post
                    handle_post_update(update);
                }
            }
            WsMessage::Binary(_) => {}
        }
    });
    
    // Subscribe to post updates
    ws.send_json(&json!({
        "type": "subscribe",
        "channel": "posts"
    }))?;
}

// Data models
#[derive(Clone, Serialize, Deserialize)]
struct Post {
    id: String,
    title: String,
    content: String,
    excerpt: String,
    image_url: Option<String>,
    author_id: String,
    created_at: f64,
}

impl Model for Post {
    const TABLE_NAME: &'static str = "posts";
}

#[derive(Serialize, Deserialize)]
struct PostFormData {
    title: String,
    content: String,
    image: Option<String>,
    csrf_token: String,
}

#[derive(Serialize, Deserialize)]
struct PostUpdate {
    post_id: String,
    update_type: String,
}

#[derive(Serialize)]
struct MetricsResponse {
    uptime: f64,
    requests_total: u64,
    errors_total: u64,
    active_users: u64,
}

// Run the app
fn main() {
    // Set panic hook for error tracking
    std::panic::set_hook(Box::new(|info| {
        if let Some(error_tracker) = get_error_tracker() {
            error_tracker.track_panic(info);
        }
    }));
    
    // Run migrations
    wasm_bindgen_futures::spawn_local(async {
        let migrator = Migrator::new(use_db())
            .add_migration(Migration {
                version: 1,
                name: "create_posts_table".to_string(),
                up: include_str!("../migrations/001_create_posts.sql").to_string(),
                down: include_str!("../migrations/001_create_posts_down.sql").to_string(),
            });
        
        migrator.migrate().await.expect("Failed to run migrations");
    });
    
    // Setup WebSocket
    setup_websocket();
    
    // Run app
    run_app::<ProductionApp>();
}