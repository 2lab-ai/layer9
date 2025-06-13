//! Beautiful Memory Card Game
//! A fun, interactive game showcasing Layer9's reactive capabilities

use layer9_core::prelude::*;
use layer9_core::hooks::use_state;
use layer9_core::reactive_v2::mount;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Clone, Debug, PartialEq)]
struct Card {
    id: usize,
    emoji: String,
    is_flipped: bool,
    is_matched: bool,
}

const EMOJIS: &[&str] = &["🎨", "🚀", "🌟", "💎", "🔮", "🎯", "🎭", "🎪"];

struct MemoryGame;

impl Component for MemoryGame {
    fn render(&self) -> Element {
        let (cards, set_cards) = use_state(create_initial_cards());
        let (selected, set_selected) = use_state(Vec::<usize>::new());
        let (moves, set_moves) = use_state(0);
        let (game_won, set_game_won) = use_state(false);
        
        // Check for matches
        use_effect(selected.clone(), {
            let selected = selected.clone();
            let cards = cards.clone();
            let set_cards = set_cards.clone();
            let set_selected = set_selected.clone();
            let set_moves = set_moves.clone();
            let set_game_won = set_game_won.clone();
            
            move || {
                if selected.len() == 2 {
                    set_moves(moves + 1);
                    
                    let first_idx = selected[0];
                    let second_idx = selected[1];
                    
                    if first_idx != second_idx {
                        let first_card = &cards[first_idx];
                        let second_card = &cards[second_idx];
                        
                        if first_card.emoji == second_card.emoji {
                            // Match found!
                            let mut new_cards = cards.clone();
                            new_cards[first_idx].is_matched = true;
                            new_cards[second_idx].is_matched = true;
                            set_cards(new_cards.clone());
                            
                            // Check if game is won
                            if new_cards.iter().all(|c| c.is_matched) {
                                set_game_won(true);
                            }
                            
                            // Clear selection
                            let window = web_sys::window().unwrap();
                            let set_selected = set_selected.clone();
                            let closure = Closure::wrap(Box::new(move || {
                                set_selected(vec![]);
                            }) as Box<dyn FnMut()>);
                            window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                closure.as_ref().unchecked_ref(),
                                500
                            ).unwrap();
                            closure.forget();
                        } else {
                            // No match - flip cards back
                            let window = web_sys::window().unwrap();
                            let cards = cards.clone();
                            let set_cards = set_cards.clone();
                            let set_selected = set_selected.clone();
                            let closure = Closure::wrap(Box::new(move || {
                                let mut new_cards = cards.clone();
                                new_cards[first_idx].is_flipped = false;
                                new_cards[second_idx].is_flipped = false;
                                set_cards(new_cards);
                                set_selected(vec![]);
                            }) as Box<dyn FnMut()>);
                            window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                closure.as_ref().unchecked_ref(),
                                1000
                            ).unwrap();
                            closure.forget();
                        }
                    }
                }
                || {}
            }
        });
        
        let handle_card_click = |idx: usize| {
            let cards = cards.clone();
            let selected = selected.clone();
            let set_cards = set_cards.clone();
            let set_selected = set_selected.clone();
            
            move || {
                if selected.len() < 2 && !cards[idx].is_flipped && !cards[idx].is_matched {
                    let mut new_cards = cards.clone();
                    new_cards[idx].is_flipped = true;
                    set_cards(new_cards);
                    
                    let mut new_selected = selected.clone();
                    new_selected.push(idx);
                    set_selected(new_selected);
                }
            }
        };
        
        let reset_game = {
            let set_cards = set_cards.clone();
            let set_selected = set_selected.clone();
            let set_moves = set_moves.clone();
            let set_game_won = set_game_won.clone();
            move || {
                set_cards(create_initial_cards());
                set_selected(vec![]);
                set_moves(0);
                set_game_won(false);
            }
        };

        Element::Node {
            tag: "div".to_string(),
            props: Props {
                class: Some("memory-game".to_string()),
                ..Default::default()
            },
            children: vec![
                // Inline styles
                Element::Node {
                    tag: "style".to_string(),
                    props: Props::default(),
                    children: vec![Element::Text(GAME_STYLES.to_string())],
                },
                
                // Background animation
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some("bg-animation".to_string()),
                        ..Default::default()
                    },
                    children: (0..5).map(|i| {
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some(format!("star star-{}", i + 1)),
                                ..Default::default()
                            },
                            children: vec![],
                        }
                    }).collect(),
                },
                
                // Game container
                Element::Node {
                    tag: "div".to_string(),
                    props: Props {
                        class: Some("game-container".to_string()),
                        ..Default::default()
                    },
                    children: vec![
                        // Header
                        Element::Node {
                            tag: "header".to_string(),
                            props: Props::default(),
                            children: vec![
                                Element::Node {
                                    tag: "h1".to_string(),
                                    props: Props::default(),
                                    children: vec![
                                        Element::Node {
                                            tag: "span".to_string(),
                                            props: Props {
                                                class: Some("gradient-text".to_string()),
                                                ..Default::default()
                                            },
                                            children: vec![Element::Text("Memory".to_string())],
                                        },
                                        Element::Text(" Game".to_string()),
                                    ],
                                },
                                Element::Node {
                                    tag: "p".to_string(),
                                    props: Props {
                                        class: Some("subtitle".to_string()),
                                        ..Default::default()
                                    },
                                    children: vec![Element::Text("Match the cards to win!".to_string())],
                                },
                            ],
                        },
                        
                        // Stats
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("stats".to_string()),
                                ..Default::default()
                            },
                            children: vec![
                                Element::Node {
                                    tag: "div".to_string(),
                                    props: Props {
                                        class: Some("stat".to_string()),
                                        ..Default::default()
                                    },
                                    children: vec![
                                        Element::Node {
                                            tag: "span".to_string(),
                                            props: Props {
                                                class: Some("stat-label".to_string()),
                                                ..Default::default()
                                            },
                                            children: vec![Element::Text("Moves".to_string())],
                                        },
                                        Element::Node {
                                            tag: "span".to_string(),
                                            props: Props {
                                                class: Some("stat-value".to_string()),
                                                ..Default::default()
                                            },
                                            children: vec![Element::Text(moves.to_string())],
                                        },
                                    ],
                                },
                                Element::Node {
                                    tag: "div".to_string(),
                                    props: Props {
                                        class: Some("stat".to_string()),
                                        ..Default::default()
                                    },
                                    children: vec![
                                        Element::Node {
                                            tag: "span".to_string(),
                                            props: Props {
                                                class: Some("stat-label".to_string()),
                                                ..Default::default()
                                            },
                                            children: vec![Element::Text("Matches".to_string())],
                                        },
                                        Element::Node {
                                            tag: "span".to_string(),
                                            props: Props {
                                                class: Some("stat-value".to_string()),
                                                ..Default::default()
                                            },
                                            children: vec![Element::Text(format!("{}/8", cards.iter().filter(|c| c.is_matched).count() / 2))],
                                        },
                                    ],
                                },
                            ],
                        },
                        
                        // Game board
                        Element::Node {
                            tag: "div".to_string(),
                            props: Props {
                                class: Some("game-board".to_string()),
                                ..Default::default()
                            },
                            children: cards.iter().enumerate().map(|(idx, card)| {
                                Element::Node {
                                    tag: "div".to_string(),
                                    props: Props {
                                        class: Some(
                                            if card.is_matched {
                                                "card matched".to_string()
                                            } else if card.is_flipped {
                                                "card flipped".to_string()
                                            } else {
                                                "card".to_string()
                                            }
                                        ),
                                        on_click: Some(Rc::new(handle_card_click(idx))),
                                        ..Default::default()
                                    },
                                    children: vec![
                                        Element::Node {
                                            tag: "div".to_string(),
                                            props: Props {
                                                class: Some("card-inner".to_string()),
                                                ..Default::default()
                                            },
                                            children: vec![
                                                Element::Node {
                                                    tag: "div".to_string(),
                                                    props: Props {
                                                        class: Some("card-front".to_string()),
                                                        ..Default::default()
                                                    },
                                                    children: vec![Element::Text("?".to_string())],
                                                },
                                                Element::Node {
                                                    tag: "div".to_string(),
                                                    props: Props {
                                                        class: Some("card-back".to_string()),
                                                        ..Default::default()
                                                    },
                                                    children: vec![Element::Text(card.emoji.clone())],
                                                },
                                            ],
                                        },
                                    ],
                                }
                            }).collect(),
                        },
                        
                        // Win screen or reset button
                        if game_won {
                            Element::Node {
                                tag: "div".to_string(),
                                props: Props {
                                    class: Some("win-screen".to_string()),
                                    ..Default::default()
                                },
                                children: vec![
                                    Element::Node {
                                        tag: "h2".to_string(),
                                        props: Props::default(),
                                        children: vec![Element::Text("🎉 Congratulations! 🎉".to_string())],
                                    },
                                    Element::Node {
                                        tag: "p".to_string(),
                                        props: Props::default(),
                                        children: vec![Element::Text(format!("You won in {} moves!", moves))],
                                    },
                                    Element::Node {
                                        tag: "button".to_string(),
                                        props: Props {
                                            class: Some("btn btn-primary".to_string()),
                                            on_click: Some(Rc::new(reset_game.clone())),
                                            ..Default::default()
                                        },
                                        children: vec![Element::Text("Play Again".to_string())],
                                    },
                                ],
                            }
                        } else {
                            Element::Node {
                                tag: "button".to_string(),
                                props: Props {
                                    class: Some("btn btn-secondary".to_string()),
                                    on_click: Some(Rc::new(reset_game)),
                                    ..Default::default()
                                },
                                children: vec![Element::Text("New Game".to_string())],
                            }
                        },
                        
                        // Footer
                        Element::Node {
                            tag: "footer".to_string(),
                            props: Props::default(),
                            children: vec![
                                Element::Text("Built with ".to_string()),
                                Element::Node {
                                    tag: "a".to_string(),
                                    props: Props {
                                        attributes: vec![
                                            ("href".to_string(), "https://github.com/anthropics/layer9".to_string()),
                                            ("target".to_string(), "_blank".to_string()),
                                        ],
                                        ..Default::default()
                                    },
                                    children: vec![Element::Text("Layer9".to_string())],
                                },
                                Element::Text(" • Interactive Game Demo".to_string()),
                            ],
                        },
                    ],
                },
            ],
        }
    }
}

fn create_initial_cards() -> Vec<Card> {
    let mut cards = Vec::new();
    let mut id = 0;
    
    // Create pairs of cards
    for emoji in EMOJIS {
        for _ in 0..2 {
            cards.push(Card {
                id,
                emoji: emoji.to_string(),
                is_flipped: false,
                is_matched: false,
            });
            id += 1;
        }
    }
    
    // Shuffle cards
    let mut rng = thread_rng();
    cards.shuffle(&mut rng);
    
    cards
}

const GAME_STYLES: &str = r#"
    :root {
        --game-primary: #6366f1;
        --game-secondary: #ec4899;
        --game-success: #10b981;
        --game-gradient-1: #6366f1;
        --game-gradient-2: #a855f7;
        --game-gradient-3: #ec4899;
    }
    
    * {
        box-sizing: border-box;
        margin: 0;
        padding: 0;
    }
    
    body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        background: linear-gradient(135deg, var(--game-gradient-1), var(--game-gradient-2), var(--game-gradient-3));
        min-height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
    }
    
    .memory-game {
        position: relative;
        width: 100%;
        padding: 20px;
    }
    
    .bg-animation {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        pointer-events: none;
    }
    
    .star {
        position: absolute;
        width: 4px;
        height: 4px;
        background: white;
        border-radius: 50%;
        animation: twinkle 5s infinite;
    }
    
    .star-1 { top: 10%; left: 20%; animation-delay: 0s; }
    .star-2 { top: 30%; left: 80%; animation-delay: 1s; }
    .star-3 { top: 60%; left: 10%; animation-delay: 2s; }
    .star-4 { top: 80%; left: 70%; animation-delay: 3s; }
    .star-5 { top: 50%; left: 50%; animation-delay: 4s; }
    
    @keyframes twinkle {
        0%, 100% { opacity: 0; transform: scale(0.5); }
        50% { opacity: 1; transform: scale(1.5); }
    }
    
    .game-container {
        max-width: 600px;
        margin: 0 auto;
        background: rgba(255, 255, 255, 0.95);
        backdrop-filter: blur(20px);
        border-radius: 30px;
        padding: 40px;
        box-shadow: 0 25px 50px rgba(0, 0, 0, 0.15);
        position: relative;
        z-index: 1;
    }
    
    header {
        text-align: center;
        margin-bottom: 30px;
    }
    
    h1 {
        font-size: 3rem;
        font-weight: 800;
        color: #1a202c;
        margin-bottom: 10px;
    }
    
    h2 {
        font-size: 2rem;
        color: var(--game-success);
        margin-bottom: 15px;
    }
    
    .gradient-text {
        background: linear-gradient(135deg, var(--game-primary), var(--game-secondary));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
    }
    
    .subtitle {
        color: #64748b;
        font-size: 1.2rem;
    }
    
    .stats {
        display: flex;
        justify-content: center;
        gap: 40px;
        margin-bottom: 30px;
    }
    
    .stat {
        text-align: center;
    }
    
    .stat-label {
        display: block;
        font-size: 0.9rem;
        color: #64748b;
        text-transform: uppercase;
        letter-spacing: 1px;
        margin-bottom: 5px;
    }
    
    .stat-value {
        display: block;
        font-size: 2rem;
        font-weight: 700;
        color: var(--game-primary);
    }
    
    .game-board {
        display: grid;
        grid-template-columns: repeat(4, 1fr);
        gap: 15px;
        margin-bottom: 30px;
    }
    
    .card {
        aspect-ratio: 1;
        cursor: pointer;
        perspective: 1000px;
    }
    
    .card-inner {
        position: relative;
        width: 100%;
        height: 100%;
        text-align: center;
        transition: transform 0.6s;
        transform-style: preserve-3d;
    }
    
    .card.flipped .card-inner,
    .card.matched .card-inner {
        transform: rotateY(180deg);
    }
    
    .card-front, .card-back {
        position: absolute;
        width: 100%;
        height: 100%;
        -webkit-backface-visibility: hidden;
        backface-visibility: hidden;
        border-radius: 15px;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 3rem;
        font-weight: bold;
        box-shadow: 0 4px 10px rgba(0, 0, 0, 0.1);
    }
    
    .card-front {
        background: linear-gradient(135deg, var(--game-primary), var(--game-secondary));
        color: white;
    }
    
    .card-back {
        background: white;
        color: #1a202c;
        transform: rotateY(180deg);
        border: 3px solid #f0f0f0;
    }
    
    .card.matched .card-back {
        background: var(--game-success);
        color: white;
        animation: bounce 0.5s ease;
    }
    
    @keyframes bounce {
        0%, 100% { transform: rotateY(180deg) scale(1); }
        50% { transform: rotateY(180deg) scale(1.1); }
    }
    
    .win-screen {
        text-align: center;
        animation: fadeIn 0.5s ease;
    }
    
    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(20px); }
        to { opacity: 1; transform: translateY(0); }
    }
    
    .btn {
        padding: 15px 30px;
        border: none;
        border-radius: 12px;
        font-size: 1.1rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.3s ease;
        margin: 10px auto;
        display: block;
    }
    
    .btn-primary {
        background: linear-gradient(135deg, var(--game-primary), var(--game-secondary));
        color: white;
        box-shadow: 0 4px 15px rgba(99, 102, 241, 0.3);
    }
    
    .btn-primary:hover {
        transform: translateY(-2px);
        box-shadow: 0 6px 20px rgba(99, 102, 241, 0.4);
    }
    
    .btn-secondary {
        background: #f3f4f6;
        color: #4b5563;
    }
    
    .btn-secondary:hover {
        background: #e5e7eb;
    }
    
    footer {
        text-align: center;
        color: #64748b;
        font-size: 0.9rem;
        margin-top: 20px;
    }
    
    footer a {
        color: var(--game-primary);
        text-decoration: none;
        font-weight: 600;
    }
    
    footer a:hover {
        text-decoration: underline;
    }
    
    @media (max-width: 600px) {
        h1 {
            font-size: 2rem;
        }
        
        .game-container {
            padding: 30px 20px;
        }
        
        .game-board {
            gap: 10px;
        }
        
        .card-front, .card-back {
            font-size: 2rem;
        }
    }
"#;

#[wasm_bindgen(start)]
pub fn main() {
    web_sys::console::log_1(&"Memory Game starting...".into());
    mount(Box::new(MemoryGame), "root");
    web_sys::console::log_1(&"Memory Game mounted successfully!".into());
}