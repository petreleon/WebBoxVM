use std::env;

pub(crate) fn trace_filter_allows(var: &str, text: &str) -> bool {
    let Ok(filters) = env::var(var) else {
        return true;
    };
    let mut saw_filter = false;
    for filter in filters.split(',').map(str::trim) {
        if filter.is_empty() {
            continue;
        }
        saw_filter = true;
        if text.contains(filter) {
            return true;
        }
    }
    !saw_filter
}
