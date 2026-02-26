use std::collections::HashMap;

use crate::{ast::ArgSpec, ArgType, ArgValue, Diagnostic, ErrorCode, EvalContext, Level, Span};

pub fn parse_args(
    tokens: &[&str],
    span_start: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> HashMap<String, ArgValue> {
    let mut args = HashMap::new();

    for token in tokens {
        let Some((key, raw_value)) = token.split_once('=') else {
            diagnostics.push(Diagnostic {
                level: Level::Error,
                code: ErrorCode::InvalidType,
                message: format!("invalid argument '{}', expected key=value", token),
                span: Span {
                    start: span_start,
                    end: span_start + token.len(),
                },
                suggestion: None,
            });
            continue;
        };

        let value = if let Ok(i) = raw_value.parse::<i64>() {
            ArgValue::Int(i)
        } else {
            ArgValue::String(raw_value.to_string())
        };
        args.insert(key.to_string(), value);
    }

    args
}

pub fn validate_args(
    actual: &HashMap<String, ArgValue>,
    spec: &[(String, ArgSpec)],
    span: Span,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (name, arg_spec) in spec {
        if arg_spec.required && !actual.contains_key(name) {
            diagnostics.push(Diagnostic {
                level: Level::Error,
                code: ErrorCode::MissingRequiredArg,
                message: format!("missing required arg '{}'", name),
                span: span.clone(),
                suggestion: None,
            });
        }
    }

    for (name, value) in actual {
        let Some(arg_spec) = spec.iter().find(|(n, _)| n == name).map(|(_, s)| s) else {
            diagnostics.push(Diagnostic {
                level: Level::Error,
                code: ErrorCode::UnknownArg,
                message: format!("unknown arg '{}'", name),
                span: span.clone(),
                suggestion: None,
            });
            continue;
        };

        match (&arg_spec.arg_type, value) {
            (ArgType::Int, ArgValue::Int(_)) => {}
            (ArgType::String, ArgValue::String(_)) => {}
            (ArgType::StaticEnum(allowed), ArgValue::String(s)) => {
                if !allowed.iter().any(|v| v == s) {
                    diagnostics.push(Diagnostic {
                        level: Level::Error,
                        code: ErrorCode::InvalidStaticEnumValue,
                        message: format!("'{}' is not in static enum", s),
                        span: span.clone(),
                        suggestion: Some(format!("allowed: {}", allowed.join(", "))),
                    });
                }
            }
            (ArgType::DynamicEnum(_), ArgValue::String(_)) => {}
            _ => diagnostics.push(Diagnostic {
                level: Level::Error,
                code: ErrorCode::InvalidType,
                message: format!("invalid type for arg '{}'", name),
                span: span.clone(),
                suggestion: None,
            }),
        }
    }
}

pub fn eval_dynamic_args(
    actual: &HashMap<String, ArgValue>,
    spec: &[(String, ArgSpec)],
    span: Span,
    ctx: &EvalContext,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (name, arg_spec) in spec {
        if let ArgType::DynamicEnum(dynamic_name) = &arg_spec.arg_type {
            if let Some(ArgValue::String(value)) = actual.get(name) {
                let Some(allowed_values) = ctx.dynamic_values.get(*dynamic_name) else {
                    diagnostics.push(Diagnostic {
                        level: Level::Error,
                        code: ErrorCode::InvalidDynamicEnumValue,
                        message: format!("dynamic enum '{}' is not provided", dynamic_name),
                        span: span.clone(),
                        suggestion: None,
                    });
                    continue;
                };

                if !allowed_values.contains(value) {
                    let mut candidates: Vec<_> = allowed_values.iter().cloned().collect();
                    candidates.sort();
                    diagnostics.push(Diagnostic {
                        level: Level::Error,
                        code: ErrorCode::InvalidDynamicEnumValue,
                        message: format!("'{}' is not in dynamic enum '{}'", value, dynamic_name),
                        span: span.clone(),
                        suggestion: Some(format!("allowed: {}", candidates.join(", "))),
                    });
                }
            }
        }
    }
}
