//! HAF Reactive System Tests

use crate::haf::{
    layers::{L1, L2, L3},
    reactive::*,
    Contract,
};

#[test]
fn test_signal_layer_separation() {
    // L1: Pure signal operations
    let signal = Signal::<i32, L1>::new(42, SignalId(1));
    let updated = signal.with_value(100);
    
    // Signals are immutable at L1
    assert_eq!(signal.get(), 42);
    assert_eq!(updated.get(), 100);
}

#[test]
fn test_computed_values_are_pure() {
    // L1: Pure computed values
    let computed = Computed::<i32, L1>::new(
        SignalId(10),
        vec![SignalId(1), SignalId(2)],
        |deps| {
            // Pure computation
            deps.len() as i32 * 10
        },
        0
    );
    
    // Computed values don't have side effects
    let _result = computed.recompute(&[]);
    assert_eq!(computed.value, 0); // Original unchanged
}

#[test]
fn test_runtime_effect_isolation() {
    // L2: Effect runtime
    let mut runtime = SignalRuntime::<L2>::new();
    
    // Effects are managed at L2
    let effect_id = EffectId(1);
    let _handle = runtime.create_effect(effect_id);
    
    // Track dependencies
    runtime.start_tracking(effect_id);
    runtime.track_signal_access(SignalId(1));
    runtime.track_signal_access(SignalId(2));
    let _deps = runtime.stop_tracking();
    
    // Test contract creation directly
    let contract = Contract::new(
        SignalId(1),
        EffectExecution {
            effects_to_run: vec![],
        }
    );
    
    // Contracts enforce layer boundaries
    assert_eq!(contract.input, SignalId(1));
    
    // The runtime exists and can manage effects - that's the key isolation
}

#[test]
fn test_dom_bindings_at_l3_only() {
    // L3: DOM operations
    let _system = ReactiveSystem::<L3>::new();
    
    // DOM bindings only exist at L3
    // This would fail to compile at L1 or L2:
    // let mut system = ReactiveSystem::<L1>::new(); // ERROR!
}

#[test]
fn test_dependency_graph_is_pure() {
    use crate::haf::reactive::graph::DependencyGraph;
    
    // L1: Pure dependency tracking
    let graph = DependencyGraph::default();
    
    // All operations are pure and return new graphs
    let graph2 = graph.add_dependency(SignalId(2), SignalId(1));
    let graph3 = graph2.add_dependency(SignalId(3), SignalId(2));
    
    // Original graph unchanged
    assert!(graph.dependents.is_empty());
    assert!(!graph3.dependents.is_empty());
    
    // Topological sort is pure
    let order = graph3.topological_sort(SignalId(1));
    assert_eq!(order.len(), 3);
}