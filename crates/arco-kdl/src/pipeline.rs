use crate::lowering::{LoweredProblem, LoweringError, lower_program};
use crate::normalize::{NormalizeError, normalize_program};
use crate::semantic::{SemanticError, SemanticProgram, validate_program};
use crate::source::{SourceError, parse_program_file};
use miette::Diagnostic;
use std::path::{Path, PathBuf};
use std::{fmt::Display, time::Duration, time::Instant};
use thiserror::Error;

#[derive(Debug)]
pub struct CompiledProgram {
    pub entrypoint: PathBuf,
    pub semantic_program: SemanticProgram,
    pub lowered_problem: LoweredProblem,
    pub timing: PipelineTiming,
}

#[derive(Debug)]
pub struct ValidatedProgram {
    pub entrypoint: PathBuf,
    pub semantic_program: SemanticProgram,
}

#[derive(Debug)]
struct ValidatedSource {
    entrypoint: PathBuf,
    parsed_source: crate::source::ParsedSource,
    semantic_program: SemanticProgram,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PipelineTiming {
    pub parse: Duration,
    pub validate: Duration,
    pub lower: Duration,
}

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error(transparent)]
    Source(#[from] SourceError),
    #[error(transparent)]
    Normalize(#[from] NormalizeError),
    #[error(transparent)]
    Semantic(#[from] SemanticError),
    #[error(transparent)]
    Lowering(#[from] LoweringError),
}

impl Diagnostic for PipelineError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        None
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        match self {
            Self::Source(error) => Some(error),
            Self::Normalize(error) => Some(error),
            Self::Semantic(error) => Some(error),
            Self::Lowering(error) => Some(error),
        }
    }
}

pub fn compile_file(path: &Path) -> Result<CompiledProgram, PipelineError> {
    let parse_start = Instant::now();
    let parsed_source = parse_program_file(path)?;
    let parse = parse_start.elapsed();

    let validate_start = Instant::now();
    let semantic_program = validate_parsed_source(&parsed_source, path)?;
    let validate = validate_start.elapsed();

    let validated = ValidatedSource {
        entrypoint: path.to_path_buf(),
        parsed_source,
        semantic_program,
    };

    let lower_start = Instant::now();
    let lowered_problem = lower_program(
        &validated.semantic_program,
        &validated.parsed_source.program,
        &validated.entrypoint,
    )?;
    let lower = lower_start.elapsed();

    Ok(CompiledProgram {
        entrypoint: validated.entrypoint,
        semantic_program: validated.semantic_program,
        lowered_problem,
        timing: PipelineTiming {
            parse,
            validate,
            lower,
        },
    })
}

pub fn validate_file(path: &Path) -> Result<ValidatedProgram, PipelineError> {
    let validated = validate_source_file(path)?;

    Ok(ValidatedProgram {
        entrypoint: validated.entrypoint,
        semantic_program: validated.semantic_program,
    })
}

fn validate_source_file(path: &Path) -> Result<ValidatedSource, PipelineError> {
    let parsed_source = parse_program_file(path)?;
    let semantic_program = validate_parsed_source(&parsed_source, path)?;

    Ok(ValidatedSource {
        entrypoint: path.to_path_buf(),
        parsed_source,
        semantic_program,
    })
}

fn validate_parsed_source(
    parsed_source: &crate::source::ParsedSource,
    path: &Path,
) -> Result<SemanticProgram, PipelineError> {
    normalize_program(&parsed_source.program, path)?;
    validate_program(&parsed_source.program, path).map_err(Into::into)
}
