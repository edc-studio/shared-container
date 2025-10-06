/// Tests for the new 0.3 API with type-level separation of sync and async
#[cfg(test)]
mod shared_sync_tests {
    use shared_container::{Shared, SyncAccess};

    #[derive(Debug, Clone, PartialEq)]
    struct TestData {
        value: i32,
    }

    #[test]
    fn test_shared_new_and_read() {
        let container = Shared::new(TestData { value: 42 });
        let guard = container.read().unwrap();
        assert_eq!(guard.value, 42);
    }

    #[test]
    fn test_shared_write() {
        let container = Shared::new(TestData { value: 42 });

        {
            let mut guard = container.write().unwrap();
            guard.value = 100;
        }

        let guard = container.read().unwrap();
        assert_eq!(guard.value, 100);
    }

    #[test]
    fn test_shared_clone() {
        let container1 = Shared::new(TestData { value: 42 });
        let container2 = container1.clone();

        {
            let mut guard = container2.write().unwrap();
            guard.value = 100;
        }

        let guard = container1.read().unwrap();
        assert_eq!(guard.value, 100);
    }

    #[test]
    fn test_shared_get_cloned() {
        let container = Shared::new(TestData { value: 42 });
        let cloned = container.get_cloned().unwrap();
        assert_eq!(cloned, TestData { value: 42 });

        {
            let mut guard = container.write().unwrap();
            guard.value = 100;
        }

        // Cloned value should remain unchanged
        assert_eq!(cloned, TestData { value: 42 });

        let new_clone = container.get_cloned().unwrap();
        assert_eq!(new_clone, TestData { value: 100 });
    }

    #[test]
    fn test_shared_weak() {
        let container = Shared::new(TestData { value: 42 });
        let weak = container.downgrade();

        let upgraded = weak.upgrade().unwrap();
        {
            let mut guard = upgraded.write().unwrap();
            guard.value = 100;
        }

        {
            let guard = container.read().unwrap();
            assert_eq!(guard.value, 100);
        }

        drop(container);
        drop(upgraded);

        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn test_shared_weak_clone() {
        let container = Shared::new(TestData { value: 42 });
        let weak1 = container.downgrade();
        let weak2 = weak1.clone();

        let upgraded1 = weak1.upgrade().unwrap();
        let upgraded2 = weak2.upgrade().unwrap();

        {
            let mut guard = upgraded2.write().unwrap();
            guard.value = 100;
        }

        {
            let guard = upgraded1.read().unwrap();
            assert_eq!(guard.value, 100);
        }

        drop(container);
        drop(upgraded1);
        drop(upgraded2);

        assert!(weak1.upgrade().is_none());
        assert!(weak2.upgrade().is_none());
    }

    #[test]
    fn test_access_error_types() {
        use shared_container::AccessError;

        let err = AccessError::UnsupportedMode;
        assert_eq!(err.to_string(), "operation not supported for this container mode");

        let err = AccessError::BorrowConflict;
        assert_eq!(err.to_string(), "borrow conflict: lock already held");

        let err = AccessError::Poisoned;
        assert_eq!(err.to_string(), "lock poisoned by panic");
    }
}

#[cfg(feature = "async")]
#[cfg(test)]
mod async_shared_tests {
    use shared_container::{AsyncShared, AsyncAccess};
    use tokio::runtime::Runtime;

    #[derive(Debug, Clone, PartialEq)]
    struct TestData {
        value: i32,
    }

    #[test]
    fn test_async_shared_new_and_read() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let container = AsyncShared::new(TestData { value: 42 });
            let guard = container.read_async().await;
            assert_eq!(guard.value, 42);
        });
    }

    #[test]
    fn test_async_shared_write() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let container = AsyncShared::new(TestData { value: 42 });

            {
                let mut guard = container.write_async().await;
                guard.value = 100;
            }

            let guard = container.read_async().await;
            assert_eq!(guard.value, 100);
        });
    }

    #[test]
    fn test_async_shared_clone() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let container1 = AsyncShared::new(TestData { value: 42 });
            let container2 = container1.clone();

            {
                let mut guard = container2.write_async().await;
                guard.value = 100;
            }

            let guard = container1.read_async().await;
            assert_eq!(guard.value, 100);
        });
    }

    #[test]
    fn test_async_shared_get_cloned() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let container = AsyncShared::new(TestData { value: 42 });
            let cloned = container.get_cloned_async().await;
            assert_eq!(cloned, TestData { value: 42 });

            {
                let mut guard = container.write_async().await;
                guard.value = 100;
            }

            // Cloned value should remain unchanged
            assert_eq!(cloned, TestData { value: 42 });

            let new_clone = container.get_cloned_async().await;
            assert_eq!(new_clone, TestData { value: 100 });
        });
    }

    #[test]
    fn test_async_shared_weak() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let container = AsyncShared::new(TestData { value: 42 });
            let weak = container.downgrade();

            let upgraded = weak.upgrade().unwrap();
            {
                let mut guard = upgraded.write_async().await;
                guard.value = 100;
            }

            {
                let guard = container.read_async().await;
                assert_eq!(guard.value, 100);
            }

            drop(container);
            drop(upgraded);

            assert!(weak.upgrade().is_none());
        });
    }
}

#[cfg(test)]
mod shared_any_tests {
    use shared_container::{Shared, SharedAny, SyncAccess};

    #[derive(Debug, Clone, PartialEq)]
    struct TestData {
        value: i32,
    }

    #[test]
    fn test_shared_any_from_sync() {
        let shared = Shared::new(TestData { value: 42 });
        let any: SharedAny<TestData> = shared.into();

        let guard = any.read().unwrap();
        assert_eq!(guard.value, 42);
    }

    #[test]
    fn test_shared_any_clone() {
        let shared = Shared::new(TestData { value: 42 });
        let any1: SharedAny<TestData> = shared.into();
        let any2 = any1.clone();

        {
            let mut guard = any2.write().unwrap();
            guard.value = 100;
        }

        let guard = any1.read().unwrap();
        assert_eq!(guard.value, 100);
    }

    #[test]
    fn test_shared_any_downgrade_upgrade() {
        let shared = Shared::new(TestData { value: 42 });
        let any: SharedAny<TestData> = shared.into();
        let weak = any.downgrade();

        let upgraded = weak.upgrade().unwrap();
        {
            let mut guard = upgraded.write().unwrap();
            guard.value = 100;
        }

        {
            let guard = any.read().unwrap();
            assert_eq!(guard.value, 100);
        }

        drop(any);
        drop(upgraded);

        assert!(weak.upgrade().is_none());
    }

    #[cfg(feature = "async")]
    #[test]
    fn test_shared_any_unsupported_mode_error() {
        use shared_container::{AsyncShared, AccessError};
        use tokio::runtime::Runtime;

        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let async_shared = AsyncShared::new(TestData { value: 42 });
            let any: SharedAny<TestData> = async_shared.into();

            // Trying to use sync methods on async container should return UnsupportedMode error
            let result = any.read();
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), AccessError::UnsupportedMode);

            let result = any.write();
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), AccessError::UnsupportedMode);

            let result = any.get_cloned();
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), AccessError::UnsupportedMode);
        });
    }
}
