//! HAF Test Suite

mod reactive_tests;
mod basic_tests;

#[cfg(test)]
mod contract_tests {
    use crate::haf::Contract;
    
    #[test]
    fn test_contract_transformation() {
        // Contracts enforce type-safe layer transitions
        let input = "L1 Data";
        let output = "L2 Processed";
        
        let contract = Contract::new(input, output);
        
        assert_eq!(contract.input, "L1 Data");
        assert_eq!(contract.output, "L2 Processed");
    }
}

#[cfg(test)]
mod compile_time_tests {
    use crate::haf::{layers::*, Layer, CanDepend};
    
    // These tests verify compile-time enforcement
    
    fn assert_can_depend<From: CanDepend<To>, To: Layer>() {}
    
    #[test]
    fn test_valid_dependencies_compile() {
        // These should compile
        assert_can_depend::<L3, L2>();
        assert_can_depend::<L3, L1>();
        assert_can_depend::<L2, L1>();
        assert_can_depend::<L1, L1>();
        assert_can_depend::<L2, L2>();
        assert_can_depend::<L3, L3>();
    }
    
    // Uncomment to verify these fail to compile:
    // #[test]
    // fn test_invalid_dependencies_fail() {
    //     assert_can_depend::<L1, L2>(); // ERROR: L1 cannot depend on L2
    //     assert_can_depend::<L1, L3>(); // ERROR: L1 cannot depend on L3
    //     assert_can_depend::<L2, L3>(); // ERROR: L2 cannot depend on L3
    // }
}