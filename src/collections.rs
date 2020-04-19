use std::collections::{ HashMap };
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub type GetKeyType<T> = fn(&T) -> String;

pub struct CustomHashSet<T> {
    get_key: GetKeyType<T>,
    _value_map: HashMap<String, T>,
}

impl<T> CustomHashSet<T> where T:Hash + 'static {
    fn default_get_key(value:&T) -> String where T: Hash {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn new(get_key_option:Option<GetKeyType<T>>) -> CustomHashSet<T> {
        let _value_map:HashMap<String, T> = HashMap::new();
        let get_key;

        if let Some(passed_get_key) = get_key_option {
            get_key = passed_get_key;
        } else {
            get_key = CustomHashSet::default_get_key;
        }

        CustomHashSet {
            get_key,
            _value_map,
        }
    }

    pub fn insert(&mut self, value:T) {
        let key = (self.get_key)(&value);

        self._value_map.insert(key, value);
    }

    pub fn contains(&self, value: &T) -> bool {
        let key = &(self.get_key)(value);

        self._value_map.contains_key(key)
    }
}