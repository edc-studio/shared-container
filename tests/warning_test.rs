#[cfg(feature = "tokio-sync")]
mod tests {
    use shared_container::SharedContainer;

    #[test]
    fn test_warnings() {
        let container = SharedContainer::new(42);
        
        // These should emit warnings when compiled with tokio-sync feature
        let _ = container.read();
        let _ = container.write();
        let _ = container.get_cloned();
        
        // These should not emit warnings
        let _ = container.read_async();
        let _ = container.write_async();
        let _ = container.get_cloned_async();
    }
}