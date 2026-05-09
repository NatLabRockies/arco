use arco_ops::{
    OpsAlgebraicProblem, OpsConstraintSense, OpsExportFormat, OpsLinearConstraint,
    OpsLinearObjective, OpsLinearTerm, OpsObjectiveSense, OpsVariableInstance, OpsVariableKind,
    portable_problem_from_ops,
};

#[test]
fn portable_mapping_preserves_ops_problem_shape() {
    let problem = OpsAlgebraicProblem {
        variable_instances: vec![OpsVariableInstance {
            name: "x".to_string(),
            family: "x".to_string(),
            lower: 0.0,
            upper: Some(10.0),
            kind: OpsVariableKind::Continuous,
        }],
        constraints: vec![OpsLinearConstraint {
            name: "c1".to_string(),
            sense: OpsConstraintSense::LessEqual,
            rhs: 5.0,
            terms: vec![OpsLinearTerm {
                variable_name: "x".to_string(),
                coefficient: 1.0,
            }],
        }],
        objective: OpsLinearObjective {
            name: "obj".to_string(),
            sense: OpsObjectiveSense::Minimize,
            constant: 0.0,
            terms: vec![OpsLinearTerm {
                variable_name: "x".to_string(),
                coefficient: 1.0,
            }],
        },
        reports: vec![],
    };

    let portable = portable_problem_from_ops(&problem);
    assert_eq!(portable.variable_instances.len(), 1);
    assert_eq!(portable.constraints.len(), 1);
    assert_eq!(portable.objective.terms.len(), 1);
}

#[test]
fn export_problem_accepts_only_ops_dto_boundary() {
    let problem = OpsAlgebraicProblem {
        variable_instances: vec![OpsVariableInstance {
            name: "x".to_string(),
            family: "x".to_string(),
            lower: 0.0,
            upper: None,
            kind: OpsVariableKind::Continuous,
        }],
        constraints: vec![],
        objective: OpsLinearObjective {
            name: "obj".to_string(),
            sense: OpsObjectiveSense::Minimize,
            constant: 1.0,
            terms: vec![],
        },
        reports: vec![],
    };

    let bytes = arco_ops::ArcoOps::export_problem(&problem, OpsExportFormat::Lp)
        .expect("ops DTO export should succeed");
    assert!(!bytes.is_empty());
}
