#![cfg(any(feature = "wasm-sync", feature = "force-wasm-impl"))]

use shared_container::SharedContainer;

#[derive(Debug, Clone, PartialEq)]
struct TestStruct {
    value: i32,
}

#[test]
fn test_wasm_read_access() {
    let container = SharedContainer::new(TestStruct { value: 42 });

    // Read access
    let guard = container.read().unwrap();
    assert_eq!(guard.value, 42);
}

#[test]
fn test_wasm_write_access() {
    let container = SharedContainer::new(TestStruct { value: 42 });

    // Write access
    {
        let mut guard = container.write().unwrap();
        guard.value = 100;
    }

    // Verify change
    let guard = container.read().unwrap();
    assert_eq!(guard.value, 100);
}

#[test]
fn test_wasm_borrow_conflict() {
    let container = SharedContainer::new(TestStruct { value: 42 });

    // Get a read borrow
    let _guard = container.read().unwrap();

    // Trying to get a write borrow while a read borrow exists should fail
    assert!(container.write().is_none());
}

#[test]
fn test_wasm_multiple_reads() {
    let container = SharedContainer::new(TestStruct { value: 42 });

    // Multiple read borrows should work
    let _guard1 = container.read().unwrap();
    let guard2 = container.read().unwrap();

    assert_eq!(guard2.value, 42);
}

#[test]
fn test_wasm_weak_ref() {
    let container = SharedContainer::new(TestStruct { value: 42 });
    let weak = container.downgrade();

    // Upgrade should work
    let container2 = weak.upgrade().unwrap();
    assert_eq!(container2.read().unwrap().value, 42);

    // After dropping all strong references, upgrade should fail
    drop(container);
    drop(container2);
    assert!(weak.upgrade().is_none());
}

#[test]
fn test_wasm_clone_container() {
    let container1 = SharedContainer::new(TestStruct { value: 42 });
    let container2 = container1.clone();

    // Modify through container2
    {
        let mut guard = container2.write().unwrap();
        guard.value = 100;
    }

    // Verify change visible through container1
    let guard = container1.read().unwrap();
    assert_eq!(guard.value, 100);
}

#[test]
fn test_wasm_get_cloned() {
    let container = SharedContainer::new(TestStruct { value: 42 });
    let cloned = container.get_cloned().unwrap();
    assert_eq!(cloned, TestStruct { value: 42 });

    // Modify original
    {
        let mut guard = container.write().unwrap();
        guard.value = 100;
    }

    // Cloned value should not change
    assert_eq!(cloned, TestStruct { value: 42 });

    // And we can get a new clone with updated value
    let new_clone = container.get_cloned().unwrap();
    assert_eq!(new_clone, TestStruct { value: 100 });
}

#[test]
fn test_wasm_weak_clone() {
    let container = SharedContainer::new(TestStruct { value: 42 });

    // Create a weak reference and clone it
    let weak1 = container.downgrade();
    let weak2 = weak1.clone();

    // Both weak references can be upgraded
    let container1 = weak1.upgrade().unwrap();
    let container2 = weak2.upgrade().unwrap();

    // Modify through container2
    {
        let mut guard = container2.write().unwrap();
        guard.value = 100;
    }

    // Change visible through container1
    {
        let guard = container1.read().unwrap();
        assert_eq!(guard.value, 100);
    }

    // Drop all strong references
    drop(container);
    drop(container1);
    drop(container2);

    // Neither weak reference can be upgraded
    assert!(weak1.upgrade().is_none());
    assert!(weak2.upgrade().is_none());
}

#[test]
fn test_wasm_partial_eq() {
    let container1 = SharedContainer::new(TestStruct { value: 42 });
    let container2 = container1.clone();
    let container3 = SharedContainer::new(TestStruct { value: 100 });

    assert_eq!(container1, container2);
    assert_ne!(container1, container3);
}


#[test]
fn test_wasm_write_conflict() {
    let container = SharedContainer::new(TestStruct { value: 42 });

    // Get a write borrow
    let _guard = container.write().unwrap();

    // Trying to get another write borrow should fail
    assert!(container.write().is_none());

    // Trying to get a read borrow should also fail
    assert!(container.read().is_none());
}
