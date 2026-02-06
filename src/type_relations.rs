use lru::LruCache;
use typedlua_parser::ast::types::Type;

/// Type relation cache for subtype checking
///
/// Caches results of subtype checks (source_type, target_type) -> bool to avoid
/// redundant computation during type checking. Uses type memory addresses as keys.
pub struct TypeRelationCache {
    cache: LruCache<(usize, usize), bool>,
    hit_count: u64,
    miss_count: u64,
}

fn type_ptr(ty: &Type) -> usize {
    ty as *const Type as usize
}

impl TypeRelationCache {
    /// Create a new cache with default capacity
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Create a new cache with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(capacity.try_into().unwrap()),
            hit_count: 0,
            miss_count: 0,
        }
    }

    /// Check if a type relation is cached
    pub fn get(&mut self, source: &Type, target: &Type) -> Option<bool> {
        let key = (type_ptr(source), type_ptr(target));
        let result = self.cache.get(&key).copied();

        if result.is_some() {
            self.hit_count += 1;
        } else {
            self.miss_count += 1;
        }

        result
    }

    /// Cache a type relation result
    pub fn insert(&mut self, source: &Type, target: &Type, result: bool) {
        let key = (type_ptr(source), type_ptr(target));
        self.cache.put(key, result);
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get cache hit count
    pub fn hit_count(&self) -> u64 {
        self.hit_count
    }

    /// Get cache miss count
    pub fn miss_count(&self) -> u64 {
        self.miss_count
    }

    /// Get cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }
}

impl Default for TypeRelationCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typedlua_parser::ast::types::{PrimitiveType, Type, TypeKind};

    fn create_test_type(primitive: PrimitiveType) -> Type {
        Type {
            kind: TypeKind::Primitive(primitive),
            span: typedlua_parser::span::Span::dummy(),
        }
    }

    #[test]
    fn test_cache_basic_operations() {
        let mut cache = TypeRelationCache::new();

        let type1 = create_test_type(PrimitiveType::Number);
        let type2 = create_test_type(PrimitiveType::String);
        let type3 = create_test_type(PrimitiveType::Boolean);

        // Cache should be empty initially
        assert_eq!(cache.get(&type1, &type2), None);
        assert_eq!(cache.hit_count(), 0);
        assert_eq!(cache.miss_count(), 1);

        // Insert and retrieve
        cache.insert(&type1, &type2, true);
        assert_eq!(cache.get(&type1, &type2), Some(true));
        assert_eq!(cache.hit_count(), 1);
        assert_eq!(cache.miss_count(), 1);

        // Different types should not collide
        assert_eq!(cache.get(&type1, &type3), None);
        assert_eq!(cache.hit_count(), 1);
        assert_eq!(cache.miss_count(), 2);

        // Cache hit rate should be 1/3
        assert!((cache.hit_rate() - 0.333).abs() < 0.001);
    }

    #[test]
    fn test_cache_symmetric() {
        let mut cache = TypeRelationCache::new();

        let type1 = create_test_type(PrimitiveType::Number);
        let type2 = create_test_type(PrimitiveType::String);

        // (A, B) and (B, A) should be different cache entries
        cache.insert(&type1, &type2, true);
        cache.insert(&type2, &type1, false);

        assert_eq!(cache.get(&type1, &type2), Some(true));
        assert_eq!(cache.get(&type2, &type1), Some(false));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = TypeRelationCache::new();

        let type1 = create_test_type(PrimitiveType::Number);
        let type2 = create_test_type(PrimitiveType::String);

        cache.insert(&type1, &type2, true);
        assert_eq!(cache.get(&type1, &type2), Some(true));

        cache.clear();
        assert_eq!(cache.get(&type1, &type2), None);
        assert_eq!(cache.hit_count(), 1); // Kept from before clear
        assert_eq!(cache.miss_count(), 1); // Only the new miss after clear
    }
}
