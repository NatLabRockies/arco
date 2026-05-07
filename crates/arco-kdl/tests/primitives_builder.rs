mod common;

use arco_kdl::source::parse_program_text;
use arco_kdl::{
    PrimitiveBuildError, build_arco_document, build_indexed_data, build_model, build_model_document,
};
use common::fixture_text;
use std::path::PathBuf;

fn parse_fixture(name: &str) -> Result<arco_kdl::source::ParsedSource, Box<dyn std::error::Error>> {
    let path = PathBuf::from("test.kdl");
    let text = fixture_text(name)?;
    Ok(parse_program_text(&text, &path)?)
}

#[test]
fn builds_primitive_model_from_kdl_source() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = parse_fixture("primitives_builds_simple_model_and_docs.kdl")?;
    let model = build_model(&parsed)?;

    assert_eq!(model.num_variables(), 2);
    assert_eq!(model.num_constraints(), 1);

    let objective = model.objective();
    assert_eq!(objective.sense, Some(arco_model::Sense::Minimize));
    assert_eq!(objective.terms.len(), 2);

    Ok(())
}

#[test]
fn builds_model_and_arco_documents_from_kdl_source() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = parse_fixture("primitives_builds_simple_model_and_docs.kdl")?;
    let model_document = build_model_document(&parsed)?;
    let arco_document = build_arco_document(&parsed)?;

    assert!(model_document.fingerprint.is_some());
    assert!(arco_document.model.is_some());
    assert!(arco_document.indexed_data.is_none());

    Ok(())
}

#[test]
fn builds_indexed_data_from_sets_and_scalar_params() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = parse_fixture("primitives_builds_indexed_data_from_sets_and_params.kdl")?;
    let indexed_data = build_indexed_data(&parsed)?;

    assert_eq!(indexed_data.sets.len(), 2);
    assert!(indexed_data.sets.contains_key("assets"));
    assert!(indexed_data.parameters.contains_key("voll"));

    Ok(())
}

#[test]
fn rejects_nonlinear_constraints_for_primitive_model_build()
-> Result<(), Box<dyn std::error::Error>> {
    let parsed = parse_fixture("primitives_rejects_nonlinear_expression.kdl")?;
    let error = build_model(&parsed).expect_err("nonlinear expression should fail");

    assert!(matches!(
        error,
        PrimitiveBuildError::UnsupportedExpression { .. }
    ));

    Ok(())
}
