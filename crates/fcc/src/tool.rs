use std::env::{current_exe, var};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Clang,
    Cc1,
    Llc,
    As,
}

impl Tool {
    pub fn name(self) -> &'static str {
        match self {
            Tool::Clang => "clang",
            Tool::Cc1 => "cc1",
            Tool::Llc => "llc",
            Tool::As => "as",
        }
    }

    pub fn env_var(self) -> &'static str {
        match self {
            Tool::Clang => "FCC_CLANG",
            Tool::Cc1 => "FCC_CC1",
            Tool::Llc => "FCC_LLC",
            Tool::As => "FCC_AS",
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
    fn every_tool_has_a_distinct_name_and_override() {
        let tools = [Tool::Clang, Tool::Cc1, Tool::Llc, Tool::As];
        let names: Vec<_> = tools.iter().map(|t| t.name()).collect();
        let vars: Vec<_> = tools.iter().map(|t| t.env_var()).collect();
        for i in 0..tools.len() {
            for j in i + 1..tools.len() {
                assert_ne!(names[i], names[j]);
                assert_ne!(vars[i], vars[j]);
            }
        }
    }

    #[test]
    fn a_system_tool_falls_back_to_path_lookup() {
        assert_eq!(
            resolve_in("clang", None, Some(PathBuf::from("/nowhere/fcc"))),
            PathBuf::from("clang")
        );
    }
}
