//! Global State Management - L6

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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
        let listeners = self.listeners.entry(type_id).or_insert_with(Vec::new);
        let id = listeners.len();
        listeners.push(listener);

        SubscriptionId { type_id, id }
    }

    fn unsubscribe(&mut self, sub_id: SubscriptionId) {
        if let Some(listeners) = self.listeners.get_mut(&sub_id.type_id) {
            if sub_id.id < listeners.len() {
                listeners.remove(sub_id.id);
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

    // Cleanup on unmount
    use_effect(move || {
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

impl<T: Clone> AtomHandle<T> {
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

    // Cleanup
    use_effect(move || {
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
    reducer: Rc<dyn Fn(&S, A) -> S>,
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
pub fn use_reducer<S: 'static + Clone, A: 'static>(
    store: &ReducerStore<S, A>,
) -> (Option<S>, impl Fn(A)) {
    let state = use_atom(&store.state);
    let store_clone = store.clone();

    let dispatch = move |action: A| {
        store_clone.dispatch(action);
    };

    (state.get().cloned(), dispatch)
}

// Helper hooks (these would be implemented elsewhere)
fn use_update() -> impl Fn() {
    // Trigger component re-render
    || {
        web_sys::console::log_1(&"Update triggered".into());
    }
}

pub fn use_effect<F, C>(effect: F)
where
    F: FnOnce() -> C,
    C: FnOnce() + 'static,
{
    // Run effect and return cleanup
    let _cleanup = effect();
    // Store cleanup for later
}

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
#[derive(Clone)]
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

#[derive(Clone)]
pub enum Theme {
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
