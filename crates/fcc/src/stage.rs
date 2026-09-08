use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Preprocess,
    Compile,
    Assemble,
    Link,
}

impl Stage {
    pub fn extension(self) -> &'static str {
        match self {
            Stage::Preprocess => "i",
            Stage::Compile => "s",
            Stage::Assemble => "o",
            Stage::Link => "out",
        }
    }
}

pub fn output_of(input: &str, last: Stage, requested: Option<&str>) -> PathBuf {
    if let Some(path) = requested {
        return PathBuf::from(path);
    }
    match last {
        Stage::Link => PathBuf::from("a.out"),
        stage => Path::new(input).with_extension(stage.extension()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_output_wins() {
        assert_eq!(output_of("a.c", Stage::Compile, Some("out.s")), PathBuf::from("out.s"));
    }

    #[test]
    fn derives_extension_per_stage() {
        assert_eq!(output_of("a.c", Stage::Preprocess, None), PathBuf::from("a.i"));
        assert_eq!(output_of("a.c", Stage::Compile, None), PathBuf::from("a.s"));
        assert_eq!(output_of("a.c", Stage::Assemble, None), PathBuf::from("a.o"));
    }

    #[test]
    fn link_defaults_to_a_out() {
        assert_eq!(output_of("a.c", Stage::Link, None), PathBuf::from("a.out"));
    }

    #[test]
    fn stages_are_ordered() {
        assert!(Stage::Preprocess < Stage::Compile);
        assert!(Stage::Compile < Stage::Assemble);
        assert!(Stage::Assemble < Stage::Link);
    }
}
