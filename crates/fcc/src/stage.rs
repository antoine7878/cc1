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
