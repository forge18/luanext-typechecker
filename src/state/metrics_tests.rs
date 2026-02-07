use crate::state::metrics::{MetricSummary, Metrics};
use std::time::Duration;

#[test]
fn test_metrics_default_creation() {
    let metrics = Metrics::new();
    let summary = metrics.get_summary();
    assert_eq!(summary.symbol_lookups, 0);
    assert_eq!(summary.type_lookups, 0);
    assert_eq!(summary.expressions_checked, 0);
}

#[test]
fn test_metrics_reset() {
    let metrics = Metrics::new();
    metrics.record_symbol_lookup(true);
    metrics.record_type_lookup(true);
    metrics.record_expression_check();
    metrics.record_statement_check();
    metrics.record_function_check();
    metrics.record_type_inference();
    metrics.record_generic_instantiation();
    metrics.record_scope_operation();
    metrics.record_allocation();
    metrics.record_module_resolution();

    metrics.reset();

    let summary = metrics.get_summary();
    assert_eq!(summary.symbol_lookups, 0);
    assert_eq!(summary.type_lookups, 0);
    assert_eq!(summary.expressions_checked, 0);
}

#[test]
fn test_symbol_lookup_tracking() {
    let metrics = Metrics::new();

    metrics.record_symbol_lookup(true);
    metrics.record_symbol_lookup(true);
    metrics.record_symbol_lookup(false);

    let summary = metrics.get_summary();
    assert_eq!(summary.symbol_lookups, 3);
}

#[test]
fn test_symbol_hit_rate_with_no_lookups() {
    let metrics = Metrics::new();
    assert_eq!(metrics.symbol_hit_rate(), 1.0);
}

#[test]
fn test_type_lookup_tracking() {
    let metrics = Metrics::new();

    metrics.record_type_lookup(true);
    metrics.record_type_lookup(false);
    metrics.record_type_lookup(false);

    let summary = metrics.get_summary();
    assert_eq!(summary.type_lookups, 3);
}

#[test]
fn test_type_hit_rate_with_no_lookups() {
    let metrics = Metrics::new();
    assert_eq!(metrics.type_hit_rate(), 1.0);
}

#[test]
fn test_record_expression_check() {
    let metrics = Metrics::new();
    metrics.record_expression_check();
    metrics.record_expression_check();

    let summary = metrics.get_summary();
    assert_eq!(summary.expressions_checked, 2);
}

#[test]
fn test_record_statement_check() {
    let metrics = Metrics::new();
    metrics.record_statement_check();
    metrics.record_statement_check();
    metrics.record_statement_check();

    let summary = metrics.get_summary();
    assert_eq!(summary.statements_checked, 3);
}

#[test]
fn test_record_function_check() {
    let metrics = Metrics::new();
    metrics.record_function_check();

    let summary = metrics.get_summary();
    assert_eq!(summary.functions_checked, 1);
}

#[test]
fn test_record_type_inference() {
    let metrics = Metrics::new();
    for _ in 0..5 {
        metrics.record_type_inference();
    }

    let summary = metrics.get_summary();
    assert_eq!(summary.types_inferred, 5);
}

#[test]
fn test_record_generic_instantiation() {
    let metrics = Metrics::new();
    metrics.record_generic_instantiation();
    metrics.record_generic_instantiation();

    let summary = metrics.get_summary();
    assert_eq!(summary.generic_instantiations, 2);
}

#[test]
fn test_record_scope_operation() {
    let metrics = Metrics::new();
    for _ in 0..10 {
        metrics.record_scope_operation();
    }

    let summary = metrics.get_summary();
    assert_eq!(summary.scope_operations, 10);
}

#[test]
fn test_record_allocation() {
    let metrics = Metrics::new();
    metrics.record_allocation();

    let summary = metrics.get_summary();
    assert_eq!(summary.allocations, 1);
}

#[test]
fn test_record_module_resolution() {
    let metrics = Metrics::new();
    metrics.record_module_resolution();

    let summary = metrics.get_summary();
    assert_eq!(summary.module_resolutions, 1);
}

#[test]
fn test_record_expression_time() {
    let metrics = Metrics::new();
    metrics.record_expression_time("BinaryOp", Duration::from_millis(100));
    metrics.record_expression_time("BinaryOp", Duration::from_millis(200));
    metrics.record_expression_time("FunctionCall", Duration::from_millis(50));
    metrics.record_expression_check();

    let summary = metrics.get_summary();
    assert!(summary.expressions_checked > 0);
}

#[test]
fn test_metric_summary_format() {
    let metrics = Metrics::new();
    metrics.record_symbol_lookup(true);
    metrics.record_type_lookup(true);
    metrics.record_expression_check();
    metrics.record_statement_check();
    metrics.record_function_check();

    let summary = metrics.get_summary();
    let formatted = summary.format();

    assert!(formatted.contains("Performance Metrics"));
    assert!(formatted.contains("Symbol Lookups: 1"));
    assert!(formatted.contains("Type Lookups: 1"));
    assert!(formatted.contains("Expressions Checked: 1"));
    assert!(formatted.contains("Statements Checked: 1"));
    assert!(formatted.contains("Functions Checked: 1"));
}

#[test]
fn test_metric_summary_hit_rates_100_percent() {
    let metrics = Metrics::new();
    metrics.record_symbol_lookup(true);
    metrics.record_symbol_lookup(true);
    metrics.record_type_lookup(true);

    let summary = metrics.get_summary();
    assert!((summary.symbol_hit_rate - 1.0).abs() < 0.001);
    assert!((summary.type_hit_rate - 1.0).abs() < 0.001);
}

#[test]
fn test_metric_summary_hit_rates_0_percent() {
    let metrics = Metrics::new();
    metrics.record_symbol_lookup(false);
    metrics.record_symbol_lookup(false);
    metrics.record_type_lookup(false);

    let summary = metrics.get_summary();
    assert!((summary.symbol_hit_rate - 0.0).abs() < 0.001);
    assert!((summary.type_hit_rate - 0.0).abs() < 0.001);
}

#[test]
fn test_symbol_hit_rate_calculation() {
    let metrics = Metrics::new();
    metrics.record_symbol_lookup(true);
    metrics.record_symbol_lookup(true);
    metrics.record_symbol_lookup(true);
    metrics.record_symbol_lookup(false);

    assert!((metrics.symbol_hit_rate() - 0.75).abs() < 0.001);
}

#[test]
fn test_type_hit_rate_calculation() {
    let metrics = Metrics::new();
    metrics.record_type_lookup(true);
    metrics.record_type_lookup(false);

    assert!((metrics.type_hit_rate() - 0.5).abs() < 0.001);
}

#[test]
fn test_metric_summary_default_values() {
    let summary = MetricSummary {
        symbol_lookups: 0,
        symbol_hit_rate: 1.0,
        type_lookups: 0,
        type_hit_rate: 1.0,
        expressions_checked: 0,
        statements_checked: 0,
        functions_checked: 0,
        types_inferred: 0,
        generic_instantiations: 0,
        module_resolutions: 0,
        scope_operations: 0,
        allocations: 0,
    };

    assert_eq!(summary.symbol_lookups, 0);
    assert_eq!(summary.expressions_checked, 0);
}

#[test]
fn test_multiple_metrics_instances() {
    let metrics1 = Metrics::new();
    let metrics2 = Metrics::new();

    metrics1.record_symbol_lookup(true);
    metrics2.record_symbol_lookup(false);

    let summary1 = metrics1.get_summary();
    let summary2 = metrics2.get_summary();

    assert_eq!(summary1.symbol_lookups, 1);
    assert_eq!(summary2.symbol_lookups, 1);
    assert!(summary1.symbol_hit_rate > summary2.symbol_hit_rate);
}
