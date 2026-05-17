use std::sync::{Arc, Mutex};
use slint::{ModelRc, VecModel};

/// A naive implementation of a Sort/Filter Proxy Model mimicking QSortFilterProxyModel
/// It wraps a source vector and generates a filtered/sorted VecModel for Slint.
pub struct CuteProxyModel<T> {
    source_data: Arc<Mutex<Vec<T>>>,
    filter_text: String,
    filter_fn: Option<Box<dyn Fn(&T, &str) -> bool + Send + Sync>>,
    sort_fn: Option<Box<dyn Fn(&T, &T) -> std::cmp::Ordering + Send + Sync>>,
}

impl<T: Clone + 'static> CuteProxyModel<T> {
    pub fn new(source: Arc<Mutex<Vec<T>>>) -> Self {
        Self {
            source_data: source,
            filter_text: String::new(),
            filter_fn: None,
            sort_fn: None,
        }
    }

    pub fn set_filter_fn(&mut self, f: impl Fn(&T, &str) -> bool + Send + Sync + 'static) {
        self.filter_fn = Some(Box::new(f));
    }

    pub fn set_sort_fn(&mut self, f: impl Fn(&T, &T) -> std::cmp::Ordering + Send + Sync + 'static) {
        self.sort_fn = Some(Box::new(f));
    }

    pub fn set_filter_text(&mut self, text: &str) {
        self.filter_text = text.to_lowercase();
    }

    /// Rebuilds the Slint ModelRc based on the current data, filter, and sort.
    /// This should be passed to the Slint UI via `ui.set_table(...)`.
    pub fn build_model<U: Clone + 'static>(&self, mapper: impl Fn(&T) -> U) -> ModelRc<U> {
        let lock = self.source_data.lock().unwrap();
        
        let mut processed: Vec<T> = if let Some(ref f_fn) = self.filter_fn {
            let filter = &self.filter_text;
            if filter.is_empty() {
                lock.clone()
            } else {
                lock.iter()
                    .filter(|item| f_fn(item, filter))
                    .cloned()
                    .collect()
            }
        } else {
            lock.clone()
        };

        if let Some(ref s_fn) = self.sort_fn {
            processed.sort_by(|a, b| s_fn(a, b));
        }

        let mapped_data: Vec<U> = processed.iter().map(mapper).collect();
        ModelRc::new(VecModel::from(mapped_data))
    }
}
