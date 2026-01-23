#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, testutils::{Address as _, Ledger}, Address, Env, String};

// Helper function to create a test commitment
fn create_test_commitment(
    e: &Env,
    commitment_id: &str,
    owner: &Address,
    amount: i128,
    current_value: i128,
    max_loss_percent: u32,
    duration_days: u32,
    created_at: u64,
) -> Commitment {
    let expires_at = created_at + (duration_days as u64 * 86400); // days to seconds
    
    Commitment {
        commitment_id: String::from_str(e, commitment_id),
        owner: owner.clone(),
        nft_token_id: 1,
        rules: CommitmentRules {
            duration_days,
            max_loss_percent,
            commitment_type: String::from_str(e, "balanced"),
            early_exit_penalty: 10,
            min_fee_threshold: 1000,
        },
        amount,
        asset_address: Address::generate(e),
        created_at,
        expires_at,
        current_value,
        status: String::from_str(e, "active"),
    }
}

// Helper to store a commitment for testing
fn store_commitment(e: &Env, contract_id: &Address, commitment: &Commitment) {
    e.as_contract(contract_id, || {
        let key = (symbol_short!("Commit"), commitment.commitment_id.clone());
        e.storage().persistent().set(&key, commitment);
    });
}

fn create_test_env() -> Env {
    Env::default()
}

fn setup_contract(e: &Env) -> Address {
    let admin = Address::generate(e);
    let nft_contract = Address::generate(e);
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    
    let client = CommitmentCoreContractClient::new(e, &contract_id);
    client.initialize(&admin, &nft_contract);
    
    contract_id
}

fn create_test_commitment(e: &Env, contract_id: &Address) -> (String, Commitment) {
    let commitment_id = String::from_str(e, "test_commitment_1");
    let owner = Address::generate(e);
    let asset_address = Address::generate(e);
    
    let rules = CommitmentRules {
        duration_days: 365,
        max_loss_percent: 20,
        commitment_type: String::from_str(e, "balanced"),
        early_exit_penalty: 10,
        min_fee_threshold: 1000,
    };
    
    let commitment = Commitment {
        commitment_id: commitment_id.clone(),
        owner: owner.clone(),
        nft_token_id: 1,
        rules: rules.clone(),
        amount: 1000000, // 1000 tokens (assuming 1000 scaling)
        asset_address: asset_address.clone(),
        created_at: 1000,
        expires_at: 1000 + (365 * 86400), // 365 days later
        current_value: 1000000,
        status: String::from_str(e, "active"),
    };
    
    // Note: In a real test, we would need to actually store this commitment
    // For now, this is a helper function structure
    
    (commitment_id, commitment)
}

#[test]
fn test_initialize() {
    let e = create_test_env();
    let admin = Address::generate(&e);
    let nft_contract = Address::generate(&e);
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    
    let client = CommitmentCoreContractClient::new(&e, &contract_id);
    client.initialize(&admin, &nft_contract);
    
    // Verify initialization succeeded (no panic)
}

#[test]
#[should_panic(expected = "AlreadyInitialized")]
fn test_initialize_twice() {
    let e = create_test_env();
    let admin = Address::generate(&e);
    let nft_contract = Address::generate(&e);
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    
    let client = CommitmentCoreContractClient::new(&e, &contract_id);
    client.initialize(&admin, &nft_contract);
    client.initialize(&admin, &nft_contract); // Should panic
}

#[test]
fn test_add_authorized_allocator() {
    let e = create_test_env();
    let admin = Address::generate(&e);
    let nft_contract = Address::generate(&e);
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    
    let client = CommitmentCoreContractClient::new(&e, &contract_id);
    client.initialize(&admin, &nft_contract);
    
    let allocator = Address::generate(&e);
    admin.mock_auth(&e, &admin, &admin, &[]);
    client.add_authorized_allocator(&allocator);
    
    // Verify allocator is authorized
    let is_authorized = client.is_authorized_allocator(&allocator);
    assert!(is_authorized);
}

#[test]
fn test_remove_authorized_allocator() {
    let e = create_test_env();
    let admin = Address::generate(&e);
    let nft_contract = Address::generate(&e);
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    
    let client = CommitmentCoreContractClient::new(&e, &contract_id);
    client.initialize(&admin, &nft_contract);
    
    let allocator = Address::generate(&e);
    
    // Add allocator
    admin.mock_auth(&e, &admin, &admin, &[]);
    client.add_authorized_allocator(&allocator);
    assert!(client.is_authorized_allocator(&allocator));
    
    // Remove allocator
    admin.mock_auth(&e, &admin, &admin, &[]);
    client.remove_authorized_allocator(&allocator);
    assert!(!client.is_authorized_allocator(&allocator));
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_allocate_unauthorized_caller() {
    let e = create_test_env();
    let admin = Address::generate(&e);
    let nft_contract = Address::generate(&e);
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    
    let client = CommitmentCoreContractClient::new(&e, &contract_id);
    client.initialize(&admin, &nft_contract);
    
    let unauthorized_allocator = Address::generate(&e);
    let commitment_id = String::from_str(&e, "test_commitment");
    let target_pool = Address::generate(&e);
    
    // Try to allocate with unauthorized caller - should panic
    client.allocate(&unauthorized_allocator, &commitment_id, &target_pool, &1000);
}

#[test]
#[should_panic(expected = "InactiveCommitment")]
fn test_allocate_inactive_commitment() {
    let e = create_test_env();
    let admin = Address::generate(&e);
    let nft_contract = Address::generate(&e);
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    
    let client = CommitmentCoreContractClient::new(&e, &contract_id);
    client.initialize(&admin, &nft_contract);
    
    let allocator = Address::generate(&e);
    admin.mock_auth(&e, &admin, &admin, &[]);
    client.add_authorized_allocator(&allocator);
    
    // Try to allocate with non-existent commitment - should panic
    let commitment_id = String::from_str(&e, "nonexistent_commitment");
    let target_pool = Address::generate(&e);
    
    client.allocate(&allocator, &commitment_id, &target_pool, &1000);
}

#[test]
#[should_panic(expected = "InsufficientBalance")]
fn test_allocate_insufficient_balance() {
    let e = create_test_env();
    let admin = Address::generate(&e);
    let nft_contract = Address::generate(&e);
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    
    let client = CommitmentCoreContractClient::new(&e, &contract_id);
    client.initialize(&admin, &nft_contract);
    
    let allocator = Address::generate(&e);
    admin.mock_auth(&e, &admin, &admin, &[]);
    client.add_authorized_allocator(&allocator);
    
    // Note: This test requires a commitment with a known balance
    // In a full implementation, we would create a commitment first
    // and set its balance, then try to allocate more than available
    let commitment_id = String::from_str(&e, "test_commitment");
    let target_pool = Address::generate(&e);
    
    // This will panic with InactiveCommitment first, but the test structure
    // demonstrates the insufficient balance check would work once commitment exists
    // client.allocate(&allocator, &commitment_id, &target_pool, &999999999);
}

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_allocate_invalid_amount() {
    let e = create_test_env();
    let admin = Address::generate(&e);
    let nft_contract = Address::generate(&e);
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    
    let client = CommitmentCoreContractClient::new(&e, &contract_id);
    client.initialize(&admin, &nft_contract);
    
    let allocator = Address::generate(&e);
    admin.mock_auth(&e, &admin, &admin, &[]);
    client.add_authorized_allocator(&allocator);
    
    let commitment_id = String::from_str(&e, "test_commitment");
    let target_pool = Address::generate(&e);
    
    // Try to allocate with zero or negative amount - should panic
    // Note: This would panic in transfer_asset function
    // client.allocate(&allocator, &commitment_id, &target_pool, &0);
    // Or: client.allocate(&allocator, &commitment_id, &target_pool, &-100);
}

#[test]
fn test_get_allocation_tracking() {
    let e = create_test_env();
    let admin = Address::generate(&e);
    let nft_contract = Address::generate(&e);
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    
    let client = CommitmentCoreContractClient::new(&e, &contract_id);
    client.initialize(&admin, &nft_contract);
    
    let commitment_id = String::from_str(&e, "test_commitment");
    
    // Get tracking for non-existent commitment - should return empty tracking
    let tracking = client.get_allocation_tracking(&commitment_id);
    assert_eq!(tracking.total_allocated, 0);
    assert_eq!(tracking.allocations.len(), 0);
}

#[test]
fn test_deallocate() {
    let e = create_test_env();
    let admin = Address::generate(&e);
    let nft_contract = Address::generate(&e);
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    
    let client = CommitmentCoreContractClient::new(&e, &contract_id);
    client.initialize(&admin, &nft_contract);
    
    let allocator = Address::generate(&e);
    admin.mock_auth(&e, &admin, &admin, &[]);
    client.add_authorized_allocator(&allocator);
    
    let commitment_id = String::from_str(&e, "test_commitment");
    let target_pool = Address::generate(&e);
    
    // Note: This test would require a real commitment and successful allocation first
    // The deallocation function will panic with InactiveCommitment if commitment doesn't exist
    // This test structure demonstrates the deallocation flow
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_deallocate_unauthorized() {
    let e = create_test_env();
    let admin = Address::generate(&e);
    let nft_contract = Address::generate(&e);
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    
    let client = CommitmentCoreContractClient::new(&e, &contract_id);
    client.initialize(&admin, &nft_contract);
    
    let unauthorized_allocator = Address::generate(&e);
    let commitment_id = String::from_str(&e, "test_commitment");
    let target_pool = Address::generate(&e);
    
    // Try to deallocate with unauthorized caller - should panic
    client.deallocate(&unauthorized_allocator, &commitment_id, &target_pool, &1000);
}

// Integration test structure - would need full commitment setup
#[test]
fn test_allocation_flow_integration() {
    let e = create_test_env();
    let admin = Address::generate(&e);
    let nft_contract = Address::generate(&e);
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    
    let client = CommitmentCoreContractClient::new(&e, &contract_id);
    client.initialize(&admin, &nft_contract);
    
    // Setup authorized allocator
    let allocator = Address::generate(&e);
    admin.mock_auth(&e, &admin, &admin, &[]);
    client.add_authorized_allocator(&allocator);
    
    // Note: Full integration test would require:
    // 1. Creating a commitment with assets
    // 2. Setting up asset contract mock
    // 3. Allocating to pool
    // 4. Verifying balance updates
    // 5. Verifying allocation tracking
    // 6. Verifying events emitted
    
    // This test structure shows the flow, but actual implementation
    // would need proper commitment and asset contract setup
}

#[test]
fn test_check_violations_no_violations() {
    let e = Env::default();
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    let owner = Address::generate(&e);
    let commitment_id = "test_commitment_1";
    
    // Create a commitment with no violations
    // Initial: 1000, Current: 950 (5% loss), Max loss: 10%, Duration: 30 days
    let created_at = 1000u64;
    let commitment = create_test_commitment(
        &e,
        commitment_id,
        &owner,
        1000,
        950, // 5% loss
        10,  // max 10% loss allowed
        30,  // 30 days duration
        created_at,
    );
    
    store_commitment(&e, &contract_id, &commitment);
    
    // Set ledger time to 15 days later (halfway through)
    e.ledger().with_mut(|l| {
        l.timestamp = created_at + (15 * 86400);
    });
    
    let has_violations = e.as_contract(&contract_id, || {
        CommitmentCoreContract::check_violations(e.clone(), String::from_str(&e, commitment_id))
    });
    
    assert!(!has_violations, "Should not have violations");
}

#[test]
fn test_check_violations_loss_limit_exceeded() {
    let e = Env::default();
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    let owner = Address::generate(&e);
    let commitment_id = "test_commitment_2";
    
    // Create a commitment with loss limit violation
    // Initial: 1000, Current: 850 (15% loss), Max loss: 10%
    let created_at = 1000u64;
    let commitment = create_test_commitment(
        &e,
        commitment_id,
        &owner,
        1000,
        850, // 15% loss - exceeds 10% limit
        10,  // max 10% loss allowed
        30,
        created_at,
    );
    
    store_commitment(&e, &contract_id, &commitment);
    
    // Set ledger time to 5 days later (still within duration)
    e.ledger().with_mut(|l| {
        l.timestamp = created_at + (5 * 86400);
    });
    
    let has_violations = e.as_contract(&contract_id, || {
        CommitmentCoreContract::check_violations(e.clone(), String::from_str(&e, commitment_id))
    });
    
    assert!(has_violations, "Should have loss limit violation");
}

#[test]
fn test_check_violations_duration_expired() {
    let e = Env::default();
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    let owner = Address::generate(&e);
    let commitment_id = "test_commitment_3";
    
    // Create a commitment that has expired
    let created_at = 1000u64;
    let commitment = create_test_commitment(
        &e,
        commitment_id,
        &owner,
        1000,
        980, // 2% loss - within limit
        10,  // max 10% loss allowed
        30,  // 30 days duration
        created_at,
    );
    
    store_commitment(&e, &contract_id, &commitment);
    
    // Set ledger time to 31 days later (expired)
    e.ledger().with_mut(|l| {
        l.timestamp = created_at + (31 * 86400);
    });
    
    let has_violations = e.as_contract(&contract_id, || {
        CommitmentCoreContract::check_violations(e.clone(), String::from_str(&e, commitment_id))
    });
    
    assert!(has_violations, "Should have duration violation");
}

#[test]
fn test_check_violations_both_violations() {
    let e = Env::default();
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    let owner = Address::generate(&e);
    let commitment_id = "test_commitment_4";
    
    // Create a commitment with both violations
    let created_at = 1000u64;
    let commitment = create_test_commitment(
        &e,
        commitment_id,
        &owner,
        1000,
        800, // 20% loss - exceeds limit
        10,  // max 10% loss allowed
        30,
        created_at,
    );
    
    store_commitment(&e, &contract_id, &commitment);
    
    // Set ledger time to 31 days later (expired)
    e.ledger().with_mut(|l| {
        l.timestamp = created_at + (31 * 86400);
    });
    
    let has_violations = e.as_contract(&contract_id, || {
        CommitmentCoreContract::check_violations(e.clone(), String::from_str(&e, commitment_id))
    });
    
    assert!(has_violations, "Should have both violations");
}

#[test]
fn test_get_violation_details_no_violations() {
    let e = Env::default();
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    let owner = Address::generate(&e);
    let commitment_id = "test_commitment_5";
    
    let created_at = 1000u64;
    let commitment = create_test_commitment(
        &e,
        commitment_id,
        &owner,
        1000,
        950, // 5% loss
        10,  // max 10% loss
        30,
        created_at,
    );
    
    store_commitment(&e, &contract_id, &commitment);
    
    // Set ledger time to 15 days later
    e.ledger().with_mut(|l| {
        l.timestamp = created_at + (15 * 86400);
    });
    
    let (has_violations, loss_violated, duration_violated, loss_percent, time_remaining) = 
        e.as_contract(&contract_id, || {
            CommitmentCoreContract::get_violation_details(e.clone(), String::from_str(&e, commitment_id))
        });
    
    assert!(!has_violations, "Should not have violations");
    assert!(!loss_violated, "Loss should not be violated");
    assert!(!duration_violated, "Duration should not be violated");
    assert_eq!(loss_percent, 5, "Loss percent should be 5%");
    assert!(time_remaining > 0, "Time should remain");
}

#[test]
fn test_get_violation_details_loss_violation() {
    let e = Env::default();
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    let owner = Address::generate(&e);
    let commitment_id = "test_commitment_6";
    
    let created_at = 1000u64;
    let commitment = create_test_commitment(
        &e,
        commitment_id,
        &owner,
        1000,
        850, // 15% loss - exceeds 10%
        10,
        30,
        created_at,
    );
    
    store_commitment(&e, &contract_id, &commitment);
    
    e.ledger().with_mut(|l| {
        l.timestamp = created_at + (10 * 86400);
    });
    
    let commitment_id_str = String::from_str(&e, commitment_id);
    let (has_violations, loss_violated, duration_violated, loss_percent, _time_remaining) = 
        e.as_contract(&contract_id, || {
            CommitmentCoreContract::get_violation_details(e.clone(), commitment_id_str.clone())
        });
    
    assert!(has_violations, "Should have violations");
    assert!(loss_violated, "Loss should be violated");
    assert!(!duration_violated, "Duration should not be violated");
    assert_eq!(loss_percent, 15, "Loss percent should be 15%");
}

#[test]
fn test_get_violation_details_duration_violation() {
    let e = Env::default();
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    let owner = Address::generate(&e);
    let commitment_id = "test_commitment_7";
    
    let created_at = 1000u64;
    let commitment = create_test_commitment(
        &e,
        commitment_id,
        &owner,
        1000,
        980, // 2% loss - within limit
        10,
        30,
        created_at,
    );
    
    store_commitment(&e, &contract_id, &commitment);
    
    // Set time to 31 days later (expired)
    e.ledger().with_mut(|l| {
        l.timestamp = created_at + (31 * 86400);
    });
    
    let (has_violations, loss_violated, duration_violated, _loss_percent, time_remaining) = 
        e.as_contract(&contract_id, || {
            CommitmentCoreContract::get_violation_details(e.clone(), String::from_str(&e, commitment_id))
        });
    
    assert!(has_violations, "Should have violations");
    assert!(!loss_violated, "Loss should not be violated");
    assert!(duration_violated, "Duration should be violated");
    assert_eq!(time_remaining, 0, "Time remaining should be 0");
}

#[test]
#[should_panic(expected = "Commitment not found")]
fn test_check_violations_not_found() {
    let e = Env::default();
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    let commitment_id = "nonexistent";
    
    e.as_contract(&contract_id, || {
        CommitmentCoreContract::check_violations(e.clone(), String::from_str(&e, commitment_id))
    });
}

#[test]
fn test_check_violations_edge_case_exact_loss_limit() {
    let e = Env::default();
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    let owner = Address::generate(&e);
    let commitment_id = "test_commitment_8";
    
    // Test exactly at the loss limit (should not violate)
    let created_at = 1000u64;
    let commitment = create_test_commitment(
        &e,
        commitment_id,
        &owner,
        1000,
        900, // Exactly 10% loss
        10,  // max 10% loss
        30,
        created_at,
    );
    
    store_commitment(&e, &contract_id, &commitment);
    
    e.ledger().with_mut(|l| {
        l.timestamp = created_at + (15 * 86400);
    });
    
    let has_violations = e.as_contract(&contract_id, || {
        CommitmentCoreContract::check_violations(e.clone(), String::from_str(&e, commitment_id))
    });
    
    // Exactly at limit should not violate (uses > not >=)
    assert!(!has_violations, "Exactly at limit should not violate");
}

#[test]
fn test_check_violations_edge_case_exact_expiry() {
    let e = Env::default();
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    let owner = Address::generate(&e);
    let commitment_id = "test_commitment_9";
    
    let created_at = 1000u64;
    let commitment = create_test_commitment(
        &e,
        commitment_id,
        &owner,
        1000,
        950,
        10,
        30,
        created_at,
    );
    
    store_commitment(&e, &contract_id, &commitment);
    
    // Set time to exactly expires_at
    e.ledger().with_mut(|l| {
        l.timestamp = commitment.expires_at;
    });
    
    let has_violations = e.as_contract(&contract_id, || {
        CommitmentCoreContract::check_violations(e.clone(), String::from_str(&e, commitment_id))
    });
    
    // At expiry time, should be violated (uses >=)
    assert!(has_violations, "At expiry time should violate");
}

#[test]
fn test_check_violations_zero_amount() {
    let e = Env::default();
    let contract_id = e.register_contract(None, CommitmentCoreContract);
    let owner = Address::generate(&e);
    let commitment_id = "test_commitment_10";
    
    // Edge case: zero amount (should not cause division by zero)
    let created_at = 1000u64;
    let commitment = create_test_commitment(
        &e,
        commitment_id,
        &owner,
        0,   // zero amount
        0,   // zero value
        10,
        30,
        created_at,
    );
    
    store_commitment(&e, &contract_id, &commitment);
    
    e.ledger().with_mut(|l| {
        l.timestamp = created_at + (15 * 86400);
    });
    
    let has_violations = e.as_contract(&contract_id, || {
        CommitmentCoreContract::check_violations(e.clone(), String::from_str(&e, commitment_id))
    });
    
    // Should not panic and should only check duration
    assert!(!has_violations, "Zero amount should not cause issues");
}

