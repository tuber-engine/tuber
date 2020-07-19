use std::collections::HashMap;

pub struct ResourceManager<T> {
    resources: HashMap<String, T>,
}

impl<T> ResourceManager<T> {
    pub fn new() -> ResourceManager<T> {
        ResourceManager {
            resources: HashMap::new(),
        }
    }

    pub fn store(&mut self, name: &str, data: T) {
        self.resources.insert(name.into(), data);
    }

    pub fn contains_resource(&self, name: &str) -> bool {
        self.resources.contains_key(name)
    }

    pub fn fetch(&self, name: &str) -> Option<&T> {
        self.resources.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_store_resource() {
        let mut resource_manager = ResourceManager::new();
        resource_manager.store("some_resource", 5u64);
        assert!(resource_manager.contains_resource("some_resource"));
    }

    #[test]
    fn should_retrieve_resource() {
        let mut resource_manager = ResourceManager::new();
        resource_manager.store("some_resource", 5u64);
        assert_eq!(resource_manager.fetch("some_resource"), Some(&5u64));
    }

    #[test]
    fn should_update_resource_if_stored_again() {
        let mut resource_manager = ResourceManager::new();
        resource_manager.store("some_resource", 5u64);
        assert_eq!(resource_manager.fetch("some_resource"), Some(&5u64));
        resource_manager.store("some_resource", 12u64);
        assert_eq!(resource_manager.fetch("some_resource"), Some(&12u64));
    }
}
