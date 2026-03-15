use pest::iterators::Pair;
use ish_ast::*;

use crate::error::ParseError;
use crate::Rule;

pub fn build_program(pairs: pest::iterators::Pairs<Rule>) -> Result<Program, Vec<ParseError>> {
    let mut statements = Vec::new();
    let mut errors = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::program => {
                // Recurse into the program's inner pairs
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::EOI => {}
                        _ => match build_statement(inner) {
                            Ok(stmt) => statements.push(stmt),
                            Err(e) => errors.push(e),
                        },
                    }
                }
            }
            Rule::EOI => {}
            _ => match build_statement(pair) {
                Ok(stmt) => statements.push(stmt),
                Err(e) => errors.push(e),
            },
        }
    }

    if errors.is_empty() {
        Ok(Program { statements })
    } else {
        Err(errors)
    }
}

fn build_statement(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    match pair.as_rule() {
        Rule::let_stmt => build_let_stmt(pair),
        Rule::assign_stmt => build_assign_stmt(pair),
        Rule::fn_decl => build_fn_decl(pair),
        Rule::if_stmt => build_if_stmt(pair),
        Rule::while_stmt => build_while_stmt(pair),
        Rule::for_stmt => build_for_stmt(pair),
        Rule::return_stmt => build_return_stmt(pair),
        Rule::throw_stmt => build_throw_stmt(pair),
        Rule::try_catch => build_try_catch(pair),
        Rule::with_block => build_with_block(pair),
        Rule::defer_stmt => build_defer_stmt(pair),
        Rule::type_alias => build_type_alias(pair),
        Rule::use_stmt => build_use_stmt(pair),
        Rule::mod_stmt => build_mod_stmt(pair),
        Rule::shell_command => build_shell_command(pair),
        Rule::annotated_stmt => build_annotated_stmt(pair),
        Rule::standard_def => build_standard_def(pair),
        Rule::entry_type_def => build_entry_type_def(pair),
        Rule::match_stmt => build_match_stmt(pair),
        Rule::expression_stmt => {
            let inner = pair.into_inner().next().unwrap();
            Ok(Statement::ExpressionStmt(build_expression(inner)?))
        }
        Rule::block => build_block(pair),
        _ => {
            let span = pair.as_span();
            Err(ParseError::new(
                span.start(),
                span.end(),
                format!("unexpected rule: {:?}", pair.as_rule()),
            ))
        }
    }
}

fn build_let_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner().peekable();

    let visibility = if inner.peek().map(|p| p.as_rule()) == Some(Rule::pub_modifier) {
        Some(build_visibility(inner.next().unwrap()))
    } else {
        None
    };

    let mut mutable = false;
    if inner.peek().map(|p| p.as_rule()) == Some(Rule::mut_kw) {
        mutable = true;
        inner.next();
    }

    let name = inner.next().unwrap().as_str().to_string();

    let mut type_annotation = None;
    let mut value = None;

    for p in inner {
        match p.as_rule() {
            Rule::type_annotation => {
                type_annotation = Some(build_type_annotation(p)?);
            }
            _ => {
                value = Some(build_expression(p)?);
            }
        }
    }

    Ok(Statement::VariableDecl {
        name,
        mutable,
        type_annotation,
        value: value.unwrap(),
        visibility,
    })
}

fn build_assign_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner();
    let target_pair = inner.next().unwrap();
    let value_pair = inner.next().unwrap();

    let target = build_assign_target(target_pair)?;
    let value = build_expression(value_pair)?;

    Ok(Statement::Assignment { target, value })
}

fn build_assign_target(pair: Pair<Rule>) -> Result<AssignTarget, ParseError> {
    let mut inner = pair.into_inner();
    let base_name = inner.next().unwrap().as_str().to_string();

    let mut target = AssignTarget::Variable(base_name.clone());
    let mut current_expr = Expression::Identifier(base_name);

    for p in inner {
        match p.as_rule() {
            Rule::identifier => {
                let prop = p.as_str().to_string();
                target = AssignTarget::Property {
                    object: Box::new(current_expr.clone()),
                    property: prop.clone(),
                };
                current_expr = Expression::PropertyAccess {
                    object: Box::new(current_expr),
                    property: prop,
                };
            }
            _ => {
                // index access expression
                let index_expr = build_expression(p)?;
                target = AssignTarget::Index {
                    object: Box::new(current_expr.clone()),
                    index: Box::new(index_expr.clone()),
                };
                current_expr = Expression::IndexAccess {
                    object: Box::new(current_expr),
                    index: Box::new(index_expr),
                };
            }
        }
    }

    Ok(target)
}

fn build_fn_decl(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner().peekable();

    let visibility = if inner.peek().map(|p| p.as_rule()) == Some(Rule::pub_modifier) {
        Some(build_visibility(inner.next().unwrap()))
    } else {
        None
    };

    let name = inner.next().unwrap().as_str().to_string();

    let mut type_params = Vec::new();
    let mut params = Vec::new();
    let mut return_type = None;
    let mut body = None;

    for p in inner {
        match p.as_rule() {
            Rule::generic_params => {
                type_params = p.into_inner()
                    .map(|id| id.as_str().to_string())
                    .collect();
            }
            Rule::param_list => {
                params = build_param_list(p)?;
            }
            Rule::type_annotation => {
                return_type = Some(build_type_annotation(p)?);
            }
            Rule::block => {
                body = Some(build_block(p)?);
            }
            _ => {}
        }
    }

    Ok(Statement::FunctionDecl {
        name,
        params,
        return_type,
        body: Box::new(body.unwrap()),
        visibility,
        type_params,
    })
}

fn build_param_list(pair: Pair<Rule>) -> Result<Vec<Parameter>, ParseError> {
    let mut params = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() == Rule::param {
            params.push(build_param(p)?);
        }
    }
    Ok(params)
}

fn build_param(pair: Pair<Rule>) -> Result<Parameter, ParseError> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();

    let mut type_annotation = None;
    let mut default_value = None;

    for p in inner {
        match p.as_rule() {
            Rule::type_annotation => {
                type_annotation = Some(build_type_annotation(p)?);
            }
            _ => {
                default_value = Some(build_expression(p)?);
            }
        }
    }

    Ok(Parameter {
        name,
        type_annotation,
        default_value,
    })
}

fn build_block(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut statements = Vec::new();
    for p in pair.into_inner() {
        statements.push(build_statement(p)?);
    }
    Ok(Statement::Block { statements })
}

fn build_if_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner();

    let condition_pair = inner.next().unwrap();
    let condition = build_expression(condition_pair)?;

    let then_pair = inner.next().unwrap();
    let then_block = build_block(then_pair)?;

    let else_block = if let Some(else_pair) = inner.next() {
        match else_pair.as_rule() {
            Rule::if_stmt => Some(Box::new(build_if_stmt(else_pair)?)),
            Rule::block => Some(Box::new(build_block(else_pair)?)),
            _ => None,
        }
    } else {
        None
    };

    Ok(Statement::If {
        condition,
        then_block: Box::new(then_block),
        else_block,
    })
}

fn build_while_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner();
    let condition = build_expression(inner.next().unwrap())?;
    let body = build_block(inner.next().unwrap())?;

    Ok(Statement::While {
        condition,
        body: Box::new(body),
    })
}

fn build_for_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner();
    let variable = inner.next().unwrap().as_str().to_string();
    let iterable = build_expression(inner.next().unwrap())?;
    let body = build_block(inner.next().unwrap())?;

    Ok(Statement::ForEach {
        variable,
        iterable,
        body: Box::new(body),
    })
}

fn build_return_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let value = pair.into_inner().next().map(|p| build_expression(p)).transpose()?;
    Ok(Statement::Return { value })
}

fn build_throw_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let value = build_expression(pair.into_inner().next().unwrap())?;
    Ok(Statement::Throw { value })
}

fn build_try_catch(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner();
    let body = build_block(inner.next().unwrap())?;
    let mut catches = Vec::new();
    let mut finally = None;

    for p in inner {
        match p.as_rule() {
            Rule::catch_clause => {
                let mut catch_inner = p.into_inner();
                let param = catch_inner.next().unwrap().as_str().to_string();
                let mut type_annotation = None;
                let mut body = None;
                for cp in catch_inner {
                    match cp.as_rule() {
                        Rule::type_annotation => {
                            type_annotation = Some(build_type_annotation(cp)?);
                        }
                        Rule::block => {
                            body = Some(build_block(cp)?);
                        }
                        _ => {}
                    }
                }
                catches.push(CatchClause {
                    param,
                    type_annotation,
                    body: body.unwrap(),
                });
            }
            Rule::block => {
                // This is the finally block
                finally = Some(Box::new(build_block(p)?));
            }
            _ => {}
        }
    }

    Ok(Statement::TryCatch {
        body: Box::new(body),
        catches,
        finally,
    })
}

fn build_with_block(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let inner = pair.into_inner();
    let mut resources = Vec::new();
    let mut body = None;

    for p in inner {
        match p.as_rule() {
            Rule::resource_binding => {
                let mut rb_inner = p.into_inner();
                let name = rb_inner.next().unwrap().as_str().to_string();
                let value = build_expression(rb_inner.next().unwrap())?;
                resources.push((name, value));
            }
            Rule::block => {
                body = Some(build_block(p)?);
            }
            _ => {}
        }
    }

    Ok(Statement::WithBlock {
        resources,
        body: Box::new(body.unwrap()),
    })
}

fn build_defer_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    let body = match inner.as_rule() {
        Rule::block => build_block(inner)?,
        Rule::expression_stmt => {
            let expr = build_expression(inner.into_inner().next().unwrap())?;
            Statement::ExpressionStmt(expr)
        }
        _ => build_statement(inner)?,
    };
    Ok(Statement::Defer { body: Box::new(body) })
}

fn build_type_annotation(pair: Pair<Rule>) -> Result<TypeAnnotation, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    build_union_type(inner)
}

fn build_union_type(pair: Pair<Rule>) -> Result<TypeAnnotation, ParseError> {
    let types: Vec<TypeAnnotation> = pair
        .into_inner()
        .map(|p| build_primary_type(p))
        .collect::<Result<Vec<_>, _>>()?;

    if types.len() == 1 {
        Ok(types.into_iter().next().unwrap())
    } else {
        Ok(TypeAnnotation::Union(types))
    }
}

fn build_primary_type(pair: Pair<Rule>) -> Result<TypeAnnotation, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::simple_type => {
            let inner_text = inner.as_str().trim();
            if inner_text == "null" {
                Ok(TypeAnnotation::Simple("null".to_string()))
            } else {
                let name = inner.into_inner().next().unwrap().as_str().to_string();
                Ok(TypeAnnotation::Simple(name))
            }
        }
        Rule::list_type => {
            let mut parts = inner.into_inner();
            let _base = parts.next().unwrap(); // the identifier (e.g., "List")
            let element_type = build_type_annotation(parts.next().unwrap())?;
            Ok(TypeAnnotation::List(Box::new(element_type)))
        }
        Rule::object_type => {
            let fields = inner
                .into_inner()
                .filter(|p| p.as_rule() == Rule::object_type_field)
                .map(|p| {
                    let mut parts = p.into_inner();
                    let name = parts.next().unwrap().as_str().to_string();
                    // Skip the optional '?' marker for now
                    let type_ann = parts
                        .find(|p| p.as_rule() == Rule::type_annotation)
                        .unwrap();
                    let ty = build_type_annotation(type_ann)?;
                    Ok((name, ty))
                })
                .collect::<Result<Vec<_>, ParseError>>()?;
            Ok(TypeAnnotation::Object(fields))
        }
        Rule::function_type => {
            let parts = inner.into_inner();
            let mut params = Vec::new();
            let mut ret = None;
            for p in parts {
                match p.as_rule() {
                    Rule::type_list => {
                        params = p.into_inner()
                            .map(|t| build_type_annotation(t))
                            .collect::<Result<Vec<_>, _>>()?;
                    }
                    Rule::type_annotation => {
                        ret = Some(build_type_annotation(p)?);
                    }
                    _ => {}
                }
            }
            Ok(TypeAnnotation::Function {
                params,
                ret: Box::new(ret.unwrap_or(TypeAnnotation::Simple("void".into()))),
            })
        }
        Rule::tuple_type => {
            let types = inner
                .into_inner()
                .map(|p| build_type_annotation(p))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(TypeAnnotation::Tuple(types))
        }
        Rule::generic_type => {
            let mut parts = inner.into_inner();
            let base = parts.next().unwrap().as_str().to_string();
            let type_args = parts
                .map(|p| build_type_annotation(p))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(TypeAnnotation::Generic { base, type_args })
        }
        _ => {
            let span = inner.as_span();
            Err(ParseError::new(
                span.start(),
                span.end(),
                format!("unexpected type rule: {:?}", inner.as_rule()),
            ))
        }
    }
}

fn build_type_alias(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner().peekable();

    let visibility = if inner.peek().map(|p| p.as_rule()) == Some(Rule::pub_modifier) {
        Some(build_visibility(inner.next().unwrap()))
    } else {
        None
    };

    let name = inner.next().unwrap().as_str().to_string();
    let definition = build_type_annotation(inner.next().unwrap())?;
    Ok(Statement::TypeAlias { name, definition, visibility })
}

fn build_visibility(pair: Pair<Rule>) -> Visibility {
    let inner: Vec<_> = pair.into_inner().collect();
    if inner.is_empty() {
        Visibility::Public
    } else {
        Visibility::PubScope(inner[0].as_str().to_string())
    }
}

fn build_use_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let module_path = pair.into_inner().next().unwrap();
    let path: Vec<String> = module_path
        .into_inner()
        .map(|p| p.as_str().to_string())
        .collect();
    Ok(Statement::Use { path })
}

fn build_mod_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner().peekable();

    let visibility = if inner.peek().map(|p| p.as_rule()) == Some(Rule::pub_modifier) {
        Some(build_visibility(inner.next().unwrap()))
    } else {
        None
    };

    let name = inner.next().unwrap().as_str().to_string();
    let body = if let Some(block_pair) = inner.next() {
        Some(Box::new(build_block(block_pair)?))
    } else {
        None
    };
    Ok(Statement::ModDecl { name, body, visibility })
}

fn build_annotated_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let inner = pair.into_inner();
    let mut annotations = Vec::new();

    // Collect annotations until we hit the inner statement
    let mut last = None;
    for p in inner {
        match p.as_rule() {
            Rule::annotation => {
                let ann_inner = p.into_inner().next().unwrap();
                match ann_inner.as_rule() {
                    Rule::standard_annotation => {
                        let args = ann_inner.into_inner().next().unwrap().as_str().trim().to_string();
                        annotations.push(Annotation::Standard(args));
                    }
                    Rule::entry_annotation => {
                        let items: Vec<EntryItem> = ann_inner
                            .into_inner()
                            .filter(|p| p.as_rule() == Rule::entry_item)
                            .map(|p| {
                                let mut item_inner = p.into_inner();
                                let name = item_inner.next().unwrap().as_str().to_string();
                                let value = item_inner.next().map(|v| v.as_str().trim().to_string());
                                EntryItem { name, value }
                            })
                            .collect();
                        annotations.push(Annotation::Entry(items));
                    }
                    _ => {}
                }
            }
            _ => {
                last = Some(p);
            }
        }
    }

    let inner_stmt = build_statement(last.unwrap())?;
    Ok(Statement::Annotated {
        annotations,
        inner: Box::new(inner_stmt),
    })
}

fn build_standard_def(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();

    let mut extends = None;
    let mut features = Vec::new();

    for p in inner {
        match p.as_rule() {
            Rule::identifier => {
                extends = Some(p.as_str().to_string());
            }
            Rule::feature_spec => {
                let mut feat_inner = p.into_inner();
                let feat_name = feat_inner.next().unwrap().as_str().to_string();
                let params: Vec<String> = feat_inner
                    .filter(|p| p.as_rule() == Rule::feature_param)
                    .map(|p| p.as_str().trim().to_string())
                    .collect();
                features.push(FeatureSpec { name: feat_name, params });
            }
            _ => {}
        }
    }

    Ok(Statement::StandardDef { name, extends, features })
}

fn build_entry_type_def(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();

    let mut fields = Vec::new();
    for p in inner {
        if p.as_rule() == Rule::entry_type_field {
            let mut field_inner = p.into_inner();
            let field_name = field_inner.next().unwrap().as_str().to_string();
            let field_value = build_expression(field_inner.next().unwrap())?;
            fields.push((field_name, field_value));
        }
    }

    Ok(Statement::EntryTypeDef { name, fields })
}

fn build_shell_command(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner().peekable();

    // Optional force-command prefix (>)
    if inner.peek().map(|p| p.as_rule()) == Some(Rule::force_command_prefix) {
        inner.next(); // consume the >
    }

    // Parse the pipeline
    let pipeline_pair = inner.next().unwrap();
    let mut pipeline_inner = pipeline_pair.into_inner();

    // First simple command
    let first_cmd_pair = pipeline_inner.next().unwrap();
    let (command, args, redirections) = build_shell_simple_command(first_cmd_pair);

    // Remaining piped commands
    let mut pipes = Vec::new();
    while let Some(next) = pipeline_inner.next() {
        match next.as_rule() {
            Rule::shell_pipe => {
                // Next should be a simple command
                if let Some(cmd_pair) = pipeline_inner.next() {
                    let (pipe_cmd, pipe_args, _) = build_shell_simple_command(cmd_pair);
                    pipes.push(ShellPipeline {
                        command: pipe_cmd,
                        args: pipe_args,
                    });
                }
            }
            Rule::shell_simple_command => {
                let (pipe_cmd, pipe_args, _) = build_shell_simple_command(next);
                pipes.push(ShellPipeline {
                    command: pipe_cmd,
                    args: pipe_args,
                });
            }
            _ => {}
        }
    }

    // Background
    let background = inner.peek().map(|p| p.as_rule()) == Some(Rule::shell_background);

    Ok(Statement::ShellCommand {
        command,
        args,
        pipes,
        redirections,
        background,
    })
}

fn build_shell_simple_command(pair: Pair<Rule>) -> (String, Vec<ShellArg>, Vec<Redirection>) {
    let mut words = Vec::new();
    let mut redirections = Vec::new();

    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::shell_word => {
                let word_inner = p.into_inner().next().unwrap();
                match word_inner.as_rule() {
                    Rule::shell_bare_word => {
                        let text = word_inner.as_str().to_string();
                        // Detect globs
                        if text.contains('*') || text.contains('?') || (text.contains('[') && text.contains(']')) {
                            words.push(ShellArg::Glob(text));
                        } else {
                            words.push(ShellArg::Bare(text));
                        }
                    }
                    Rule::shell_quoted_string => {
                        let inner = word_inner.into_inner().next().unwrap();
                        words.push(ShellArg::Quoted(inner.as_str().to_string()));
                    }
                    Rule::shell_single_string => {
                        let inner = word_inner.into_inner().next().unwrap();
                        words.push(ShellArg::Quoted(inner.as_str().to_string()));
                    }
                    Rule::env_var => {
                        let raw = word_inner.as_str();
                        let name = if raw.starts_with("${") && raw.ends_with('}') {
                            &raw[2..raw.len() - 1]
                        } else {
                            &raw[1..]
                        };
                        words.push(ShellArg::EnvVar(name.to_string()));
                    }
                    Rule::command_substitution => {
                        if let Ok(cmd) = build_command_sub_inner(word_inner) {
                            words.push(ShellArg::CommandSub(Box::new(cmd)));
                        }
                    }
                    _ => {
                        words.push(ShellArg::Bare(word_inner.as_str().to_string()));
                    }
                }
            }
            Rule::shell_redirection => {
                let mut redir_inner = p.into_inner();
                let op = redir_inner.next().unwrap();
                let target_pair = redir_inner.next().unwrap();
                let target_word = target_pair.into_inner().next().unwrap();
                let target = target_word.as_str().to_string();
                let kind = match op.as_str() {
                    ">>" => RedirectKind::StdoutAppend,
                    "2>&1" => RedirectKind::StderrAndStdout,
                    "2>" => RedirectKind::StderrWrite,
                    "&>" => RedirectKind::AllWrite,
                    _ => RedirectKind::StdoutWrite,
                };
                redirections.push(Redirection { kind, target });
            }
            _ => {}
        }
    }

    // First word is the command, rest are args
    let command = if let Some(first) = words.first() {
        match first {
            ShellArg::Bare(s) | ShellArg::Glob(s) => s.clone(),
            _ => String::new(),
        }
    } else {
        String::new()
    };
    let args = if words.len() > 1 { words[1..].to_vec() } else { Vec::new() };
    (command, args, redirections)
}

fn build_command_sub_inner(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    // command_substitution = { "$(" ~ statement* ~ ")" }
    let mut statements = Vec::new();
    for p in pair.into_inner() {
        statements.push(build_statement(p)?);
    }
    if statements.len() == 1 {
        Ok(statements.into_iter().next().unwrap())
    } else {
        Ok(Statement::Block { statements })
    }
}

fn build_command_substitution(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let cmd = build_command_sub_inner(pair)?;
    Ok(Expression::CommandSubstitution(Box::new(cmd)))
}

// --- Expression building ---

pub fn build_expression(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    match pair.as_rule() {
        Rule::expression | Rule::or_expr | Rule::and_expr | Rule::comparison
        | Rule::addition | Rule::multiplication | Rule::unary | Rule::postfix
        | Rule::primary => build_expr_inner(pair),

        Rule::integer_literal => {
            let val: i64 = pair.as_str().parse().map_err(|_| {
                ParseError::new(pair.as_span().start(), pair.as_span().end(), "invalid integer")
            })?;
            Ok(Expression::Literal(Literal::Int(val)))
        }
        Rule::float_literal => {
            let val: f64 = pair.as_str().parse().map_err(|_| {
                ParseError::new(pair.as_span().start(), pair.as_span().end(), "invalid float")
            })?;
            Ok(Expression::Literal(Literal::Float(val)))
        }
        Rule::boolean_literal => {
            Ok(Expression::Literal(Literal::Bool(pair.as_str() == "true")))
        }
        Rule::null_literal => Ok(Expression::Literal(Literal::Null)),
        Rule::string_literal => {
            let inner = pair.into_inner().next().unwrap();
            let s = unescape_single_string(inner.as_str());
            Ok(Expression::Literal(Literal::String(s)))
        }
        Rule::identifier => Ok(Expression::Identifier(pair.as_str().to_string())),

        Rule::object_literal => build_object_literal(pair),
        Rule::list_literal => build_list_literal(pair),
        Rule::lambda => build_lambda(pair),
        Rule::interp_string => build_interp_string(pair),
        Rule::triple_double_string => build_triple_double_string(pair),
        Rule::triple_single_string => build_triple_single_string(pair),
        Rule::char_literal => build_char_literal(pair),
        Rule::extended_double_string => build_extended_string(pair),
        Rule::extended_single_string => build_extended_string(pair),
        Rule::extended_triple_double_string => build_extended_string(pair),
        Rule::extended_triple_single_string => build_extended_string(pair),
        Rule::command_substitution => build_command_substitution(pair),
        Rule::env_var => {
            let raw = pair.as_str();
            // Strip leading ${ and trailing } or leading $
            let name = if raw.starts_with("${") && raw.ends_with('}') {
                &raw[2..raw.len() - 1]
            } else {
                &raw[1..]
            };
            Ok(Expression::EnvVar(name.to_string()))
        }

        _ => {
            let span = pair.as_span();
            Err(ParseError::new(
                span.start(),
                span.end(),
                format!("unexpected expression rule: {:?}", pair.as_rule()),
            ))
        }
    }
}

fn build_expr_inner(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    match pair.as_rule() {
        Rule::expression => {
            let inner = pair.into_inner().next().unwrap();
            build_expr_inner(inner)
        }
        Rule::or_expr => build_binary_chain(pair, |op_str| match op_str {
            "or" => Some(BinaryOperator::Or),
            _ => None,
        }),
        Rule::and_expr => build_binary_chain(pair, |op_str| match op_str {
            "and" => Some(BinaryOperator::And),
            _ => None,
        }),
        Rule::comparison => build_comparison(pair),
        Rule::addition => build_binary_chain(pair, |op_str| match op_str {
            "+" => Some(BinaryOperator::Add),
            "-" => Some(BinaryOperator::Sub),
            _ => None,
        }),
        Rule::multiplication => build_binary_chain(pair, |op_str| match op_str {
            "*" => Some(BinaryOperator::Mul),
            "/" => Some(BinaryOperator::Div),
            "%" => Some(BinaryOperator::Mod),
            _ => None,
        }),
        Rule::unary => build_unary(pair),
        Rule::postfix => build_postfix(pair),
        Rule::primary => build_primary(pair),
        _ => build_expression(pair),
    }
}

fn build_binary_chain(
    pair: Pair<Rule>,
    op_map: impl Fn(&str) -> Option<BinaryOperator>,
) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();
    let mut left = build_expr_inner(first)?;

    while let Some(op_pair) = inner.next() {
        let op_str = op_pair.as_str();
        if let Some(op) = op_map(op_str) {
            let right_pair = inner.next().unwrap();
            let right = build_expr_inner(right_pair)?;
            left = Expression::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        } else {
            // The pair itself is the next operand (no explicit operator token for
            // rules that use inline operators like add_op/sub_op).
            let right = build_expr_inner(op_pair)?;
            // This shouldn't normally happen with proper grammar, but handle gracefully.
            return Ok(right);
        }
    }

    Ok(left)
}

fn build_comparison(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let left = build_expr_inner(inner.next().unwrap())?;

    if let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "==" => BinaryOperator::Eq,
            "!=" => BinaryOperator::NotEq,
            "<=" => BinaryOperator::LtEq,
            ">=" => BinaryOperator::GtEq,
            "<" => BinaryOperator::Lt,
            ">" => BinaryOperator::Gt,
            other => {
                return Err(ParseError::new(
                    op_pair.as_span().start(),
                    op_pair.as_span().end(),
                    format!("unknown comparison operator: {}", other),
                ))
            }
        };
        let right = build_expr_inner(inner.next().unwrap())?;
        Ok(Expression::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    } else {
        Ok(left)
    }
}

fn build_unary(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();

    match first.as_rule() {
        Rule::not_op => {
            let operand = build_expr_inner(inner.next().unwrap())?;
            Ok(Expression::UnaryOp {
                op: UnaryOperator::Not,
                operand: Box::new(operand),
            })
        }
        Rule::neg_op => {
            let operand = build_expr_inner(inner.next().unwrap())?;
            Ok(Expression::UnaryOp {
                op: UnaryOperator::Negate,
                operand: Box::new(operand),
            })
        }
        _ => build_expr_inner(first),
    }
}

fn build_postfix(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();
    let mut expr = build_expr_inner(first)?;

    for p in inner {
        match p.as_rule() {
            Rule::call_args => {
                let args = if let Some(arg_list) = p.into_inner().next() {
                    arg_list
                        .into_inner()
                        .map(|a| build_expression(a))
                        .collect::<Result<Vec<_>, _>>()?
                } else {
                    vec![]
                };
                expr = Expression::FunctionCall {
                    callee: Box::new(expr),
                    args,
                };
            }
            Rule::dot_access => {
                let prop = p.into_inner().next().unwrap().as_str().to_string();
                expr = Expression::PropertyAccess {
                    object: Box::new(expr),
                    property: prop,
                };
            }
            Rule::index_access => {
                let index = build_expression(p.into_inner().next().unwrap())?;
                expr = Expression::IndexAccess {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            }
            Rule::try_op => {
                expr = Expression::UnaryOp {
                    op: UnaryOperator::Try,
                    operand: Box::new(expr),
                };
            }
            _ => {}
        }
    }

    Ok(expr)
}

fn build_primary(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    build_expression(inner)
}

fn build_object_literal(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut pairs = Vec::new();
    for p in pair.into_inner() {
        if p.as_rule() == Rule::object_pair {
            let mut inner = p.into_inner();
            let key_pair = inner.next().unwrap();
            let key = match key_pair.as_rule() {
                Rule::identifier => key_pair.as_str().to_string(),
                Rule::string_literal => {
                    let s_inner = key_pair.into_inner().next().unwrap();
                    unescape_single_string(s_inner.as_str())
                }
                Rule::interp_string => {
                    // For object keys, only support non-interpolated double-quoted strings
                    let mut text = String::new();
                    for p in key_pair.into_inner() {
                        if p.as_rule() == Rule::interp_string_text {
                            text.push_str(&unescape_double_string(p.as_str()));
                        }
                    }
                    text
                }
                _ => key_pair.as_str().to_string(),
            };
            let value = build_expression(inner.next().unwrap())?;
            pairs.push((key, value));
        }
    }
    Ok(Expression::ObjectLiteral(pairs))
}

fn build_list_literal(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let elements = pair
        .into_inner()
        .map(|p| build_expression(p))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Expression::ListLiteral(elements))
}

fn build_lambda(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let inner = pair.into_inner();
    let mut params = Vec::new();

    // Collect params and find the body
    let mut body_pair = None;
    for p in inner {
        match p.as_rule() {
            Rule::param_list => {
                params = build_param_list(p)?;
            }
            Rule::block => {
                body_pair = Some(p);
            }
            _ => {
                // Must be an expression body
                let expr = build_expression(p)?;
                let body = Statement::Block {
                    statements: vec![Statement::Return { value: Some(expr) }],
                };
                return Ok(Expression::Lambda {
                    params,
                    body: Box::new(body),
                });
            }
        }
    }

    let body = if let Some(block_pair) = body_pair {
        build_block(block_pair)?
    } else {
        Statement::Block { statements: vec![] }
    };

    Ok(Expression::Lambda {
        params,
        body: Box::new(body),
    })
}

fn build_interp_string(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut parts = Vec::new();
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::interp_string_text => {
                parts.push(StringPart::Text(unescape_double_string(p.as_str())));
            }
            Rule::interp_string_interp => {
                let expr = build_expression(p.into_inner().next().unwrap())?;
                parts.push(StringPart::Expr(expr));
            }
            Rule::interp_string_env => {
                let raw = p.as_str();
                let name = &raw[1..]; // strip leading $
                parts.push(StringPart::Expr(Expression::EnvVar(name.to_string())));
            }
            _ => {}
        }
    }
    // If there are no interpolation parts, emit a plain Literal::String
    if parts.len() == 1 {
        if let StringPart::Text(ref text) = parts[0] {
            return Ok(Expression::Literal(Literal::String(text.clone())));
        }
    }
    if parts.is_empty() {
        return Ok(Expression::Literal(Literal::String(String::new())));
    }
    Ok(Expression::StringInterpolation(parts))
}

fn build_triple_double_string(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut parts = Vec::new();
    for p in pair.into_inner() {
        match p.as_rule() {
            Rule::triple_double_text => {
                parts.push(StringPart::Text(p.as_str().to_string()));
            }
            Rule::triple_double_interp => {
                let expr = build_expression(p.into_inner().next().unwrap())?;
                parts.push(StringPart::Expr(expr));
            }
            Rule::triple_double_env => {
                let raw = p.as_str();
                let name = &raw[1..];
                parts.push(StringPart::Expr(Expression::EnvVar(name.to_string())));
            }
            _ => {}
        }
    }
    let parts = strip_triple_quote_indentation(parts);
    if parts.len() == 1 {
        if let StringPart::Text(ref text) = parts[0] {
            return Ok(Expression::Literal(Literal::String(text.clone())));
        }
    }
    if parts.is_empty() {
        return Ok(Expression::Literal(Literal::String(String::new())));
    }
    Ok(Expression::StringInterpolation(parts))
}

fn build_triple_single_string(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    let raw = inner.as_str().to_string();
    // Strip indentation based on closing delimiter position
    let stripped = strip_triple_quote_literal(&raw);
    Ok(Expression::Literal(Literal::String(stripped)))
}

fn build_char_literal(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    let raw = inner.as_str();
    let ch = unescape_char_literal(raw).ok_or_else(|| {
        ParseError::new(0, 0, format!("invalid char literal: {}", raw))
    })?;
    Ok(Expression::Literal(Literal::Char(ch)))
}

fn build_extended_string(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    // All extended delimiter forms: just take the inner content verbatim
    let inner = pair.into_inner().next().unwrap();
    Ok(Expression::Literal(Literal::String(inner.as_str().to_string())))
}

fn unescape_single_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\'') => result.push('\''),
                Some('\\') => result.push('\\'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn unescape_double_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('/') => result.push('/'),
                Some('b') => result.push('\u{0008}'),
                Some('f') => result.push('\u{000C}'),
                Some('0') => result.push('\0'),
                Some('{') => result.push('{'),
                Some('}') => result.push('}'),
                Some('$') => result.push('$'),
                Some('u') => {
                    // \u{XXXX} form
                    if chars.next() == Some('{') {
                        let hex: String = chars.by_ref().take_while(|c| *c != '}').collect();
                        if let Ok(code) = u32::from_str_radix(&hex, 16) {
                            if let Some(ch) = char::from_u32(code) {
                                result.push(ch);
                            }
                        }
                    }
                }
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn unescape_char_literal(s: &str) -> Option<char> {
    let mut chars = s.chars();
    let first = chars.next()?;
    if first == '\\' {
        match chars.next()? {
            'n' => Some('\n'),
            'r' => Some('\r'),
            't' => Some('\t'),
            '\\' => Some('\\'),
            '\'' => Some('\''),
            '0' => Some('\0'),
            'u' => {
                if chars.next() == Some('{') {
                    let hex: String = chars.take_while(|c| *c != '}').collect();
                    u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32)
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        if chars.next().is_some() {
            None // more than one character
        } else {
            Some(first)
        }
    }
}

/// Strip indentation from triple-quoted literal strings.
/// Finds the indentation of the last line (before closing delimiter)
/// and removes that prefix from all lines.
fn strip_triple_quote_literal(s: &str) -> String {
    let lines: Vec<&str> = s.split('\n').collect();
    if lines.is_empty() {
        return String::new();
    }
    // The last line's leading whitespace defines the baseline
    let last_line = lines.last().unwrap();
    let baseline = last_line.len() - last_line.trim_start().len();
    let mut result_lines = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i == 0 && line.trim().is_empty() {
            continue; // skip leading empty line after opening delimiter
        }
        if i == lines.len() - 1 && line.trim().is_empty() {
            continue; // skip trailing line before closing delimiter
        }
        if line.len() >= baseline {
            result_lines.push(&line[baseline..]);
        } else {
            result_lines.push(line.trim_start());
        }
    }
    result_lines.join("\n")
}

/// Strip indentation from triple-quoted interpolated strings.
fn strip_triple_quote_indentation(parts: Vec<StringPart>) -> Vec<StringPart> {
    // For simplicity, find baseline from last text part's last line
    let mut baseline = 0;
    if let Some(StringPart::Text(ref text)) = parts.last() {
        let lines: Vec<&str> = text.split('\n').collect();
        if let Some(last) = lines.last() {
            baseline = last.len() - last.trim_start().len();
        }
    }
    if baseline == 0 {
        return parts;
    }
    let mut result = Vec::new();
    for part in parts {
        match part {
            StringPart::Text(text) => {
                let lines: Vec<&str> = text.split('\n').collect();
                let mut stripped = Vec::new();
                for line in &lines {
                    if line.len() >= baseline {
                        stripped.push(&line[baseline..]);
                    } else if line.trim().is_empty() {
                        stripped.push("");
                    } else {
                        stripped.push(line);
                    }
                }
                let joined = stripped.join("\n");
                if !joined.is_empty() {
                    result.push(StringPart::Text(joined));
                }
            }
            other => result.push(other),
        }
    }
    // Trim leading/trailing empty text parts from triple-quote boundaries
    if let Some(StringPart::Text(ref s)) = result.first() {
        if s.starts_with('\n') {
            let trimmed = s[1..].to_string();
            if trimmed.is_empty() {
                result.remove(0);
            } else {
                result[0] = StringPart::Text(trimmed);
            }
        }
    }
    if let Some(StringPart::Text(ref s)) = result.last() {
        if s.ends_with('\n') {
            let len = result.len();
            let trimmed = s[..s.len() - 1].to_string();
            if trimmed.is_empty() {
                result.remove(len - 1);
            } else {
                result[len - 1] = StringPart::Text(trimmed);
            }
        }
    }
    result
}

fn build_match_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner();
    let subject = build_expression(inner.next().unwrap())?;
    let mut arms = Vec::new();
    for p in inner {
        if p.as_rule() == Rule::match_arm {
            arms.push(build_match_arm(p)?);
        }
    }
    Ok(Statement::Match { subject, arms })
}

fn build_match_arm(pair: Pair<Rule>) -> Result<MatchArm, ParseError> {
    let mut inner = pair.into_inner();
    let pattern = build_match_pattern(inner.next().unwrap())?;
    let body_pair = inner.next().unwrap();
    let body = match body_pair.as_rule() {
        Rule::block => build_block(body_pair)?,
        Rule::expression_stmt => {
            let expr = build_expression(body_pair.into_inner().next().unwrap())?;
            Statement::ExpressionStmt(expr)
        }
        _ => {
            let span = body_pair.as_span();
            return Err(ParseError::new(span.start(), span.end(), "unexpected match arm body".to_string()));
        }
    };
    Ok(MatchArm { pattern, body })
}

fn build_match_pattern(pair: Pair<Rule>) -> Result<MatchPattern, ParseError> {
    // "_" is a string literal in the grammar, so it won't produce a child pair
    let mut inner_iter = pair.into_inner();
    let inner = match inner_iter.next() {
        Some(p) => p,
        None => return Ok(MatchPattern::Wildcard), // bare "_" matched
    };
    match inner.as_rule() {
        Rule::integer_literal => {
            let n: i64 = inner.as_str().parse().unwrap();
            Ok(MatchPattern::Literal(Literal::Int(n)))
        }
        Rule::float_literal => {
            let n: f64 = inner.as_str().parse().unwrap();
            Ok(MatchPattern::Literal(Literal::Float(n)))
        }
        Rule::boolean_literal => {
            Ok(MatchPattern::Literal(Literal::Bool(inner.as_str() == "true")))
        }
        Rule::null_literal => {
            Ok(MatchPattern::Literal(Literal::Null))
        }
        Rule::string_literal => {
            let raw = inner.into_inner().next().unwrap().as_str();
            Ok(MatchPattern::Literal(Literal::String(unescape_single_string(raw))))
        }
        Rule::char_literal => {
            let raw = inner.into_inner().next().unwrap().as_str();
            let ch = unescape_char_literal(raw).ok_or_else(|| {
                ParseError::new(0, 0, format!("invalid char literal in pattern: {}", raw))
            })?;
            Ok(MatchPattern::Literal(Literal::Char(ch)))
        }
        Rule::interp_string => {
            // In match patterns, interp_string is used only for non-interpolated doubles
            let mut text = String::new();
            for p in inner.into_inner() {
                if p.as_rule() == Rule::interp_string_text {
                    text.push_str(&unescape_double_string(p.as_str()));
                }
            }
            Ok(MatchPattern::Literal(Literal::String(text)))
        }
        Rule::identifier => {
            let name = inner.as_str().to_string();
            if name == "_" {
                Ok(MatchPattern::Wildcard)
            } else {
                Ok(MatchPattern::Identifier(name))
            }
        }
        _ => {
            let span = inner.as_span();
            Err(ParseError::new(span.start(), span.end(), "unexpected match pattern".to_string()))
        }
    }
}
