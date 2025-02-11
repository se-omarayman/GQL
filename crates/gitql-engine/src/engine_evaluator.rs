use gitql_ast::expression::ArithmeticExpression;
use gitql_ast::expression::ArithmeticOperator;
use gitql_ast::expression::BetweenExpression;
use gitql_ast::expression::BitwiseExpression;
use gitql_ast::expression::BitwiseOperator;
use gitql_ast::expression::BooleanExpression;
use gitql_ast::expression::CallExpression;
use gitql_ast::expression::CaseExpression;
use gitql_ast::expression::ComparisonExpression;
use gitql_ast::expression::ComparisonOperator;
use gitql_ast::expression::Expression;
use gitql_ast::expression::ExpressionKind::*;
use gitql_ast::expression::InExpression;
use gitql_ast::expression::LikeExpression;
use gitql_ast::expression::LogicalExpression;
use gitql_ast::expression::LogicalOperator;
use gitql_ast::expression::NumberExpression;
use gitql_ast::expression::PrefixUnary;
use gitql_ast::expression::PrefixUnaryOperator;
use gitql_ast::expression::StringExpression;
use gitql_ast::expression::SymbolExpression;
use gitql_ast::function::FUNCTIONS;
use gitql_ast::types::DataType;
use gitql_ast::value::Value;

use regex::Regex;
use std::collections::HashMap;
use std::string::String;

pub fn evaluate_expression(
    expression: &Box<dyn Expression>,
    object: &HashMap<String, Value>,
) -> Result<Value, String> {
    match expression.get_expression_kind() {
        String => {
            let expr = expression
                .as_any()
                .downcast_ref::<StringExpression>()
                .unwrap();
            return evaluate_string(expr);
        }
        Symbol => {
            let expr = expression
                .as_any()
                .downcast_ref::<SymbolExpression>()
                .unwrap();
            return evaluate_symbol(expr, object);
        }
        Number => {
            let expr = expression
                .as_any()
                .downcast_ref::<NumberExpression>()
                .unwrap();
            return evaluate_number(expr);
        }
        Boolean => {
            let expr = expression
                .as_any()
                .downcast_ref::<BooleanExpression>()
                .unwrap();
            return evaluate_boolean(expr);
        }
        PrefixUnary => {
            let expr = expression.as_any().downcast_ref::<PrefixUnary>().unwrap();
            return evaluate_prefix_unary(expr, object);
        }
        Arithmetic => {
            let expr = expression
                .as_any()
                .downcast_ref::<ArithmeticExpression>()
                .unwrap();
            return evaluate_arithmetic(expr, object);
        }
        Comparison => {
            let expr = expression
                .as_any()
                .downcast_ref::<ComparisonExpression>()
                .unwrap();
            return evaluate_comparison(expr, object);
        }
        Like => {
            let expr = expression
                .as_any()
                .downcast_ref::<LikeExpression>()
                .unwrap();
            return evaulate_like(expr, object);
        }
        Logical => {
            let expr = expression
                .as_any()
                .downcast_ref::<LogicalExpression>()
                .unwrap();
            return evaluate_logical(expr, object);
        }
        Bitwise => {
            let expr = expression
                .as_any()
                .downcast_ref::<BitwiseExpression>()
                .unwrap();
            return evaluate_bitwise(expr, object);
        }
        Call => {
            let expr = expression
                .as_any()
                .downcast_ref::<CallExpression>()
                .unwrap();
            return evaluate_call(expr, object);
        }
        Between => {
            let expr = expression
                .as_any()
                .downcast_ref::<BetweenExpression>()
                .unwrap();
            return evaluate_between(expr, object);
        }
        Case => {
            let expr = expression
                .as_any()
                .downcast_ref::<CaseExpression>()
                .unwrap();
            return evaluate_case(expr, object);
        }
        In => {
            let expr = expression.as_any().downcast_ref::<InExpression>().unwrap();
            return evaluate_in(expr, object);
        }
    };
}

fn evaluate_string(expr: &StringExpression) -> Result<Value, String> {
    return Ok(Value::Text(expr.value.to_owned()));
}

fn evaluate_symbol(
    expr: &SymbolExpression,
    object: &HashMap<String, Value>,
) -> Result<Value, String> {
    if object.contains_key(&expr.value) {
        return Ok(object.get(&expr.value).unwrap().clone());
    }
    return Err(format!("Invalid column name `{}`", &expr.value));
}

fn evaluate_number(expr: &NumberExpression) -> Result<Value, String> {
    return Ok(expr.value.to_owned());
}

fn evaluate_boolean(expr: &BooleanExpression) -> Result<Value, String> {
    return Ok(Value::Boolean(expr.is_true));
}

fn evaluate_prefix_unary(
    expr: &PrefixUnary,
    object: &HashMap<String, Value>,
) -> Result<Value, String> {
    let rhs = evaluate_expression(&expr.right, object)?;
    return if expr.op == PrefixUnaryOperator::Bang {
        Ok(Value::Boolean(!rhs.as_bool()))
    } else {
        Ok(Value::Integer(-rhs.as_int()))
    };
}

fn evaluate_arithmetic(
    expr: &ArithmeticExpression,
    object: &HashMap<String, Value>,
) -> Result<Value, String> {
    let lhs = evaluate_expression(&expr.left, object)?;
    let rhs = evaluate_expression(&expr.right, object)?;

    return match expr.operator {
        ArithmeticOperator::Plus => Ok(lhs.plus(&rhs)),
        ArithmeticOperator::Minus => Ok(lhs.minus(&rhs)),
        ArithmeticOperator::Star => lhs.mul(&rhs),
        ArithmeticOperator::Slash => lhs.div(&rhs),
        ArithmeticOperator::Modulus => lhs.modulus(&rhs),
    };
}

fn evaluate_comparison(
    expr: &ComparisonExpression,
    object: &HashMap<String, Value>,
) -> Result<Value, String> {
    let lhs = evaluate_expression(&expr.left, object)?;
    let rhs = evaluate_expression(&expr.right, object)?;

    let left_type = lhs.data_type();
    let comparison_result = if left_type == DataType::Integer {
        let ilhs = lhs.as_int();
        let irhs = rhs.as_int();
        ilhs.cmp(&irhs)
    } else if left_type == DataType::Float {
        let ilhs = lhs.as_float();
        let irhs = rhs.as_float();
        ilhs.total_cmp(&irhs)
    } else if left_type == DataType::Boolean {
        let ilhs = lhs.as_bool();
        let irhs = rhs.as_bool();
        ilhs.cmp(&irhs)
    } else {
        lhs.as_text().cmp(&rhs.as_text())
    };

    return Ok(Value::Boolean(match expr.operator {
        ComparisonOperator::Greater => comparison_result.is_gt(),
        ComparisonOperator::GreaterEqual => comparison_result.is_ge(),
        ComparisonOperator::Less => comparison_result.is_lt(),
        ComparisonOperator::LessEqual => comparison_result.is_le(),
        ComparisonOperator::Equal => comparison_result.is_eq(),
        ComparisonOperator::NotEqual => !comparison_result.is_eq(),
    }));
}

fn evaulate_like(expr: &LikeExpression, object: &HashMap<String, Value>) -> Result<Value, String> {
    let rhs = evaluate_expression(&expr.pattern, object)?.as_text();
    if rhs.is_empty() {
        return Ok(Value::Boolean(false));
    }

    let pattern = &format!("^{}$", rhs.replace('%', ".*").replace('_', "."));
    let regex_result = Regex::new(pattern);
    if regex_result.is_err() {
        return Err(regex_result.err().unwrap().to_string());
    }
    let regex = regex_result.ok().unwrap();

    let lhs = evaluate_expression(&expr.input, object)?.as_text();
    return Ok(Value::Boolean(regex.is_match(&lhs)));
}

fn evaluate_logical(
    expr: &LogicalExpression,
    object: &HashMap<String, Value>,
) -> Result<Value, String> {
    let lhs = evaluate_expression(&expr.left, object)?.as_bool();
    if expr.operator == LogicalOperator::And && !lhs {
        return Ok(Value::Boolean(false));
    }

    if expr.operator == LogicalOperator::Or && lhs {
        return Ok(Value::Boolean(true));
    }

    let rhs = evaluate_expression(&expr.right, object)?.as_bool();

    return Ok(Value::Boolean(match expr.operator {
        LogicalOperator::And => lhs && rhs,
        LogicalOperator::Or => lhs || rhs,
        LogicalOperator::Xor => lhs ^ rhs,
    }));
}

fn evaluate_bitwise(
    expr: &BitwiseExpression,
    object: &HashMap<String, Value>,
) -> Result<Value, String> {
    let lhs = evaluate_expression(&expr.left, object)?.as_int();
    let rhs = evaluate_expression(&expr.right, object)?.as_int();

    return match expr.operator {
        BitwiseOperator::Or => Ok(Value::Integer(lhs | rhs)),
        BitwiseOperator::And => Ok(Value::Integer(lhs & rhs)),
        BitwiseOperator::RightShift => {
            if rhs >= 64 {
                Err("Attempt to shift right with overflow".to_string())
            } else {
                Ok(Value::Integer(lhs >> rhs))
            }
        }
        BitwiseOperator::LeftShift => {
            if rhs >= 64 {
                Err("Attempt to shift left with overflow".to_string())
            } else {
                Ok(Value::Integer(lhs << rhs))
            }
        }
    };
}

fn evaluate_call(expr: &CallExpression, object: &HashMap<String, Value>) -> Result<Value, String> {
    let function_name = expr.function_name.as_str();
    let function = FUNCTIONS.get(function_name).unwrap();

    let mut arguments = vec![];
    for arg in expr.arguments.iter() {
        arguments.push(evaluate_expression(arg, object)?);
    }

    return Ok(function(arguments));
}

fn evaluate_between(
    expr: &BetweenExpression,
    object: &HashMap<String, Value>,
) -> Result<Value, String> {
    let value_result = evaluate_expression(&expr.value, object);
    if value_result.is_err() {
        return value_result;
    }

    let range_start_result = evaluate_expression(&expr.range_start, object);
    if range_start_result.is_err() {
        return range_start_result;
    }

    let range_end_result = evaluate_expression(&expr.range_end, object);
    if range_end_result.is_err() {
        return range_end_result;
    }

    let value = value_result.ok().unwrap().as_int();
    let range_start = range_start_result.ok().unwrap().as_int();
    let range_end = range_end_result.ok().unwrap().as_int();
    return Ok(Value::Boolean(value >= range_start && value <= range_end));
}

fn evaluate_case(expr: &CaseExpression, object: &HashMap<String, Value>) -> Result<Value, String> {
    let conditions = &expr.conditions;
    let values = &expr.values;

    for i in 0..conditions.len() {
        let condition = evaluate_expression(&conditions[i], object)?;
        if condition.as_bool() {
            return evaluate_expression(&values[i], object);
        }
    }

    return match &expr.default_value {
        Some(default_value) => evaluate_expression(default_value, object),
        _ => Err("Invalid case statement".to_owned()),
    };
}

fn evaluate_in(expr: &InExpression, object: &HashMap<String, Value>) -> Result<Value, String> {
    let argument = evaluate_expression(&expr.argument, object)?;
    for value_expr in &expr.values {
        let value = evaluate_expression(value_expr, object)?;
        if argument.eq(&value) {
            return Ok(Value::Boolean(true));
        }
    }
    return Ok(Value::Boolean(false));
}
