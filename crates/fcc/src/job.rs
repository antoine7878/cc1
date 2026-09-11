use std::path::{Path, PathBuf};

use crate::stage::{PIPELINE, Stage, base_name, entry_stage, output_of};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub stage: Stage,
    pub input: PathBuf,
    pub output: PathBuf,
}

pub fn plan(input: &str, last: Stage, requested: Option<&str>, tmp: &Path, id: usize) -> Result<Vec<Step>, String> {
    let entry = entry_stage(input)?;
    let stop = last.min(Stage::Assemble);
    let mut steps = Vec::new();
    let mut current = PathBuf::from(input);

    for stage in PIPELINE {
        if stage < entry || stage > stop {
            continue;
        }
        let output = match stage == stop && last != Stage::Link {
            true => output_of(input, last, requested),
            false => tmp.join(temp_name(input, stage, id)),
        };
        steps.push(Step { stage, input: current, output: output.clone() });
        current = output;
    }
    Ok(steps)
}

pub fn object_of(steps: &[Step]) -> Option<&Path> {
    steps.last().map(|step| step.output.as_path())
}

fn temp_name(input: &str, stage: Stage, id: usize) -> String {
    let stem = Path::new(base_name(input)).file_stem().and_then(|s| s.to_str()).unwrap_or("x");
    format!("{id}-{stem}.{}", stage.extension())
}
