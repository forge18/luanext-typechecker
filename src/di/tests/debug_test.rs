// Debug test to understand DI container issues
use crate::di::{DiContainer, ServiceLifetime};

#[derive(Debug, Clone, PartialEq)]
struct SimpleService {
    value: i32,
}

#[test]
fn test_simple_service_registration_and_resolution() {
    let mut container = DiContainer::new();

    // Register a simple service
    container.register(|_| SimpleService { value: 42 }, ServiceLifetime::Singleton);

    // Check if service is registered
    assert!(container.is_registered::<SimpleService>());

    // Try to resolve the service
    let service = container.resolve::<SimpleService>();
    assert!(service.is_some(), "Service should be resolved");

    let service = service.unwrap();
    assert_eq!(service.value, 42);
}

#[test]
fn test_service_not_registered() {
    let mut container = DiContainer::new();

    // Check that unregistered service returns None
    assert!(!container.is_registered::<SimpleService>());
    let service = container.resolve::<SimpleService>();
    assert!(service.is_none(), "Unregistered service should return None");
}
