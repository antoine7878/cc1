use std::env::current_exe;
use std::path::PathBuf;

pub fn release_bin(name: &str) -> PathBuf {
    let exe = current_exe().expect("cannot locate the running test executable");
    let target = exe
        .ancestors()
        .find(|dir| dir.file_name().is_some_and(|n| n == "target"))
        .unwrap_or_else(|| panic!("no target/ directory above {}", exe.display()));
    target.join("release").join(name)
}

pub fn require_release_bin(name: &str) -> PathBuf {
    let bin = release_bin(name);
    assert!(
        bin.is_file(),
        "{} is missing: run `make ttest` (or `cargo build --release -p ft_lex -p ft_yacc`)",
        bin.display()
    );
    bin
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_into_the_workspace_release_dir() {
        let bin = release_bin("ft_lex");
        assert!(bin.ends_with("target/release/ft_lex"), "got {}", bin.display());
        assert!(bin.is_absolute());
    }

    #[test]
    fn is_independent_of_the_calling_crate() {
        assert_eq!(release_bin("a").parent(), release_bin("b").parent());
    }
}
