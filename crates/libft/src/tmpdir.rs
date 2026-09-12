use std::env::temp_dir;
use std::fs::{create_dir, remove_dir_all};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::id;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct TmpDir {
    path: PathBuf,
}

impl TmpDir {
    pub fn new(prefix: &str) -> Self {
        let base = temp_dir();
        loop {
            let count = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = base.join(format!("{}-{}-{}", prefix, id(), count));
            match create_dir(&path) {
                Ok(()) => return Self { path },
                Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("cannot create {}: {e}", path.display()),
            }
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn join(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }

    pub fn join_str(&self, name: &str) -> String {
        self.join(name).to_string_lossy().into_owned()
    }

    pub fn keep(self) -> PathBuf {
        self.path.clone()
    }
}

impl Drop for TmpDir {
    fn drop(&mut self) {
        let _ = remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use std::fs::write;

    use super::*;

    #[test]
    fn creates_a_real_directory() {
        let dir = TmpDir::new("libft-create");
        assert!(dir.path().is_dir());
    }

    #[test]
    fn distinct_dirs_never_collide() {
        let a = TmpDir::new("libft-uniq");
        let b = TmpDir::new("libft-uniq");
        assert_ne!(a.path(), b.path());
    }

    #[test]
    fn name_is_scoped_to_the_process() {
        let dir = TmpDir::new("libft-pid");
        let name = dir.path().file_name().unwrap().to_string_lossy().into_owned();
        assert!(name.contains(&id().to_string()));
    }

    #[test]
    fn join_stays_inside_the_directory() {
        let dir = TmpDir::new("libft-join");
        assert_eq!(dir.join("f").parent(), Some(dir.path()));
        assert_eq!(dir.join_str("f"), dir.join("f").to_string_lossy());
    }

    #[test]
    fn drop_removes_the_directory_and_its_contents() {
        let path = {
            let dir = TmpDir::new("libft-drop");
            write(dir.join("inner"), b"x").unwrap();
            dir.path().to_path_buf()
        };
        assert!(!path.exists());
    }

    #[test]
    fn keep_defuses_removal() {
        let path = TmpDir::new("libft-keep").keep();
        assert!(path.is_dir());
        remove_dir_all(&path).unwrap();
    }
}
