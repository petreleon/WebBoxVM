#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
std::thread_local! {
    static FAIL_AFTER: Cell<usize> = const { Cell::new(usize::MAX) };
}

pub(super) fn should_fail(_core: usize) -> bool {
    #[cfg(test)]
    {
        return FAIL_AFTER.with(|limit| _core >= limit.get());
    }
    #[cfg(not(test))]
    false
}

#[cfg(test)]
pub(in crate::runtime) fn with_failure_after<T>(limit: usize, run: impl FnOnce() -> T) -> T {
    FAIL_AFTER.with(|slot| {
        let previous = slot.replace(limit);
        let _reset = Reset(slot, previous);
        run()
    })
}

#[cfg(test)]
struct Reset<'a>(&'a Cell<usize>, usize);

#[cfg(test)]
impl Drop for Reset<'_> {
    fn drop(&mut self) {
        self.0.set(self.1);
    }
}
