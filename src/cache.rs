use std::thread;
use std::time::{Duration, SystemTime};
use std::sync::{Mutex, Arc};
use std::collections::HashMap;

pub struct Cache {
    pub cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    pub interval: Duration,
}

#[derive(Clone)]
pub struct CacheEntry {
    pub created_at: SystemTime,
    pub data: String,
}

fn reap_loop(cache: Arc<Mutex<HashMap<String, CacheEntry>>>, interval: Duration) {
    let mut guard = cache.lock().unwrap();
    let keys_to_remove: Vec<String> = guard.iter()
            .filter(|(_, value)| value.created_at.elapsed().unwrap() > interval)
            .map(|(key, _)| key.clone())
            .collect();
    for key in keys_to_remove {
        println!("Found an old cache entry: {}", key);
        guard.remove(&key);
        println!("Current cache entries: {}", guard.len());
    }
}

impl Cache {
    pub fn new(interval: Duration) -> Cache {
        let this = Cache {
            cache: Arc::new(Mutex::new(HashMap::new())),
            interval,
        };
        let cache = this.cache.clone();
        thread::spawn(move || { 
            loop {
                thread::sleep(interval/2);
                reap_loop(cache.clone(), interval);
            }
        });
        this
    }

    pub fn get_cache(&self, key: &str) -> Option<CacheEntry> {
        let guard = self.cache.lock().unwrap();
        guard.get(key).cloned()
    }

    pub fn add_cache(&mut self, key: &str, value: &str) {
        let mut guard = self.cache.lock().unwrap();
        guard.entry(key.to_string())
            .or_insert_with( || CacheEntry {
                created_at: SystemTime::now(),
                data: value.to_string(),
            });
    }
}