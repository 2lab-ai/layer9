//! Global State Management - L6

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::hooks::use_effect;

// Type alias to simplify complex types
type ReducerFn<S, A> = Rc<dyn Fn(&S, A) -> S>;

thread_local! {
    static STORE: RefCell<Store> = RefCell::new(Store::new());
}

/// Store for global state
struct Store {
    state: HashMap<TypeId, Box<dyn Any>>,
    listeners: HashMap<TypeId, Vec<Box<dyn Fn()>>>,
}

impl Store {
    fn new() -> Self {
        Store {
            state: HashMap::new(),
            listeners: HashMap::new(),
        }
    }

    fn set<T: 'static>(&mut self, value: T) {
        let type_id = TypeId::of::<T>();
        self.state.insert(type_id, Box::new(value));

        // Notify listeners
        if let Some(listeners) = self.listeners.get(&type_id) {
            for listener in listeners {
                listener();
            }
        }
    }

    fn get<T: 'static + Clone>(&self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.state
            .get(&type_id)
            .and_then(|any| any.downcast_ref::<T>())
            .cloned()
    }

    fn subscribe<T: 'static>(&mut self, listener: Box<dyn Fn()>) -> SubscriptionId {
        let type_id = TypeId::of::<T>();
        let listeners = self.listeners.entry(type_id).or_default();
        let id = listeners.len();
        listeners.push(listener);

        SubscriptionId { type_id, id }
    }

    fn unsubscribe(&mut self, sub_id: SubscriptionId) {
        if let Some(listeners) = self.listeners.get_mut(&sub_id.type_id) {
            if sub_id.id < listeners.len() {
                let _ = listeners.remove(sub_id.id);
            }
        }
    }
}

/// Subscription ID for cleanup
pub struct SubscriptionId {
    type_id: TypeId,
    id: usize,
}

/// Create a global state atom
pub fn create_atom<T: 'static + Clone + Default>(initial: T) -> Atom<T> {
    STORE.with(|store| {
        store.borrow_mut().set(initial);
    });

    Atom {
        _phantom: std::marker::PhantomData,
    }
}

/// Atom represents a piece of global state
pub struct Atom<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: 'static + Clone> Atom<T> {
    pub fn get(&self) -> Option<T> {
        STORE.with(|store| store.borrow().get::<T>())
    }

    pub fn set(&self, value: T) {
        STORE.with(|store| store.borrow_mut().set(value));
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        if let Some(mut value) = self.get() {
            f(&mut value);
            self.set(value);
        }
    }

    pub fn subscribe(&self, listener: impl Fn() + 'static) -> SubscriptionId {
        STORE.with(|store| store.borrow_mut().subscribe::<T>(Box::new(listener)))
    }
}

/// Use global state hook
pub fn use_atom<T: 'static + Clone>(atom: &Atom<T>) -> AtomHandle<T> {
    let value = atom.get();
    let update_trigger = use_update();

    // Subscribe to changes
    let sub_id = atom.subscribe(move || {
        update_trigger();
    });

    // Cleanup on unmount (run once with empty deps)
    use_effect((), move || {
        move || {
            STORE.with(|store| {
                store.borrow_mut().unsubscribe(sub_id);
            });
        }
    });

    AtomHandle {
        value,
        atom: atom.clone(),
    }
}

/// Handle to atom value with update capability
pub struct AtomHandle<T> {
    value: Option<T>,
    atom: Atom<T>,
}

impl<T: Clone + 'static> AtomHandle<T> {
    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn set(&self, value: T) {
        self.atom.set(value);
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        self.atom.update(f);
    }
}

/// Selector for derived state
pub struct Selector<T, U> {
    source: Atom<T>,
    selector: Rc<dyn Fn(&T) -> U>,
}

impl<T: 'static + Clone, U: 'static + Clone> Selector<T, U> {
    pub fn new(source: Atom<T>, selector: impl Fn(&T) -> U + 'static) -> Self {
        Selector {
            source,
            selector: Rc::new(selector),
        }
    }

    pub fn get(&self) -> Option<U> {
        self.source.get().map(|value| (self.selector)(&value))
    }
}

/// Use selector hook
pub fn use_selector<T: 'static + Clone, U: 'static + Clone>(
    selector: &Selector<T, U>,
) -> Option<U> {
    let value = selector.get();
    let update_trigger = use_update();

    // Subscribe to source changes
    let sub_id = selector.source.subscribe(move || {
        update_trigger();
    });

    // Cleanup (run once with empty deps)
    use_effect((), move || {
        move || {
            STORE.with(|store| {
                store.borrow_mut().unsubscribe(sub_id);
            });
        }
    });

    value
}

/// Redux-style reducer store
pub struct ReducerStore<S, A> {
    state: Atom<S>,
    reducer: ReducerFn<S, A>,
}

impl<S: 'static + Clone + Default, A: 'static> ReducerStore<S, A> {
    pub fn new(initial: S, reducer: impl Fn(&S, A) -> S + 'static) -> Self {
        ReducerStore {
            state: create_atom(initial),
            reducer: Rc::new(reducer),
        }
    }

    pub fn dispatch(&self, action: A) {
        if let Some(current) = self.state.get() {
            let new_state = (self.reducer)(&current, action);
            self.state.set(new_state);
        }
    }

    pub fn get_state(&self) -> Option<S> {
        self.state.get()
    }
}

/// Use reducer hook
pub fn use_reducer<S: 'static + Clone + Default, A: 'static>(
    store: &ReducerStore<S, A>,
) -> (Option<S>, impl Fn(A)) {
    let state = use_atom(&store.state);
    let store_clone = store.clone();

    let dispatch = move |action: A| {
        store_clone.dispatch(action);
    };

    (state.get().cloned(), dispatch)
}

// Helper hooks that integrate with the reactive system
fn use_update() -> impl Fn() {
    // Trigger component re-render using the reactive system
    || {
        crate::reactive_v2::queue_current_render();
    }
}

// use_effect is now provided by the hooks module

// Make types cloneable for convenience
impl<T> Clone for Atom<T> {
    fn clone(&self) -> Self {
        Atom {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S, A> Clone for ReducerStore<S, A> {
    fn clone(&self) -> Self {
        ReducerStore {
            state: self.state.clone(),
            reducer: self.reducer.clone(),
        }
    }
}

// Example usage:
#[derive(Clone, Default)]
pub struct AppState {
    pub user: Option<User>,
    pub theme: Theme,
    pub count: i32,
}

#[derive(Clone)]
pub struct User {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

pub enum AppAction {
    SetUser(Option<User>),
    ToggleTheme,
    Increment,
    Decrement,
}

pub fn create_app_store() -> ReducerStore<AppState, AppAction> {
    let initial = AppState {
        user: None,
        theme: Theme::Dark,
        count: 0,
    };

    ReducerStore::new(initial, |state, action| {
        let mut new_state = state.clone();

        match action {
            AppAction::SetUser(user) => new_state.user = user,
            AppAction::ToggleTheme => {
                new_state.theme = match state.theme {
                    Theme::Light => Theme::Dark,
                    Theme::Dark => Theme::Light,
                };
            }
            AppAction::Increment => new_state.count += 1,
            AppAction::Decrement => new_state.count -= 1,
        }

        new_state
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn test_atom_creation_and_retrieval() {
        let atom = create_atom(42);
        assert_eq!(atom.get(), Some(42));
    }

    #[test]
    fn test_atom_set_and_update() {
        let atom = create_atom(10);
        
        atom.set(20);
        assert_eq!(atom.get(), Some(20));
        
        atom.update(|val| *val += 5);
        assert_eq!(atom.get(), Some(25));
    }

    #[test]
    fn test_atom_subscription() {
        let atom = create_atom(0);
        let callback_count = Rc::new(RefCell::new(0));
        let callback_count_clone = callback_count.clone();
        
        let _sub_id = atom.subscribe(move || {
            *callback_count_clone.borrow_mut() += 1;
        });
        
        // Setting value should trigger callback
        atom.set(1);
        assert_eq!(*callback_count.borrow(), 1);
        
        atom.set(2);
        assert_eq!(*callback_count.borrow(), 2);
        
        atom.update(|val| *val += 1);
        assert_eq!(*callback_count.borrow(), 3);
    }

    #[test]
    fn test_atom_unsubscribe() {
        let atom = create_atom(0);
        let callback_count = Rc::new(RefCell::new(0));
        let callback_count_clone = callback_count.clone();
        
        let sub_id = atom.subscribe(move || {
            *callback_count_clone.borrow_mut() += 1;
        });
        
        atom.set(1);
        assert_eq!(*callback_count.borrow(), 1);
        
        // Unsubscribe
        STORE.with(|store| {
            store.borrow_mut().unsubscribe(sub_id);
        });
        
        // Should not trigger callback after unsubscribe
        atom.set(2);
        assert_eq!(*callback_count.borrow(), 1);
    }

    #[test]
    fn test_selector() {
        let atom = create_atom(10);
        let selector = Selector::new(atom.clone(), |val| val * 2);
        
        assert_eq!(selector.get(), Some(20));
        
        atom.set(15);
        assert_eq!(selector.get(), Some(30));
    }

    #[test]
    fn test_reducer_store() {
        #[derive(Clone, PartialEq, Debug, Default)]
        struct TestState {
            count: i32,
            text: String,
        }
        
        enum TestAction {
            Increment,
            Decrement,
            SetText(String),
        }
        
        let store = ReducerStore::new(TestState::default(), |state, action| {
            let mut new_state = state.clone();
            match action {
                TestAction::Increment => new_state.count += 1,
                TestAction::Decrement => new_state.count -= 1,
                TestAction::SetText(text) => new_state.text = text,
            }
            new_state
        });
        
        assert_eq!(store.get_state().unwrap().count, 0);
        
        store.dispatch(TestAction::Increment);
        assert_eq!(store.get_state().unwrap().count, 1);
        
        store.dispatch(TestAction::Increment);
        assert_eq!(store.get_state().unwrap().count, 2);
        
        store.dispatch(TestAction::Decrement);
        assert_eq!(store.get_state().unwrap().count, 1);
        
        store.dispatch(TestAction::SetText("Hello".to_string()));
        let state = store.get_state().unwrap();
        assert_eq!(state.count, 1);
        assert_eq!(state.text, "Hello");
    }

    #[test]
    fn test_app_store_example() {
        let store = create_app_store();
        
        // Initial state
        let state = store.get_state().unwrap();
        assert!(state.user.is_none());
        assert!(matches!(state.theme, Theme::Dark));
        assert_eq!(state.count, 0);
        
        // Set user
        store.dispatch(AppAction::SetUser(Some(User {
            id: "1".to_string(),
            name: "Test User".to_string(),
        })));
        
        let state = store.get_state().unwrap();
        assert!(state.user.is_some());
        assert_eq!(state.user.as_ref().unwrap().name, "Test User");
        
        // Toggle theme
        store.dispatch(AppAction::ToggleTheme);
        let state = store.get_state().unwrap();
        assert!(matches!(state.theme, Theme::Light));
        
        store.dispatch(AppAction::ToggleTheme);
        let state = store.get_state().unwrap();
        assert!(matches!(state.theme, Theme::Dark));
        
        // Counter operations
        store.dispatch(AppAction::Increment);
        store.dispatch(AppAction::Increment);
        assert_eq!(store.get_state().unwrap().count, 2);
        
        store.dispatch(AppAction::Decrement);
        assert_eq!(store.get_state().unwrap().count, 1);
    }

    #[test]
    fn test_multiple_atoms_different_types() {
        // Test with different types to ensure isolation
        let string_atom = create_atom("hello".to_string());
        let int_atom = create_atom(42i32);
        let bool_atom = create_atom(true);
        
        assert_eq!(string_atom.get(), Some("hello".to_string()));
        assert_eq!(int_atom.get(), Some(42));
        assert_eq!(bool_atom.get(), Some(true));
        
        string_atom.set("world".to_string());
        int_atom.set(100);
        bool_atom.set(false);
        
        assert_eq!(string_atom.get(), Some("world".to_string()));
        assert_eq!(int_atom.get(), Some(100));
        assert_eq!(bool_atom.get(), Some(false));
    }

    #[test]
    fn test_atom_subscription_behavior() {
        // Test subscription for a single atom type
        let atom = create_atom(0);
        
        let callback_count = Rc::new(RefCell::new(0));
        let callback_clone = callback_count.clone();
        
        let sub_id = atom.subscribe(move || {
            *callback_clone.borrow_mut() += 1;
        });
        
        // First update should trigger callback
        atom.set(1);
        assert_eq!(*callback_count.borrow(), 1);
        
        // Second update should also trigger
        atom.set(2);
        assert_eq!(*callback_count.borrow(), 2);
        
        // After unsubscribe, no more callbacks
        STORE.with(|store| {
            store.borrow_mut().unsubscribe(sub_id);
        });
        
        atom.set(3);
        assert_eq!(*callback_count.borrow(), 2); // Should remain 2
    }

    #[test]
    fn test_complex_selector_chain() {
        let base_atom = create_atom(5);
        let doubled = Selector::new(base_atom.clone(), |val| val * 2);
        let squared = Selector::new(base_atom.clone(), |val| val * val);
        
        assert_eq!(doubled.get(), Some(10));
        assert_eq!(squared.get(), Some(25));
        
        base_atom.set(3);
        assert_eq!(doubled.get(), Some(6));
        assert_eq!(squared.get(), Some(9));
    }
}
