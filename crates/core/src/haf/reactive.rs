//! HAF-compliant Reactive System
//! 
//! This module provides a layered reactive system following HAF principles:
//! - L1: Pure signal values and computations
//! - L2: Effect scheduling and dependency tracking
//! - L3: DOM bindings and external subscriptions

use crate::haf::{layers::{L1, L2, L3}, Layer, Contract};
use std::marker::PhantomData;
use std::collections::HashSet;

// Type aliases to simplify complex types
type SignalCallback = Box<dyn Fn(&dyn std::any::Any)>;
type ElementUpdateFn = Box<dyn Fn(&web_sys::Element, &dyn std::any::Any)>;
type SignalSubscriptions = std::collections::HashMap<SignalId, Vec<SignalCallback>>;
type ComputeFn<T> = Box<dyn Fn(&[SignalId]) -> T>;

// ==================== L1: Pure Signal Layer ====================

/// Pure signal value (L1)
#[derive(Debug, Clone)]
pub struct Signal<T, L: Layer> {
    pub value: T,
    pub id: SignalId,
    _layer: PhantomData<L>,
}

/// Signal identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignalId(pub u64);

/// Computed value (L1)
pub struct Computed<T, L: Layer> {
    pub compute: ComputeFn<T>,
    pub dependencies: Vec<SignalId>,
    pub value: T,
    pub id: SignalId,
    _layer: PhantomData<L>,
}

impl<T: Clone, L: Layer> Signal<T, L> {
    /// Create new signal (pure)
    pub fn new(value: T, id: SignalId) -> Self {
        Self {
            value,
            id,
            _layer: PhantomData,
        }
    }
    
    /// Get current value (pure)
    pub fn get(&self) -> T {
        self.value.clone()
    }
    
    /// Create new signal with updated value (pure)
    pub fn with_value(&self, value: T) -> Self {
        Self {
            value,
            id: self.id,
            _layer: PhantomData,
        }
    }
}

impl<T: Clone> Computed<T, L1> {
    /// Create new computed value (pure)
    pub fn new<F>(id: SignalId, dependencies: Vec<SignalId>, compute: F, initial: T) -> Self 
    where
        F: Fn(&[SignalId]) -> T + 'static,
    {
        Self {
            compute: Box::new(compute),
            dependencies,
            value: initial,
            id,
            _layer: PhantomData,
        }
    }
    
    /// Recompute value (pure)
    pub fn recompute(&self, _signal_values: &[(SignalId, Box<dyn std::any::Any>)]) -> T {
        // In real implementation, would look up signal values
        (self.compute)(&self.dependencies)
    }
}

/// Pure reactive graph operations
pub mod graph {
    use super::*;
    
    /// Dependency graph (pure data structure)
    #[derive(Debug, Default)]
    pub struct DependencyGraph {
        /// Signal -> Dependents mapping
        pub dependents: std::collections::HashMap<SignalId, Vec<SignalId>>,
        /// Signal -> Dependencies mapping
        pub dependencies: std::collections::HashMap<SignalId, Vec<SignalId>>,
    }
    
    impl DependencyGraph {
        /// Add dependency relationship (pure)
        pub fn add_dependency(&self, signal: SignalId, depends_on: SignalId) -> Self {
            let mut graph = self.clone();
            
            graph.dependencies
                .entry(signal)
                .or_default()
                .push(depends_on);
                
            graph.dependents
                .entry(depends_on)
                .or_default()
                .push(signal);
                
            graph
        }
        
        /// Get signals that depend on given signal (pure)
        pub fn get_dependents(&self, signal: SignalId) -> Vec<SignalId> {
            self.dependents
                .get(&signal)
                .cloned()
                .unwrap_or_default()
        }
        
        /// Topological sort for update order (pure)
        pub fn topological_sort(&self, changed: SignalId) -> Vec<SignalId> {
            let mut visited = HashSet::new();
            let mut stack = vec![changed];
            let mut result = Vec::new();
            
            while let Some(signal) = stack.pop() {
                if visited.insert(signal) {
                    result.push(signal);
                    for dependent in self.get_dependents(signal) {
                        stack.push(dependent);
                    }
                }
            }
            
            result
        }
    }
    
    impl Clone for DependencyGraph {
        fn clone(&self) -> Self {
            Self {
                dependents: self.dependents.clone(),
                dependencies: self.dependencies.clone(),
            }
        }
    }
}

// ==================== L2: Reactive Runtime Layer ====================

/// Effect tracking (L2)
pub struct EffectTracker<L: Layer> {
    pub id: EffectId,
    pub dependencies: Vec<SignalId>,
    pub cleanup: Option<Box<dyn FnOnce()>>,
    _layer: PhantomData<L>,
}

/// Effect identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectId(pub u64);

/// Signal runtime (L2)
pub struct SignalRuntime<L: Layer> {
    /// Tracked effects
    effects: Vec<EffectTracker<L>>,
    /// Current tracking context
    tracking_context: Option<EffectId>,
    /// Scheduled updates
    update_queue: Vec<SignalId>,
    /// Batch depth
    batch_depth: usize,
    _layer: PhantomData<L>,
}

impl Default for SignalRuntime<L2> {
    fn default() -> Self {
        Self::new()
    }
}

impl SignalRuntime<L2> {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            tracking_context: None,
            update_queue: Vec::new(),
            batch_depth: 0,
            _layer: PhantomData,
        }
    }
    
    /// Start tracking dependencies
    pub fn start_tracking(&mut self, effect_id: EffectId) {
        self.tracking_context = Some(effect_id);
    }
    
    /// Stop tracking dependencies
    pub fn stop_tracking(&mut self) -> Vec<SignalId> {
        self.tracking_context = None;
        // Return collected dependencies
        vec![]
    }
    
    /// Register signal access
    pub fn track_signal_access(&mut self, signal_id: SignalId) {
        if let Some(effect_id) = self.tracking_context {
            // Record dependency
            if let Some(effect) = self.effects.iter_mut().find(|e| e.id == effect_id) {
                if !effect.dependencies.contains(&signal_id) {
                    effect.dependencies.push(signal_id);
                }
            }
        }
    }
    
    /// Schedule update
    pub fn schedule_update(&mut self, signal_id: SignalId) {
        self.update_queue.push(signal_id);
        
        if self.batch_depth == 0 {
            self.flush_updates();
        }
    }
    
    /// Batch updates
    pub fn batch<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R
    {
        self.batch_depth += 1;
        let result = f();
        self.batch_depth -= 1;
        
        if self.batch_depth == 0 {
            self.flush_updates();
        }
        
        result
    }
    
    /// Flush pending updates
    pub fn flush_updates(&mut self) -> Vec<Contract<SignalId, EffectExecution>> {
        let updates = std::mem::take(&mut self.update_queue);
        
        updates.into_iter().map(|signal_id| {
            Contract::new(
                signal_id,
                EffectExecution {
                    effects_to_run: vec![], // Would calculate affected effects
                }
            )
        }).collect()
    }
    
    /// Create effect
    pub fn create_effect(&mut self, id: EffectId) -> EffectHandle {
        self.effects.push(EffectTracker {
            id,
            dependencies: Vec::new(),
            cleanup: None,
            _layer: PhantomData,
        });
        
        EffectHandle { id }
    }
    
    /// Dispose effect
    pub fn dispose_effect(&mut self, handle: EffectHandle) {
        if let Some(pos) = self.effects.iter().position(|e| e.id == handle.id) {
            let effect = self.effects.remove(pos);
            if let Some(cleanup) = effect.cleanup {
                cleanup();
            }
        }
    }
}

/// Effect handle
pub struct EffectHandle {
    id: EffectId,
}

/// Effect execution descriptor
pub struct EffectExecution {
    pub effects_to_run: Vec<EffectId>,
}

/// Signal update contract (L1 → L2)
pub struct SignalUpdateContract;

impl SignalUpdateContract {
    pub fn transform(signal: Signal<i32, L1>) -> Contract<Signal<i32, L1>, SignalRuntime<L2>> {
        let mut runtime = SignalRuntime::new();
        runtime.schedule_update(signal.id);
        Contract::new(signal, runtime)
    }
}

// ==================== L3: Reactive Bindings Layer ====================

/// DOM binding (L3)
pub struct DomBinding<L: Layer> {
    pub element: web_sys::Element,
    pub signal_id: SignalId,
    pub update_fn: ElementUpdateFn,
    _layer: PhantomData<L>,
}

/// Reactive system (L3)
pub struct ReactiveSystem<L: Layer> {
    /// Signal runtime contract
    #[allow(dead_code)]
    runtime: Contract<SignalRuntime<L2>, ()>,
    /// DOM bindings
    bindings: Vec<DomBinding<L>>,
    /// Signal subscriptions
    subscriptions: SignalSubscriptions,
    _layer: PhantomData<L>,
}

impl Default for ReactiveSystem<L3> {
    fn default() -> Self {
        Self::new()
    }
}

impl ReactiveSystem<L3> {
    pub fn new() -> Self {
        Self {
            runtime: Contract::new(SignalRuntime::new(), ()),
            bindings: Vec::new(),
            subscriptions: SignalSubscriptions::new(),
            _layer: PhantomData,
        }
    }
    
    /// Bind signal to DOM element
    pub fn bind_to_dom<T: 'static>(
        &mut self,
        signal_id: SignalId,
        element: web_sys::Element,
        update_fn: impl Fn(&web_sys::Element, &T) + 'static,
    ) {
        self.bindings.push(DomBinding {
            element,
            signal_id,
            update_fn: Box::new(move |el: &web_sys::Element, val: &dyn std::any::Any| {
                if let Some(typed_val) = val.downcast_ref::<T>() {
                    update_fn(el, typed_val);
                }
            }) as ElementUpdateFn,
            _layer: PhantomData,
        });
    }
    
    /// Subscribe to signal changes
    pub fn subscribe<T: 'static>(
        &mut self,
        signal_id: SignalId,
        callback: impl Fn(&T) + 'static,
    ) {
        self.subscriptions
            .entry(signal_id)
            .or_default()
            .push(Box::new(move |val: &dyn std::any::Any| {
                if let Some(typed_val) = val.downcast_ref::<T>() {
                    callback(typed_val);
                }
            }) as SignalCallback);
    }
    
    /// Execute effect contracts from runtime
    pub fn execute_effects(&mut self, contracts: Vec<Contract<SignalId, EffectExecution>>) {
        for contract in contracts {
            let signal_id = contract.input;
            
            // Update DOM bindings
            for binding in &self.bindings {
                if binding.signal_id == signal_id {
                    // In real implementation, would get signal value
                    // (binding.update_fn)(&binding.element, &value);
                }
            }
            
            // Call subscriptions
            if let Some(subs) = self.subscriptions.get(&signal_id) {
                for _sub in subs {
                    // sub(&value);
                }
            }
        }
    }
}

// ==================== Example Usage ====================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pure_signal_operations() {
        // L1: Pure signal operations
        let signal = Signal::<i32, L1>::new(42, SignalId(1));
        assert_eq!(signal.get(), 42);
        
        let updated = signal.with_value(100);
        assert_eq!(updated.get(), 100);
        assert_eq!(signal.get(), 42); // Original unchanged
    }
    
    #[test]
    fn test_dependency_graph() {
        use graph::DependencyGraph;
        
        let graph = DependencyGraph::default();
        let graph = graph.add_dependency(SignalId(2), SignalId(1));
        let graph = graph.add_dependency(SignalId(3), SignalId(2));
        
        let deps = graph.get_dependents(SignalId(1));
        assert_eq!(deps, vec![SignalId(2)]);
        
        let order = graph.topological_sort(SignalId(1));
        assert_eq!(order, vec![SignalId(1), SignalId(2), SignalId(3)]);
    }
    
    #[test]
    fn test_runtime_effect_tracking() {
        let mut runtime = SignalRuntime::<L2>::new();
        
        let effect_id = EffectId(1);
        let _handle = runtime.create_effect(effect_id);
        
        runtime.start_tracking(effect_id);
        runtime.track_signal_access(SignalId(1));
        runtime.track_signal_access(SignalId(2));
        let _deps = runtime.stop_tracking();
        
        // Verify dependencies were tracked
        assert_eq!(runtime.effects[0].dependencies.len(), 2);
    }
}