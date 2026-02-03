// Simple test to verify DI container works
use crate::di::{DiContainer, ServiceLifetime};

#[test]
fn test_simple_di_container() {
    let mut container = DiContainer::new();

    // Register a simple service
    container.register(|_| String::from("test_service"), ServiceLifetime::Transient);

    // Resolve the service
    let service = container.resolve::<String>();
    assert!(service.is_some());
    assert_eq!(service.unwrap(), "test_service");
}

#[test]
fn test_singleton_service() {
    let mut container = DiContainer::new();

    // Register a singleton service
    container.register(|_| String::from("singleton"), ServiceLifetime::Singleton);

    // Resolve the service twice
    let service1 = container.resolve::<String>();
    let service2 = container.resolve::<String>();

    assert!(service1.is_some());
    assert!(service2.is_some());
    assert_eq!(service1.unwrap(), service2.unwrap());
}

#[test]
fn test_unregistered_service() {
    let mut container = DiContainer::new();

    // Try to resolve an unregistered service
    let service = container.resolve::<String>();
    assert!(service.is_none());
}
