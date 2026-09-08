use std::env::{current_exe, var};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Clang,
    Cc1,
}

impl Tool {
    pub fn name(self) -> &'static str {
        match self {
            Tool::Clang => "clang",
            Tool::Cc1 => "cc1",
        }
    }

    pub fn env_var(self) -> &'static str {
        match self {
            Tool::Clang => "FCC_CPP",
            Tool::Cc1 => "FCC_CC1",
        }
    }

    pub fn resolve(self) -> PathBuf {
        resolve_in(self.name(), var(self.env_var()).ok(), current_exe().ok())
    }
}

pub fn resolve_in(name: &str, override_path: Option<String>, exe: Option<PathBuf>) -> PathBuf {
    if let Some(path) = override_path {
        return PathBuf::from(path);
    }
    let sibling = exe.as_ref().and_then(|e| e.parent()).map(|dir| dir.join(name));
    match sibling {
        Some(path) if path.is_file() => path,
        _ => PathBuf::from(name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_wins() {
        let got = resolve_in(
            "clang",
            Some("/opt/clang".to_string()),
            Some(PathBuf::from("/usr/bin/fcc")),
        );
        assert_eq!(got, PathBuf::from("/opt/clang"));
    }

    #[test]
    fn falls_back_to_bare_name_when_no_sibling() {
        let got = resolve_in("cc1", None, Some(PathBuf::from("/nonexistent/dir/fcc")));
        assert_eq!(got, PathBuf::from("cc1"));
    }

    #[test]
    fn falls_back_to_bare_name_without_exe() {
        assert_eq!(resolve_in("cc1", None, None), PathBuf::from("cc1"));
    }

    #[test]
    fn finds_sibling_binary() {
        let exe = current_exe().unwrap();
        let name = exe.file_name().unwrap().to_str().unwrap().to_string();
        let got = resolve_in(&name, None, Some(exe.clone()));
        assert_eq!(got, exe);
    }

    #[test]
    fn names_and_env_vars_are_distinct() {
        assert_eq!(Tool::Clang.name(), "clang");
        assert_eq!(Tool::Cc1.name(), "cc1");
        assert_ne!(Tool::Clang.env_var(), Tool::Cc1.env_var());
    }

    #[test]
    fn a_system_tool_falls_back_to_path_lookup() {
        assert_eq!(
            resolve_in("clang", None, Some(PathBuf::from("/nowhere/fcc"))),
            PathBuf::from("clang")
        );
    }
}
