// Simple singleton test to debug the issue
use crate::di::{DiContainer, ServiceLifetime};

#[derive(Debug, Clone, PartialEq, Copy)]
struct SimpleTestService {
    id: usize,
}

impl SimpleTestService {
    fn new() -> Self {
        Self { id: 1 }
    }
}

#[test]
fn test_simple_singleton_service() {
    let mut container = DiContainer::new();

    // Register a simple service
    container.register(|_| SimpleTestService::new(), ServiceLifetime::Singleton);

    // Check if service is registered
    assert!(container.is_registered::<SimpleTestService>());

    // Try to resolve the service
    let service1 = container.resolve::<SimpleTestService>();
    assert!(
        service1.is_some(),
        "First service resolution should succeed"
    );

    let service2 = container.resolve::<SimpleTestService>();
    assert!(
        service2.is_some(),
        "Second service resolution should succeed"
    );

    let service1 = service1.unwrap();
    let service2 = service2.unwrap();

    // For singleton, both should be the same
    assert_eq!(service1.id, service2.id);
    assert_eq!(service1.id, 1);
    assert_eq!(container.singleton_count(), 1);
}
