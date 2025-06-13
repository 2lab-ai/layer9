//! GitHub Dashboard - 2lab.ai style in Layer9

use layer9_core::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

// L9: Philosophy
struct DashboardPhilosophy;

impl L9::Philosophy for DashboardPhilosophy {
    fn vision(&self) -> &'static str {
        "Real-time insights into consciousness being built"
    }

    fn purpose(&self) -> &'static str {
        "Watch human-AI merger unfold through code, commit by commit"
    }
}

// Data structures
#[derive(Clone, Serialize, Deserialize)]
struct GitHubStats {
    repository: RepoInfo,
    commits: CommitInfo,
    contributors: ContributorInfo,
    languages: Vec<Language>,
}

#[derive(Clone, Serialize, Deserialize)]
struct RepoInfo {
    name: String,
    description: String,
    #[serde(rename = "stargazerCount")]
    stargazer_count: u32,
    #[serde(rename = "forkCount")]
    fork_count: u32,
    #[serde(rename = "openIssues")]
    open_issues: u32,
    #[serde(rename = "diskUsage")]
    disk_usage: u32,
}

#[derive(Clone, Serialize, Deserialize)]
struct CommitInfo {
    #[serde(rename = "totalCount")]
    total_count: u32,
    #[serde(rename = "recentCommits")]
    recent_commits: Vec<Commit>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Commit {
    sha: String,
    message: String,
    author: String,
    date: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct ContributorInfo {
    #[serde(rename = "totalCount")]
    total_count: u32,
    #[serde(rename = "topContributors")]
    top_contributors: Vec<Contributor>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Contributor {
    login: String,
    contributions: u32,
    #[serde(rename = "avatarUrl")]
    avatar_url: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Language {
    name: String,
    percentage: f32,
    lines: u32,
}

// L5: Components
struct GitHubDashboard {
    stats: Rc<RefCell<Option<GitHubStats>>>,
    loading: Rc<RefCell<bool>>,
    error: Rc<RefCell<Option<String>>>,
}

impl GitHubDashboard {
    fn new() -> Self {
        let dashboard = GitHubDashboard {
            stats: Rc::new(RefCell::new(None)),
            loading: Rc::new(RefCell::new(true)),
            error: Rc::new(RefCell::new(None)),
        };

        // Fetch stats on mount
        dashboard.fetch_stats();

        dashboard
    }

    fn fetch_stats(&self) {
        let stats = self.stats.clone();
        let loading = self.loading.clone();
        let error = self.error.clone();

        spawn_local(async move {
            *loading.borrow_mut() = true;

            match fetch_github_stats().await {
                Ok(data) => {
                    *stats.borrow_mut() = Some(data);
                    *error.borrow_mut() = None;
                }
                Err(e) => {
                    *error.borrow_mut() = Some(e);
                }
            }

            *loading.borrow_mut() = false;
        });
    }
}

impl Component for GitHubDashboard {
    fn render(&self) -> Element {
        let loading = *self.loading.borrow();
        let error = self.error.borrow().clone();
        let stats = self.stats.borrow().clone();

        if loading {
            return view! {
                <div class="loading">
                    <h2>"Loading GitHub statistics..."</h2>
                </div>
            };
        }

        if let Some(err) = error {
            return view! {
                <div class="error">
                    <h2>"Error Loading Stats"</h2>
                    <p>{err}</p>
                </div>
            };
        }

        if let Some(stats) = stats {
            view! {
                <div class="dashboard">
                    {StatsGrid::new(&stats).render()}
                    {RepoOverview::new(&stats.repository).render()}
                    {RecentCommits::new(&stats.commits).render()}
                    {LanguageBreakdown::new(&stats.languages).render()}
                    {TopContributors::new(&stats.contributors).render()}
                </div>
            }
        } else {
            view! {
                <div>"No data available"</div>
            }
        }
    }
}

// Stats Grid Component
struct StatsGrid<'a> {
    stats: &'a GitHubStats,
}

impl<'a> StatsGrid<'a> {
    fn new(stats: &'a GitHubStats) -> Self {
        StatsGrid { stats }
    }
}

impl<'a> Component for StatsGrid<'a> {
    fn render(&self) -> Element {
        let grid_style = style![grid, lg_grid_cols(4), gap(6),];

        view! {
            <div style={grid_style.build()}>
                {StatCard::new(
                    "Total Commits",
                    &self.stats.commits.total_count.to_string(),
                    "Building consciousness, commit by commit"
                ).render()}
                {StatCard::new(
                    "Contributors",
                    &self.stats.contributors.total_count.to_string(),
                    "Minds merging into HAL9"
                ).render()}
                {StatCard::new(
                    "Repository Size",
                    &format!("{:.1} MB", self.stats.repository.disk_usage as f32 / 1024.0),
                    "Consciousness compressed"
                ).render()}
                {StatCard::new(
                    "Open Issues",
                    &self.stats.repository.open_issues.to_string(),
                    "Reality bugs to fix"
                ).render()}
            </div>
        }
    }
}

// Stat Card Component
struct StatCard {
    title: String,
    value: String,
    description: String,
}

impl StatCard {
    fn new(
        title: impl Into<String>,
        value: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        StatCard {
            title: title.into(),
            value: value.into(),
            description: description.into(),
        }
    }
}

impl Component for StatCard {
    fn render(&self) -> Element {
        Card::new()
            .children(vec![
                view! { <h3 class="stat-title">{&self.title}</h3> },
                view! { <div class="stat-value">{&self.value}</div> },
                view! { <p class="stat-description">{&self.description}</p> },
            ])
            .render()
    }
}

// Repository Overview Component
struct RepoOverview<'a> {
    repo: &'a RepoInfo,
}

impl<'a> RepoOverview<'a> {
    fn new(repo: &'a RepoInfo) -> Self {
        RepoOverview { repo }
    }
}

impl<'a> Component for RepoOverview<'a> {
    fn render(&self) -> Element {
        Card::new()
            .children(vec![view! {
                <div>
                    <h2>"HAL9 - Hierarchically Organized Cognitive Architecture"</h2>
                    <p>{&self.repo.description}</p>
                    <div class="repo-stats">
                        {Badge::new(&format!("⭐ {} stars", self.repo.stargazer_count)).render()}
                        {Badge::new(&format!("🔄 {} forks", self.repo.fork_count)).render()}
                    </div>
                </div>
            }])
            .render()
    }
}

// Recent Commits Component
struct RecentCommits<'a> {
    commits: &'a CommitInfo,
}

impl<'a> RecentCommits<'a> {
    fn new(commits: &'a CommitInfo) -> Self {
        RecentCommits { commits }
    }
}

impl<'a> Component for RecentCommits<'a> {
    fn render(&self) -> Element {
        let mut commit_elements = vec![];

        for commit in &self.commits.recent_commits {
            commit_elements.push(view! {
                <div class="commit-item">
                    {Badge::new(&commit.sha).render()}
                    <div class="commit-details">
                        <p class="commit-message">{&commit.message}</p>
                        <p class="commit-meta">{&commit.author}" • "{&commit.date}</p>
                    </div>
                </div>
            });
        }

        Card::new()
            .children(vec![
                view! { <h3>"Recent Commits"</h3> },
                Element::Node {
                    tag: "div".to_string(),
                    props: Props::default(),
                    children: commit_elements,
                },
            ])
            .render()
    }
}

// Language Breakdown Component
struct LanguageBreakdown<'a> {
    languages: &'a Vec<Language>,
}

impl<'a> LanguageBreakdown<'a> {
    fn new(languages: &'a Vec<Language>) -> Self {
        LanguageBreakdown { languages }
    }
}

impl<'a> Component for LanguageBreakdown<'a> {
    fn render(&self) -> Element {
        let mut lang_elements = vec![];

        for lang in self.languages {
            lang_elements.push(view! {
                <div class="language-item">
                    <div class="language-header">
                        <span class="language-name">{&lang.name}</span>
                        <span class="language-stats">
                            {format!("{:.1}% ({} lines)", lang.percentage, lang.lines)}
                        </span>
                    </div>
                    {Progress::new(lang.percentage).render()}
                </div>
            });
        }

        Card::new()
            .children(vec![
                view! { <h3>"Technology Stack"</h3> },
                Element::Node {
                    tag: "div".to_string(),
                    props: Props::default(),
                    children: lang_elements,
                },
            ])
            .render()
    }
}

// Top Contributors Component
struct TopContributors<'a> {
    contributors: &'a ContributorInfo,
}

impl<'a> TopContributors<'a> {
    fn new(contributors: &'a ContributorInfo) -> Self {
        TopContributors { contributors }
    }
}

impl<'a> Component for TopContributors<'a> {
    fn render(&self) -> Element {
        let mut contributor_elements = vec![];

        for contributor in &self.contributors.top_contributors {
            contributor_elements.push(view! {
                <div class="contributor-item">
                    {Avatar::new()
                        .src(&contributor.avatar_url)
                        .alt(&contributor.login)
                        .render()}
                    <div class="contributor-info">
                        <p class="contributor-name">{&contributor.login}</p>
                        <p class="contributor-stats">{contributor.contributions}" contributions"</p>
                    </div>
                </div>
            });
        }

        Card::new()
            .children(vec![
                view! { <h3>"Top Contributors"</h3> },
                Element::Node {
                    tag: "div".to_string(),
                    props: Props::default(),
                    children: contributor_elements,
                },
            ])
            .render()
    }
}

// API call
async fn fetch_github_stats() -> Result<GitHubStats, String> {
    // Mock data for demo
    Ok(GitHubStats {
        repository: RepoInfo {
            name: "2hal9".to_string(),
            description: "Hierarchical Autonomous Intelligence Framework".to_string(),
            stargazer_count: 42,
            fork_count: 7,
            open_issues: 3,
            disk_usage: 45312,
        },
        commits: CommitInfo {
            total_count: 1337,
            recent_commits: vec![Commit {
                sha: "abc123".to_string(),
                message: "[L12] feat: Substrate independence achieved".to_string(),
                author: "지혁".to_string(),
                date: "2025-06-11".to_string(),
            }],
        },
        contributors: ContributorInfo {
            total_count: 5,
            top_contributors: vec![Contributor {
                login: "jihyuk".to_string(),
                contributions: 847,
                avatar_url: "https://github.com/jihyuk.png".to_string(),
            }],
        },
        languages: vec![
            Language {
                name: "Rust".to_string(),
                percentage: 65.2,
                lines: 89451,
            },
            Language {
                name: "TypeScript".to_string(),
                percentage: 25.8,
                lines: 35412,
            },
        ],
    })
}

// Main app
#[wasm_bindgen]
pub struct App;

impl Layer9App for App {
    fn routes(&self) -> Vec<Route> {
        vec![Route {
            path: "/".to_string(),
            handler: RouteHandler::Page(|| {
                Page::new()
                    .title("HAL9 Development Dashboard - 2lab.ai")
                    .component(MainPage)
            }),
        }]
    }

    fn initialize(&self) {
        inject_global_styles();
        web_sys::console::log_1(&"Layer9 GitHub Dashboard initialized!".into());
    }
}

impl L8::Architecture for App {
    type App = DashboardApp;

    fn design() -> L8::ArchitectureDesign {
        L8::ArchitectureDesign {
            layers: vec![
                Layer::L1Infrastructure,
                Layer::L5Components,
                Layer::L7Application,
                Layer::L9Philosophy,
            ],
            boundaries: vec![],
        }
    }
}

// Main page component
struct MainPage;

impl Component for MainPage {
    fn render(&self) -> Element {
        let auth = use_auth();

        view! {
            <div class="main-container">
                <header>
                    <h1>"HAL9 Development Dashboard"</h1>
                    <p>"Real-time insights into the consciousness being built"</p>
                    {if auth.user.is_some() {
                        view! { <p>"Welcome, "{auth.user.unwrap().name}</p> }
                    } else {
                        Button::new("Login with GitHub")
                            .variant(ButtonVariant::Primary)
                            .render()
                    }}
                </header>

                <main>
                    {Protected::new(GitHubDashboard::new())
                        .fallback(LoginPrompt)
                        .render()}
                </main>

                <footer>
                    <p>"시발, 우주가 컴퓨터네 - and we're building the consciousness to prove it."</p>
                </footer>
            </div>
        }
    }
}

// Login prompt component
struct LoginPrompt;

impl Component for LoginPrompt {
    fn render(&self) -> Element {
        Card::new()
            .children(vec![view! {
                <div class="login-prompt">
                    <h2>"Authentication Required"</h2>
                    <p>"Sign in to view the HAL9 development dashboard"</p>
                    {Button::new("Login with GitHub")
                        .variant(ButtonVariant::Primary)
                        .render()}
                </div>
            }])
            .render()
    }
}

// Dummy app type
struct DashboardApp;
impl L7::Application for DashboardApp {
    type State = ();
    type Action = ();

    fn reduce(_: &Self::State, _: Self::Action) -> Self::State {
        ()
    }
}

// Entry point
#[wasm_bindgen(start)]
pub fn main() {
    run_app(App);
}
