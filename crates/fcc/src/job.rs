use std::path::{Path, PathBuf};

use crate::stage::{PIPELINE, Stage, base_name, entry_stage, output_of};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub stage: Stage,
    pub input: PathBuf,
    pub output: PathBuf,
}

pub fn plan(
    input: &str,
    last: Stage,
    requested: Option<&str>,
    tmp: &Path,
    id: usize,
) -> Result<Vec<Step>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> &'static Path {
        Path::new("/tmp/fcc")
    }

    fn stages(steps: &[Step]) -> Vec<Stage> {
        steps.iter().map(|step| step.stage).collect()
    }

    #[test]
    fn source_to_executable_runs_every_stage() {
        let steps = plan("a.c", Stage::Link, None, tmp(), 0).unwrap();
        assert_eq!(stages(&steps), PIPELINE);
        assert_eq!(steps[0].input, PathBuf::from("a.c"));
        assert_eq!(steps[3].output, tmp().join("0-a.o"));
    }

    #[test]
    fn each_step_consumes_the_previous_output() {
        let steps = plan("a.c", Stage::Link, None, tmp(), 0).unwrap();
        for pair in steps.windows(2) {
            assert_eq!(pair[0].output, pair[1].input);
        }
    }

    #[test]
    fn compile_only_stops_at_the_object() {
        let steps = plan("src/a.c", Stage::Assemble, None, tmp(), 0).unwrap();
        assert_eq!(stages(&steps), PIPELINE);
        assert_eq!(steps[3].output, PathBuf::from("a.o"));
    }

    #[test]
    fn preprocess_only_runs_one_step_to_stdout() {
        let steps = plan("a.c", Stage::Preprocess, None, tmp(), 0).unwrap();
        assert_eq!(stages(&steps), [Stage::Preprocess]);
        assert_eq!(steps[0].output, PathBuf::from("-"));
    }

    #[test]
    fn requested_output_names_the_final_step() {
        let steps = plan("a.c", Stage::Lower, Some("out.s"), tmp(), 0).unwrap();
        assert_eq!(steps.last().unwrap().output, PathBuf::from("out.s"));
        assert_eq!(steps[0].output, tmp().join("0-a.i"));
    }

    #[test]
    fn preprocessed_source_skips_the_preprocessor() {
        let steps = plan("a.i", Stage::Link, None, tmp(), 0).unwrap();
        assert_eq!(stages(&steps), [Stage::Compile, Stage::Lower, Stage::Assemble]);
        assert_eq!(steps[0].input, PathBuf::from("a.i"));
    }

    #[test]
    fn assembly_only_gets_assembled() {
        let steps = plan("a.s", Stage::Link, None, tmp(), 0).unwrap();
        assert_eq!(stages(&steps), [Stage::Assemble]);
    }

    #[test]
    fn nothing_to_do_when_the_entry_is_past_the_last_stage() {
        assert!(plan("a.s", Stage::Preprocess, None, tmp(), 0).unwrap().is_empty());
        assert_eq!(object_of(&[]), None);
    }

    #[test]
    fn temporaries_are_named_after_the_operand_index() {
        let first = plan("one/a.c", Stage::Link, None, tmp(), 0).unwrap();
        let second = plan("two/a.c", Stage::Link, None, tmp(), 1).unwrap();
        assert_eq!(first[0].output, tmp().join("0-a.i"));
        assert_eq!(second[0].output, tmp().join("1-a.i"));
    }

    #[test]
    fn object_of_reports_the_last_output() {
        let steps = plan("a.c", Stage::Link, None, tmp(), 3).unwrap();
        assert_eq!(object_of(&steps), Some(tmp().join("3-a.o").as_path()));
    }

    #[test]
    fn unknown_suffix_has_no_plan() {
        assert!(plan("a.txt", Stage::Link, None, tmp(), 0).is_err());
    }
}
