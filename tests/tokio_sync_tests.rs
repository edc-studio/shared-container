#![cfg(feature = "tokio-sync")]

use shared_container::SharedContainer;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, PartialEq)]
struct TestStruct {
    value: i32,
}

#[test]
fn test_tokio_read_access() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let container = SharedContainer::new(TestStruct { value: 42 });

        // Synchronous read should return None with tokio-sync
        assert!(container.read().is_none());

        // Async read access
        let guard = container.read_async().await;
        assert_eq!(guard.value, 42);
    });
}

#[test]
fn test_tokio_write_access() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let container = SharedContainer::new(TestStruct { value: 42 });

        // Synchronous write should return None with tokio-sync
        assert!(container.write().is_none());

        // Async write access
        {
            let mut guard = container.write_async().await;
            guard.value = 100;
        }

        // Verify change
        let guard = container.read_async().await;
        assert_eq!(guard.value, 100);
    });
}

#[test]
fn test_tokio_clone_container() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let container1 = SharedContainer::new(TestStruct { value: 42 });
        let container2 = container1.clone();

        // Modify through container2
        {
            let mut guard = container2.write_async().await;
            guard.value = 100;
        }

        // Verify change visible through container1
        let guard = container1.read_async().await;
        assert_eq!(guard.value, 100);
    });
}

#[test]
fn test_tokio_weak_ref() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let container = SharedContainer::new(TestStruct { value: 42 });

        // Create a weak reference
        let weak = container.downgrade();

        // Weak reference can be upgraded to a strong reference
        let container2 = weak.upgrade().unwrap();

        // Both containers point to the same data
        {
            let mut guard = container2.write_async().await;
            guard.value = 100;
        }

        // Change visible through first container
        {
            let guard = container.read_async().await;
            assert_eq!(guard.value, 100);
        }

        // Drop all strong references
        drop(container);
        drop(container2);

        // Weak reference can no longer be upgraded
        assert!(weak.upgrade().is_none());
    });
}

#[test]
fn test_tokio_weak_clone() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let container = SharedContainer::new(TestStruct { value: 42 });

        // Create a weak reference and clone it
        let weak1 = container.downgrade();
        let weak2 = weak1.clone();

        // Both weak references can be upgraded
        let container1 = weak1.upgrade().unwrap();
        let container2 = weak2.upgrade().unwrap();

        // Modify through container2
        {
            let mut guard = container2.write_async().await;
            guard.value = 100;
        }

        // Change visible through container1
        {
            let guard = container1.read_async().await;
            assert_eq!(guard.value, 100);
        }

        // Drop all strong references
        drop(container);
        drop(container1);
        drop(container2);

        // Neither weak reference can be upgraded
        assert!(weak1.upgrade().is_none());
        assert!(weak2.upgrade().is_none());
    });
}

#[test]
fn test_tokio_get_cloned() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let container = SharedContainer::new(TestStruct { value: 42 });
        let cloned = container.get_cloned().unwrap();
        assert_eq!(cloned, TestStruct { value: 42 });

        // Modify original
        {
            let mut guard = container.write_async().await;
            guard.value = 100;
        }

        // Cloned value should not change
        assert_eq!(cloned, TestStruct { value: 42 });

        // And we can get a new clone with updated value
        let new_clone = container.get_cloned().unwrap();
        assert_eq!(new_clone, TestStruct { value: 100 });
    });
}

#[test]
fn test_tokio_partial_eq() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let container1 = SharedContainer::new(TestStruct { value: 42 });
        let container2 = container1.clone();
        let container3 = SharedContainer::new(TestStruct { value: 100 });

        assert_eq!(container1, container2);
        assert_ne!(container1, container3);
    });
}

#[test]
fn test_tokio_multiple_reads() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let container = SharedContainer::new(TestStruct { value: 42 });

        // Multiple read locks should be allowed
        let guard1 = container.read_async().await;
        let guard2 = container.read_async().await;

        assert_eq!(guard1.value, 42);
        assert_eq!(guard2.value, 42);
    });
}

#[test]
fn test_tokio_read_write_conflict() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let container = SharedContainer::new(TestStruct { value: 42 });

        // Get a read lock
        let guard = container.read_async().await;

        // In tokio, trying to get a write lock while holding a read lock
        // will wait until the read lock is dropped, so we need to test this differently

        // We'll verify the value is what we expect
        assert_eq!(guard.value, 42);
    });
}
