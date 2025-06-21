//! Database CRUD Example
//! Demonstrates real database operations with Layer9

use layer9_core::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use std::rc::Rc;

/// User model for database operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub email: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Model for User {
    const TABLE_NAME: &'static str = "users";
}

/// Post model for database operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Post {
    pub id: Option<i64>,
    pub user_id: i64,
    pub title: String,
    pub content: Option<String>,
    pub published: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Model for Post {
    const TABLE_NAME: &'static str = "posts";
}

/// CRUD operations component
#[derive(Debug, Clone, Default)]
pub struct CrudApp {
    users: Vec<User>,
    posts: Vec<Post>,
    selected_user: Option<User>,
    new_user: User,
    new_post: Post,
    error: Option<String>,
    loading: bool,
}

impl Component for CrudApp {
    fn render(&self) -> Element {
        let mut children = vec![
            Element::Node {
                tag: "h1".to_string(),
                props: Props::default(),
                children: vec![Element::Text("Database CRUD Example".to_string())],
            }
        ];

        // Error display
        if let Some(error) = &self.error {
            let error_props = Props {
                attributes: vec![("style".to_string(), "background: #fee; color: #c00; padding: 10px; margin: 10px 0; border-radius: 4px;".to_string())],
                ..Default::default()
            };
            
            children.push(Element::Node {
                tag: "div".to_string(),
                props: error_props,
                children: vec![Element::Text(error.clone())],
            });
        }

        // Loading indicator
        if self.loading {
            let loading_props = Props {
                attributes: vec![("style".to_string(), "text-align: center; padding: 20px;".to_string())],
                ..Default::default()
            };
            
            children.push(Element::Node {
                tag: "div".to_string(),
                props: loading_props,
                children: vec![Element::Text("Loading...".to_string())],
            });
        }

        // User section
        children.push(self.render_user_section());

        // Post section
        if let Some(user) = &self.selected_user {
            children.push(self.render_post_section(user));
        }

        let root_props = Props {
            class: Some("crud-app".to_string()),
            attributes: vec![("style".to_string(), "padding: 20px; max-width: 1200px; margin: 0 auto;".to_string())],
            ..Default::default()
        };

        Element::Node {
            tag: "div".to_string(),
            props: root_props,
            children,
        }
    }
}

impl CrudApp {
    fn render_user_section(&self) -> Element {
        let section_props = Props {
            class: Some("user-section".to_string()),
            attributes: vec![("style".to_string(), "margin-bottom: 40px;".to_string())],
            ..Default::default()
        };

        let mut children = vec![
            Element::Node {
                tag: "h2".to_string(),
                props: Props::default(),
                children: vec![Element::Text("Users".to_string())],
            }
        ];

        // Create user form
        children.push(self.render_create_user_form());

        // Users list
        children.push(self.render_users_list());

        Element::Node {
            tag: "div".to_string(),
            props: section_props,
            children,
        }
    }

    fn render_create_user_form(&self) -> Element {
        let form_props = Props {
            class: Some("create-user".to_string()),
            attributes: vec![("style".to_string(), "background: #f5f5f5; padding: 15px; border-radius: 4px; margin-bottom: 20px;".to_string())],
            ..Default::default()
        };

        let username_input = {
            let props = Props {
                attributes: vec![
                    ("type".to_string(), "text".to_string()),
                    ("placeholder".to_string(), "Username".to_string()),
                    ("value".to_string(), self.new_user.username.clone()),
                    ("style".to_string(), "margin-right: 10px; padding: 5px;".to_string()),
                ],
                ..Default::default()
            };
            
            Element::Node {
                tag: "input".to_string(),
                props,
                children: vec![],
            }
        };

        let email_input = {
            let props = Props {
                attributes: vec![
                    ("type".to_string(), "email".to_string()),
                    ("placeholder".to_string(), "Email".to_string()),
                    ("value".to_string(), self.new_user.email.clone()),
                    ("style".to_string(), "margin-right: 10px; padding: 5px;".to_string()),
                ],
                ..Default::default()
            };
            
            Element::Node {
                tag: "input".to_string(),
                props,
                children: vec![],
            }
        };

        let create_button = {
            let props = Props {
                attributes: vec![("style".to_string(), "padding: 5px 15px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;".to_string())],
                on_click: Some(Rc::new(|| {
                    web_sys::console::log_1(&"Create user clicked".into());
                })),
                ..Default::default()
            };
            
            Element::Node {
                tag: "button".to_string(),
                props,
                children: vec![Element::Text("Create User".to_string())],
            }
        };

        Element::Node {
            tag: "div".to_string(),
            props: form_props,
            children: vec![
                Element::Node {
                    tag: "h3".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text("Create New User".to_string())],
                },
                username_input,
                email_input,
                create_button,
            ],
        }
    }

    fn render_users_list(&self) -> Element {
        let list_props = Props {
            class: Some("users-list".to_string()),
            attributes: vec![("style".to_string(), "display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 15px;".to_string())],
            ..Default::default()
        };

        let user_cards: Vec<Element> = self.users.iter().map(|user| {
            self.render_user_card(user)
        }).collect();

        Element::Node {
            tag: "div".to_string(),
            props: list_props,
            children: user_cards,
        }
    }

    fn render_user_card(&self, user: &User) -> Element {
        let card_props = Props {
            class: Some("user-card".to_string()),
            attributes: vec![("style".to_string(), "border: 1px solid #ddd; padding: 15px; border-radius: 4px; background: white;".to_string())],
            ..Default::default()
        };

        let view_posts_button = {
            let props = Props {
                attributes: vec![("style".to_string(), "padding: 5px 10px; background: #28a745; color: white; border: none; border-radius: 4px; cursor: pointer; margin-right: 5px;".to_string())],
                ..Default::default()
            };
            
            Element::Node {
                tag: "button".to_string(),
                props,
                children: vec![Element::Text("View Posts".to_string())],
            }
        };

        let delete_button = {
            let props = Props {
                attributes: vec![("style".to_string(), "padding: 5px 10px; background: #dc3545; color: white; border: none; border-radius: 4px; cursor: pointer;".to_string())],
                ..Default::default()
            };
            
            Element::Node {
                tag: "button".to_string(),
                props,
                children: vec![Element::Text("Delete".to_string())],
            }
        };

        Element::Node {
            tag: "div".to_string(),
            props: card_props,
            children: vec![
                Element::Node {
                    tag: "h4".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text(user.username.clone())],
                },
                Element::Node {
                    tag: "p".to_string(),
                    props: Props {
                        attributes: vec![("style".to_string(), "color: #666; margin: 5px 0;".to_string())],
                        ..Default::default()
                    },
                    children: vec![Element::Text(user.email.clone())],
                },
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        attributes: vec![("style".to_string(), "margin-top: 10px;".to_string())],
                        ..Default::default()
                    },
                    children: vec![view_posts_button, delete_button],
                },
            ],
        }
    }

    fn render_post_section(&self, user: &User) -> Element {
        let section_props = Props {
            class: Some("posts-section".to_string()),
            ..Default::default()
        };

        let mut children = vec![
            Element::Node {
                tag: "h2".to_string(),
                props: Props::default(),
                children: vec![Element::Text(format!("Posts by {}", user.username))],
            }
        ];

        // Create post form
        children.push(self.render_create_post_form());

        // Posts list
        children.push(self.render_posts_list());

        Element::Node {
            tag: "div".to_string(),
            props: section_props,
            children,
        }
    }

    fn render_create_post_form(&self) -> Element {
        let form_props = Props {
            class: Some("create-post".to_string()),
            attributes: vec![("style".to_string(), "background: #f5f5f5; padding: 15px; border-radius: 4px; margin-bottom: 20px;".to_string())],
            ..Default::default()
        };

        Element::Node {
            tag: "div".to_string(),
            props: form_props,
            children: vec![
                Element::Node {
                    tag: "h3".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text("Create New Post".to_string())],
                },
            ],
        }
    }

    fn render_posts_list(&self) -> Element {
        let list_props = Props {
            class: Some("posts-list".to_string()),
            ..Default::default()
        };

        let post_cards: Vec<Element> = self.posts.iter().map(|post| {
            self.render_post_card(post)
        }).collect();

        Element::Node {
            tag: "div".to_string(),
            props: list_props,
            children: post_cards,
        }
    }

    fn render_post_card(&self, post: &Post) -> Element {
        let card_props = Props {
            class: Some("post-card".to_string()),
            attributes: vec![("style".to_string(), "border: 1px solid #ddd; padding: 15px; border-radius: 4px; margin-bottom: 10px; background: white;".to_string())],
            ..Default::default()
        };

        let mut children = vec![
            Element::Node {
                tag: "h4".to_string(),
                props: Props::default(),
                children: vec![Element::Text(post.title.clone())],
            }
        ];

        if let Some(content) = &post.content {
            children.push(Element::Node {
                tag: "p".to_string(),
                props: Props {
                    attributes: vec![("style".to_string(), "color: #666; margin: 10px 0;".to_string())],
                    ..Default::default()
                },
                children: vec![Element::Text(content.clone())],
            });
        }

        children.push(Element::Node {
            tag: "p".to_string(),
            props: Props {
                attributes: vec![("style".to_string(), "font-size: 0.9em; color: #999;".to_string())],
                ..Default::default()
            },
            children: vec![Element::Text(if post.published { "Published" } else { "Draft" }.to_string())],
        });

        let delete_button = {
            let props = Props {
                attributes: vec![("style".to_string(), "padding: 5px 10px; background: #dc3545; color: white; border: none; border-radius: 4px; cursor: pointer;".to_string())],
                ..Default::default()
            };
            
            Element::Node {
                tag: "button".to_string(),
                props,
                children: vec![Element::Text("Delete".to_string())],
            }
        };

        children.push(delete_button);

        Element::Node {
            tag: "div".to_string(),
            props: card_props,
            children,
        }
    }

    /// Load all users from the database
    pub fn load_users(&mut self) {
        self.loading = true;
        self.error = None;
        
        wasm_bindgen_futures::spawn_local(async move {
            let repo = use_repository::<User>();
            match repo.find_all().await {
                Ok(users) => {
                    web_sys::console::log_1(&format!("Loaded {} users", users.len()).into());
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load users: {:?}", e).into());
                }
            }
        });
    }
    
    /// Create a new user
    pub fn create_user(&mut self) {
        if self.new_user.username.is_empty() || self.new_user.email.is_empty() {
            self.error = Some("Username and email are required".to_string());
            return;
        }
        
        let user = self.new_user.clone();
        self.loading = true;
        self.error = None;
        
        wasm_bindgen_futures::spawn_local(async move {
            let repo = use_repository::<User>();
            match repo.insert(&user).await {
                Ok(created_user) => {
                    web_sys::console::log_1(&format!("Created user: {:?}", created_user).into());
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to create user: {:?}", e).into());
                }
            }
        });
        
        // Clear form
        self.new_user = User::default();
    }
    
    /// Delete a user
    pub fn delete_user(&mut self, id: i64) {
        self.loading = true;
        self.error = None;
        
        wasm_bindgen_futures::spawn_local(async move {
            let repo = use_repository::<User>();
            match repo.delete(id).await {
                Ok(_) => {
                    web_sys::console::log_1(&format!("Deleted user {}", id).into());
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to delete user: {:?}", e).into());
                }
            }
        });
    }
    
    /// Load posts for a specific user
    pub fn load_user_posts(&mut self, user_id: i64) {
        self.loading = true;
        self.error = None;
        self.posts.clear();
        
        wasm_bindgen_futures::spawn_local(async move {
            let repo = use_repository::<Post>();
            let query = repo.query().where_eq("user_id", user_id);
            
            match query.execute(&use_db()).await {
                Ok(posts) => {
                    web_sys::console::log_1(&format!("Loaded {} posts", posts.len()).into());
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load posts: {:?}", e).into());
                }
            }
        });
    }
    
    /// Create a new post
    pub fn create_post(&mut self) {
        if self.new_post.title.is_empty() {
            self.error = Some("Post title is required".to_string());
            return;
        }
        
        if let Some(user) = &self.selected_user {
            let mut post = self.new_post.clone();
            post.user_id = user.id.unwrap_or(0);
            
            self.loading = true;
            self.error = None;
            
            wasm_bindgen_futures::spawn_local(async move {
                let repo = use_repository::<Post>();
                match repo.insert(&post).await {
                    Ok(created_post) => {
                        web_sys::console::log_1(&format!("Created post: {:?}", created_post).into());
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to create post: {:?}", e).into());
                    }
                }
            });
            
            // Clear form
            self.new_post = Post::default();
        }
    }
    
    /// Delete a post
    pub fn delete_post(&mut self, id: i64) {
        self.loading = true;
        self.error = None;
        
        wasm_bindgen_futures::spawn_local(async move {
            let repo = use_repository::<Post>();
            match repo.delete(id).await {
                Ok(_) => {
                    web_sys::console::log_1(&format!("Deleted post {}", id).into());
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to delete post: {:?}", e).into());
                }
            }
        });
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    // Initialize the app
    init_renderer();
    let app = CrudApp::default();
    mount(Box::new(app), "app");
    queue_current_render();
    
    // Load initial data
    wasm_bindgen_futures::spawn_local(async {
        let repo = use_repository::<User>();
        match repo.find_all().await {
            Ok(users) => {
                web_sys::console::log_1(&format!("Initial load: {} users", users.len()).into());
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to load initial users: {:?}", e).into());
            }
        }
    });
}