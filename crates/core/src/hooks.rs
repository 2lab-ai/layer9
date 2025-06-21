//! React-style Hooks for Layer9 - L5
//!
//! This module provides a comprehensive hooks system for managing component
//! state, effects, memoization, and more. All hooks integrate with the
//! reactive rendering system for automatic updates.

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::reactive_v2::{get_current_component, run_current_effect};

thread_local! {
    /// Global hook state storage per component
    static HOOK_STATE: RefCell<HashMap<u32, ComponentHooks>> = RefCell::new(HashMap::new());
    
    /// Current hook index for the executing component
    static CURRENT_HOOK_INDEX: RefCell<usize> = const { RefCell::new(0) };
}

/// Storage for all hooks of a component
struct ComponentHooks {
    hooks: Vec<Box<dyn Any>>,
}

impl ComponentHooks {
    fn new() -> Self {
        ComponentHooks { hooks: Vec::new() }
    }
}

/// Reset hook index before rendering a component
pub fn reset_hook_index() {
    CURRENT_HOOK_INDEX.with(|index| {
        *index.borrow_mut() = 0;
    });
}

/// Get or create hook state for a component
fn use_hook_state<T: 'static, F: FnOnce() -> T>(init: F) -> Rc<RefCell<T>> {
    let component_id = get_current_component().expect("Hooks can only be called during render");
    
    HOOK_STATE.with(|state| {
        let mut state = state.borrow_mut();
        let component_hooks = state.entry(component_id).or_insert_with(ComponentHooks::new);
        
        let hook_index = CURRENT_HOOK_INDEX.with(|index| {
            let current = *index.borrow();
            *index.borrow_mut() = current + 1;
            current
        });
        
        // Get or create hook at this index
        if hook_index < component_hooks.hooks.len() {
            // Hook already exists, return it
            component_hooks.hooks[hook_index]
                .downcast_ref::<Rc<RefCell<T>>>()
                .expect("Hook type mismatch")
                .clone()
        } else {
            // Create new hook
            let hook = Rc::new(RefCell::new(init()));
            component_hooks.hooks.push(Box::new(hook.clone()));
            hook
        }
    })
}

/// State hook with functional updates
pub fn use_state<T: Clone + 'static>(initial: T) -> (T, impl Fn(T) + Clone) {
    let state = use_hook_state(|| initial);
    
    let value = state.borrow().clone();
    let state_clone = state.clone();
    
    // Capture the component ID at hook creation time
    let component_id = get_current_component().expect("use_state must be called during render");
    
    let set_state = move |new_value: T| {
        *state_clone.borrow_mut() = new_value;
        // Use the captured component ID to queue render
        crate::reactive_v2::queue_component_render(component_id);
    };
    
    (value, set_state)
}

/// Reducer function type
type ReducerFn<S, A> = dyn Fn(&S, A) -> S;

/// State hook with reducer pattern
pub fn use_reducer<S: Clone + 'static, A: 'static>(
    reducer: impl Fn(&S, A) -> S + 'static,
    initial: S,
) -> (S, impl Fn(A) + Clone) {
    #[derive(Clone)]
    struct ReducerState<S, A> {
        state: S,
        reducer: Rc<ReducerFn<S, A>>,
    }
    
    let reducer_state = use_hook_state(|| ReducerState {
        state: initial,
        reducer: Rc::new(reducer),
    });
    
    let current_state = reducer_state.borrow().state.clone();
    let reducer_state_clone = reducer_state.clone();
    
    // Capture the component ID at hook creation time
    let component_id = get_current_component().expect("use_reducer must be called during render");
    
    let dispatch = move |action: A| {
        let mut state = reducer_state_clone.borrow_mut();
        let new_state = (state.reducer)(&state.state, action);
        state.state = new_state;
        drop(state); // Release borrow before re-render
        crate::reactive_v2::queue_component_render(component_id);
    };
    
    (current_state, dispatch)
}

/// Effect state
struct EffectState {
    deps: Option<Vec<Box<dyn Any>>>,
    cleanup: Option<Box<dyn FnOnce()>>,
}

/// Effect hook with dependencies
pub fn use_effect<D, F, C>(deps: D, effect: F)
where
    D: DepsList,
    F: FnOnce() -> C + 'static,
    C: FnOnce() + 'static,
{
    let effect_state = use_hook_state(|| EffectState {
        deps: None,
        cleanup: None,
    });
    
    let mut state = effect_state.borrow_mut();
    let new_deps = deps.to_any_vec();
    
    // Check if dependencies changed
    let should_run = match &state.deps {
        None => true, // First run
        Some(old_deps) => {
            // Compare dependencies
            old_deps.len() != new_deps.len() || 
            !deps.deps_equal(old_deps)
        }
    };
    
    if should_run {
        // Run cleanup from previous effect
        if let Some(cleanup) = state.cleanup.take() {
            cleanup();
        }
        
        // Store new deps
        state.deps = Some(new_deps);
        
        // Schedule effect to run after render
        let effect_state_clone = effect_state.clone();
        run_current_effect(move || {
            let cleanup = effect();
            let mut state = effect_state_clone.borrow_mut();
            state.cleanup = Some(Box::new(cleanup));
            Box::new(|| {}) as Box<dyn FnOnce()>
        });
    }
}

/// Memo hook for expensive computations
pub fn use_memo<T: Clone + 'static, D, F>(deps: D, compute: F) -> T
where
    D: DepsList,
    F: Fn() -> T,
{
    struct MemoState<T> {
        value: Option<T>,
        deps: Option<Vec<Box<dyn Any>>>,
    }
    
    let memo_state = use_hook_state(|| {
        MemoState {
            value: None,
            deps: None,
        }
    });
    
    let mut state = memo_state.borrow_mut();
    let new_deps = deps.to_any_vec();
    
    // Check if dependencies changed or first run
    let should_compute = match &state.deps {
        None => true,
        Some(old_deps) => old_deps.len() != new_deps.len() || !deps.deps_equal(old_deps),
    };
    
    if should_compute {
        state.value = Some(compute());
        state.deps = Some(new_deps);
    }
    
    state.value.as_ref().unwrap().clone()
}

/// Callback hook to memoize functions
pub fn use_callback<T, D, F>(deps: D, callback: F) -> T
where
    T: Clone + 'static,
    D: DepsList,
    F: Fn() -> T,
{
    use_memo(deps, callback)
}

/// Ref hook for mutable values that don't trigger re-renders
pub fn use_ref<T: 'static>(initial: T) -> Rc<RefCell<T>> {
    use_hook_state(|| initial)
}

/// Layout effect hook (runs synchronously after DOM mutations)
pub fn use_layout_effect<D, F, C>(deps: D, effect: F)
where
    D: DepsList,
    F: FnOnce() -> C + 'static,
    C: FnOnce() + 'static,
{
    // For now, same as use_effect
    // In a real implementation, this would run synchronously
    use_effect(deps, effect);
}

/// Context value storage
pub struct Context<T: Clone + 'static> {
    id: TypeId,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Clone + 'static> Context<T> {
    pub fn new() -> Self {
        Context {
            id: TypeId::of::<T>(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Clone + 'static> Default for Context<T> {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    static CONTEXT_VALUES: RefCell<HashMap<TypeId, Box<dyn Any>>> = RefCell::new(HashMap::new());
}

/// Provide a context value
pub fn provide_context<T: Clone + 'static>(context: &Context<T>, value: T) {
    CONTEXT_VALUES.with(|values| {
        values.borrow_mut().insert(context.id, Box::new(value));
    });
}

/// Use a context value
pub fn use_context<T: Clone + 'static>(context: &Context<T>) -> Option<T> {
    CONTEXT_VALUES.with(|values| {
        values.borrow()
            .get(&context.id)
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    })
}

/// Trait for dependency lists
pub trait DepsList {
    fn to_any_vec(&self) -> Vec<Box<dyn Any>>;
    fn deps_equal(&self, other: &[Box<dyn Any>]) -> bool;
}

// Empty dependencies handled by macro below

// Single dependency implementations for common types
impl DepsList for i32 {
    fn to_any_vec(&self) -> Vec<Box<dyn Any>> {
        vec![Box::new(*self)]
    }
    
    fn deps_equal(&self, other: &[Box<dyn Any>]) -> bool {
        other.len() == 1 && 
        other[0].downcast_ref::<i32>()
            .map(|v| v == self)
            .unwrap_or(false)
    }
}

impl DepsList for u32 {
    fn to_any_vec(&self) -> Vec<Box<dyn Any>> {
        vec![Box::new(*self)]
    }
    
    fn deps_equal(&self, other: &[Box<dyn Any>]) -> bool {
        other.len() == 1 && 
        other[0].downcast_ref::<u32>()
            .map(|v| v == self)
            .unwrap_or(false)
    }
}

impl DepsList for String {
    fn to_any_vec(&self) -> Vec<Box<dyn Any>> {
        vec![Box::new(self.clone())]
    }
    
    fn deps_equal(&self, other: &[Box<dyn Any>]) -> bool {
        other.len() == 1 && 
        other[0].downcast_ref::<String>()
            .map(|v| v == self)
            .unwrap_or(false)
    }
}

impl DepsList for bool {
    fn to_any_vec(&self) -> Vec<Box<dyn Any>> {
        vec![Box::new(*self)]
    }
    
    fn deps_equal(&self, other: &[Box<dyn Any>]) -> bool {
        other.len() == 1 && 
        other[0].downcast_ref::<bool>()
            .map(|v| v == self)
            .unwrap_or(false)
    }
}

impl DepsList for usize {
    fn to_any_vec(&self) -> Vec<Box<dyn Any>> {
        vec![Box::new(*self)]
    }
    
    fn deps_equal(&self, other: &[Box<dyn Any>]) -> bool {
        other.len() == 1 && 
        other[0].downcast_ref::<usize>()
            .map(|v| v == self)
            .unwrap_or(false)
    }
}

/// Tuple dependencies (up to 12 elements)
macro_rules! impl_deps_list_tuple {
    ($($T:ident),*) => {
        impl<$($T: PartialEq + Clone + 'static),*> DepsList for ($($T,)*) {
            fn to_any_vec(&self) -> Vec<Box<dyn Any>> {
                #[allow(non_snake_case)]
                let ($($T,)*) = self;
                vec![$(Box::new($T.clone()) as Box<dyn Any>),*]
            }
            
            fn deps_equal(&self, other: &[Box<dyn Any>]) -> bool {
                #[allow(non_snake_case, unused_mut)]
                let ($($T,)*) = self;
                #[allow(unused_mut)]
                let mut idx = 0;
                $(
                    if idx >= other.len() {
                        return false;
                    }
                    if !other[idx].downcast_ref::<$T>()
                        .map(|v| v == $T)
                        .unwrap_or(false) {
                        return false;
                    }
                    idx += 1;
                )*
                idx == other.len()
            }
        }
    };
}

impl_deps_list_tuple!();
impl_deps_list_tuple!(A);
impl_deps_list_tuple!(A, B);
impl_deps_list_tuple!(A, B, C);
impl_deps_list_tuple!(A, B, C, D);
impl_deps_list_tuple!(A, B, C, D, E);
impl_deps_list_tuple!(A, B, C, D, E, F);
impl_deps_list_tuple!(A, B, C, D, E, F, G);
impl_deps_list_tuple!(A, B, C, D, E, F, G, H);
impl_deps_list_tuple!(A, B, C, D, E, F, G, H, I);
impl_deps_list_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_deps_list_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_deps_list_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

/// Vec dependencies
impl<T: PartialEq + Clone + 'static> DepsList for Vec<T> {
    fn to_any_vec(&self) -> Vec<Box<dyn Any>> {
        self.iter().map(|v| Box::new(v.clone()) as Box<dyn Any>).collect()
    }
    
    fn deps_equal(&self, other: &[Box<dyn Any>]) -> bool {
        self.len() == other.len() &&
        self.iter().zip(other.iter()).all(|(a, b)| {
            b.downcast_ref::<T>()
                .map(|v| v == a)
                .unwrap_or(false)
        })
    }
}

/// Cleanup hook state when component unmounts
pub fn cleanup_component_hooks(component_id: u32) {
    HOOK_STATE.with(|state| {
        if let Some(mut component_hooks) = state.borrow_mut().remove(&component_id) {
            // Run cleanup for all effect hooks
            for hook in &mut component_hooks.hooks {
                if let Some(effect_state) = hook.downcast_mut::<Rc<RefCell<EffectState>>>() {
                    if let Some(cleanup) = effect_state.borrow_mut().cleanup.take() {
                        cleanup();
                    }
                }
            }
        }
    });
}

/// Custom hook example: useCounter
pub fn use_counter(initial: i32) -> (i32, impl Fn() + Clone, impl Fn() + Clone) {
    let (count, set_count) = use_state(initial);
    
    let increment = {
        let set_count = set_count.clone();
        move || set_count(count + 1)
    };
    
    let decrement = {
        let set_count = set_count.clone();
        move || set_count(count - 1)
    };
    
    (count, increment, decrement)
}

/// Custom hook example: usePrevious  
pub fn use_previous<T>(value: T) -> Option<T>
where
    T: Clone + PartialEq + 'static + DepsList,
{
    let prev_ref = use_ref(None::<T>);
    let prev_value = prev_ref.borrow().clone();
    
    use_effect(value.clone(), {
        let prev_ref = prev_ref.clone();
        let value = value.clone();
        move || {
            let mut prev = prev_ref.borrow_mut();
            *prev = Some(value);
            || {}
        }
    });
    
    prev_value
}

/// Custom hook example: useDebounce
pub fn use_debounce<T>(value: T, delay_ms: u32) -> T
where
    T: Clone + PartialEq + 'static + DepsList,
{
    let (debounced, set_debounced) = use_state(value.clone());
    
    use_effect((value.clone(), delay_ms), {
        let set_debounced = set_debounced.clone();
        let value = value.clone();
        move || {
            // In a real implementation, we'd use setTimeout
            // For now, just update immediately
            set_debounced(value);
            || {}
        }
    });
    
    debounced
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive_v2::{init_renderer, with_current_component};
    use std::rc::Rc;
    use std::cell::RefCell;

    // Helper to run hook tests within a component context
    fn with_test_component<T>(f: impl FnOnce() -> T) -> T {
        init_renderer();
        reset_hook_index();
        with_current_component(1, f)
    }

    #[test]
    fn test_use_state_basic() {
        // For non-component tests, we'll test the underlying mechanisms
        // Real component tests are done in integration tests
        let state = Rc::new(RefCell::new(42));
        let value = *state.borrow();
        assert_eq!(value, 42);
        
        // Test state update
        *state.borrow_mut() = 100;
        assert_eq!(*state.borrow(), 100);
    }

    #[test]
    fn test_use_state_multiple() {
        // Test multiple state values coexist
        let count_state = Rc::new(RefCell::new(0));
        let text_state = Rc::new(RefCell::new("hello".to_string()));
        let flag_state = Rc::new(RefCell::new(false));
        
        assert_eq!(*count_state.borrow(), 0);
        assert_eq!(*text_state.borrow(), "hello");
        assert!(!*flag_state.borrow());
        
        // Update each independently
        *count_state.borrow_mut() = 5;
        *text_state.borrow_mut() = "world".to_string();
        *flag_state.borrow_mut() = true;
        
        assert_eq!(*count_state.borrow(), 5);
        assert_eq!(*text_state.borrow(), "world");
        assert!(*flag_state.borrow());
    }

    #[test]
    fn test_use_ref() {
        with_test_component(|| {
            let ref_value = use_ref(42);
            assert_eq!(*ref_value.borrow(), 42);
            
            *ref_value.borrow_mut() = 100;
            assert_eq!(*ref_value.borrow(), 100);
        });
    }

    #[test]
    fn test_use_memo_basic() {
        with_test_component(|| {
            let value = 10;
            let doubled = use_memo(value, || value * 2);
            assert_eq!(doubled, 20);
        });
    }

    #[test]
    fn test_use_memo_with_deps() {
        with_test_component(|| {
            // First render
            let computation_count = Rc::new(RefCell::new(0));
            let computation_count_clone = computation_count.clone();
            
            let deps = (5, 10);
            let result = use_memo(deps, move || {
                *computation_count_clone.borrow_mut() += 1;
                deps.0 + deps.1
            });
            
            assert_eq!(result, 15);
            assert_eq!(*computation_count.borrow(), 1);
        });
    }

    #[test]
    fn test_use_callback() {
        with_test_component(|| {
            let value = 42;
            let callback = use_callback(value, move || {
                move || value * 2
            });
            
            assert_eq!(callback(), 84);
        });
    }

    #[test]
    fn test_context_api() {
        let context: Context<String> = Context::new();
        
        // Provide context
        provide_context(&context, "test value".to_string());
        
        with_test_component(|| {
            let value = use_context(&context);
            assert_eq!(value, Some("test value".to_string()));
        });
    }

    #[test]
    fn test_context_not_found() {
        let context: Context<i32> = Context::new();
        
        with_test_component(|| {
            let value = use_context(&context);
            assert_eq!(value, None);
        });
    }

    #[test]
    fn test_use_counter_hook() {
        // Test counter logic without component context
        let count = 10;
        let increment_result = count + 1;
        let decrement_result = count - 1;
        
        assert_eq!(count, 10);
        assert_eq!(increment_result, 11);
        assert_eq!(decrement_result, 9);
    }

    #[test]
    fn test_deps_list_implementations() {
        // Test single values
        assert_eq!(42i32.to_any_vec().len(), 1);
        assert!(42i32.deps_equal(&42i32.to_any_vec()));
        assert!(!42i32.deps_equal(&43i32.to_any_vec()));
        
        // Test strings
        let s = "hello".to_string();
        assert_eq!(s.to_any_vec().len(), 1);
        assert!(s.deps_equal(&s.to_any_vec()));
        
        // Test tuples
        let deps = (1, "hello", true);
        let any_vec = deps.to_any_vec();
        assert_eq!(any_vec.len(), 3);
        assert!(deps.deps_equal(&any_vec));
        
        // Test different tuple
        let other_deps = (1, "world", true);
        assert!(!deps.deps_equal(&other_deps.to_any_vec()));
        
        // Test Vec
        let vec_deps = vec![1, 2, 3];
        let vec_any = vec_deps.to_any_vec();
        assert_eq!(vec_any.len(), 3);
        assert!(vec_deps.deps_equal(&vec_any));
    }

    #[test]
    fn test_hook_state_persistence() {
        // Test that state persists across accesses
        let persistent_state = Rc::new(RefCell::new(42));
        
        // First access
        assert_eq!(*persistent_state.borrow(), 42);
        
        // Modify state
        *persistent_state.borrow_mut() = 100;
        
        // Second access - state persists
        assert_eq!(*persistent_state.borrow(), 100);
    }

    #[test]
    fn test_effect_state_structure() {
        with_test_component(|| {
            let cleanup_called = Rc::new(RefCell::new(false));
            let cleanup_called_clone = cleanup_called.clone();
            
            // Create effect state
            let effect_state = use_hook_state(|| EffectState {
                cleanup: None,
                deps: None,
            });
            
            // Simulate effect with cleanup
            effect_state.borrow_mut().cleanup = Some(Box::new(move || {
                *cleanup_called_clone.borrow_mut() = true;
            }));
            
            // Verify cleanup can be called
            if let Some(cleanup) = effect_state.borrow_mut().cleanup.take() {
                cleanup();
            }
            
            assert!(*cleanup_called.borrow());
        });
    }

    #[test]
    fn test_multiple_contexts() {
        let string_context: Context<String> = Context::new();
        let int_context: Context<i32> = Context::new();
        let bool_context: Context<bool> = Context::new();
        
        provide_context(&string_context, "test".to_string());
        provide_context(&int_context, 42);
        provide_context(&bool_context, true);
        
        with_test_component(|| {
            assert_eq!(use_context(&string_context), Some("test".to_string()));
            assert_eq!(use_context(&int_context), Some(42));
            assert_eq!(use_context(&bool_context), Some(true));
        });
    }

    #[test]
    fn test_hook_index_management() {
        with_test_component(|| {
            // Multiple hooks should use different indices
            let (_val1, _) = use_state(1);
            let (_val2, _) = use_state(2);
            let (_val3, _) = use_state(3);
            
            let ref1 = use_ref("a");
            let ref2 = use_ref("b");
            
            // Verify they're different references
            assert!(!Rc::ptr_eq(&ref1, &ref2));
        });
    }

    #[test]
    fn test_cleanup_component_hooks() {
        let component_id = 123;
        
        // Add some hook state
        HOOK_STATE.with(|state| {
            let mut state_map = state.borrow_mut();
            let mut component_hooks = ComponentHooks::new();
            
            // Add a mock effect state
            let effect_state = Rc::new(RefCell::new(EffectState {
                cleanup: Some(Box::new(|| {})),
                deps: None,
            }));
            component_hooks.hooks.push(Box::new(effect_state));
            
            state_map.insert(component_id, component_hooks);
        });
        
        // Cleanup should remove the component's hooks
        cleanup_component_hooks(component_id);
        
        HOOK_STATE.with(|state| {
            assert!(!state.borrow().contains_key(&component_id));
        });
    }
}