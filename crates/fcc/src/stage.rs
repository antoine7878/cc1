use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Preprocess,
    Compile,
    Lower,
    Assemble,
    Link,
}

pub const PIPELINE: [Stage; 4] = [Stage::Preprocess, Stage::Compile, Stage::Lower, Stage::Assemble];

impl Stage {
    pub fn extension(self) -> &'static str {
        match self {
            Stage::Preprocess => "i",
            Stage::Compile => "ll",
            Stage::Lower => "s",
            Stage::Assemble => "o",
            Stage::Link => "out",
        }
    }
}

pub fn entry_stage(input: &str) -> Result<Stage, String> {
    match Path::new(input).extension().and_then(|e| e.to_str()) {
        Some("c") => Ok(Stage::Preprocess),
        Some("i") => Ok(Stage::Compile),
        Some("ll") => Ok(Stage::Lower),
        Some("s") => Ok(Stage::Assemble),
        Some("o" | "a" | "so") => Ok(Stage::Link),
        _ => Err(format!("{input}: unrecognized file suffix")),
    }
}

pub fn output_of(input: &str, last: Stage, requested: Option<&str>) -> PathBuf {
    if let Some(path) = requested {
        return PathBuf::from(path);
    }
    match last {
        Stage::Preprocess => PathBuf::from("-"),
        Stage::Link => PathBuf::from("a.out"),
        stage => PathBuf::from(base_name(input)).with_extension(stage.extension()),
    }
}

pub fn base_name(input: &str) -> &str {
    Path::new(input).file_name().and_then(|n| n.to_str()).unwrap_or(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_output_wins() {
        assert_eq!(output_of("a.c", Stage::Compile, Some("out.ll")), PathBuf::from("out.ll"));
        assert_eq!(output_of("a.c", Stage::Preprocess, Some("a.i")), PathBuf::from("a.i"));
    }

    #[test]
    fn derives_extension_per_stage() {
        assert_eq!(output_of("a.c", Stage::Compile, None), PathBuf::from("a.ll"));
        assert_eq!(output_of("a.c", Stage::Lower, None), PathBuf::from("a.s"));
        assert_eq!(output_of("a.c", Stage::Assemble, None), PathBuf::from("a.o"));
    }

    #[test]
    fn preprocess_defaults_to_stdout() {
        assert_eq!(output_of("a.c", Stage::Preprocess, None), PathBuf::from("-"));
    }

    #[test]
    fn link_defaults_to_a_out() {
        assert_eq!(output_of("a.c", Stage::Link, None), PathBuf::from("a.out"));
    }

    #[test]
    fn object_is_created_in_the_current_directory() {
        assert_eq!(output_of("src/a.c", Stage::Assemble, None), PathBuf::from("a.o"));
    }

    #[test]
    fn stages_are_ordered() {
        assert!(Stage::Preprocess < Stage::Compile);
        assert!(Stage::Compile < Stage::Lower);
        assert!(Stage::Lower < Stage::Assemble);
        assert!(Stage::Assemble < Stage::Link);
    }

    #[test]
    fn suffix_selects_the_entry_stage() {
        assert_eq!(entry_stage("a.c"), Ok(Stage::Preprocess));
        assert_eq!(entry_stage("a.i"), Ok(Stage::Compile));
        assert_eq!(entry_stage("a.ll"), Ok(Stage::Lower));
        assert_eq!(entry_stage("a.s"), Ok(Stage::Assemble));
        assert_eq!(entry_stage("a.o"), Ok(Stage::Link));
        assert_eq!(entry_stage("libc.a"), Ok(Stage::Link));
        assert_eq!(entry_stage("libc.so"), Ok(Stage::Link));
    }

    #[test]
    fn unknown_suffix_is_rejected() {
        assert!(entry_stage("a.txt").is_err());
        assert!(entry_stage("a").is_err());
    }

    #[test]
    fn base_name_strips_directories() {
        assert_eq!(base_name("src/a.c"), "a.c");
        assert_eq!(base_name("a.c"), "a.c");
    }
}
