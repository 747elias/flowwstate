use fm_core::{Result, error};
use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, Instant, SystemTime},
};
#[derive(Default)]
pub struct Stability {
    seen: HashMap<String, (u64, SystemTime, Instant)>,
}
impl Stability {
    pub fn ready(&mut self, path: &str, size: u64, modified: SystemTime, wait: u64) -> bool {
        let e = self
            .seen
            .entry(path.into())
            .or_insert((size, modified, Instant::now()));
        if e.0 != size || e.1 != modified {
            *e = (size, modified, Instant::now());
        }
        e.2.elapsed() >= Duration::from_secs(wait)
    }
    pub fn forget(&mut self, path: &str) {
        self.seen.remove(path);
    }
}
pub fn watch(
    path: &Path,
    tx: tokio::sync::mpsc::UnboundedSender<()>,
) -> Result<notify::RecommendedWatcher> {
    use notify::Watcher;
    let mut watcher = notify::recommended_watcher(move |r: notify::Result<notify::Event>| {
        if r.is_ok() {
            let _ = tx.send(());
        }
    })
    .map_err(error)?;
    watcher
        .watch(path, notify::RecursiveMode::Recursive)
        .map_err(error)?;
    Ok(watcher)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn waits_for_stable_metadata() {
        let mut s = Stability::default();
        let m = SystemTime::now();
        assert!(!s.ready("a", 10, m, 30));
        assert!(!s.ready("a", 20, m, 30));
        assert!(s.ready("a", 20, m, 0));
        s.forget("a");
        assert!(!s.ready("a", 20, m, 30));
    }
}
