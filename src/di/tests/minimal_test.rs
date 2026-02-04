// Minimal test to isolate the DI issue
use crate::di::{DiContainer, ServiceLifetime};

#[test]
fn test_minimal_di() {
    let mut container = DiContainer::new();

    // Register a simple i32 service
    container.register(|_| 42i32, ServiceLifetime::Singleton);

    // Check if service is registered
    assert!(container.is_registered::<i32>());
    println!(
        "Service is registered: {}",
        container.is_registered::<i32>()
    );

    // First resolution
    let result1 = container.resolve::<i32>();
    println!("First resolution: {:?}", result1);
    assert!(result1.is_some(), "First resolution should succeed");

    // Second resolution
    let result2 = container.resolve::<i32>();
    println!("Second resolution: {:?}", result2);
    assert!(result2.is_some(), "Second resolution should succeed");

    let val1 = result1.unwrap();
    let val2 = result2.unwrap();

    assert_eq!(val1, 42);
    assert_eq!(val2, 42);
    assert_eq!(container.singleton_count(), 1);
}
