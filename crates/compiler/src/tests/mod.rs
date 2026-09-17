use std::rc::Rc;

use ast::statement::{
    Declaration, DeclarationKind, Expression, ExpressionKind, ForInit, ForStep, FunctionParam,
    Statement, StatementKind, Type,
};
use chunk::types::{Op, Value};
use token::keywords::Operator;
use token::token::Span;

use crate::types::Compiler;

fn span(line: u32) -> Span {
    Span {
        line,
        col: 1,
        len: 1,
    }
}

fn number(n: f64) -> Expression {
    Expression {
        kind: ExpressionKind::LiteralNumber(n),
        span: span(1),
    }
}

fn binary(op: Operator, left: Expression, right: Expression) -> Expression {
    Expression {
        kind: ExpressionKind::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        },
        span: span(1),
    }
}

fn unary(op: Operator, operand: Expression) -> Expression {
    Expression {
        kind: ExpressionKind::UnaryOp {
            op,
            operand: Box::new(operand),
        },
        span: span(1),
    }
}

fn disassemble(expression: Expression, name: &str) -> String {
    let mut compiler = Compiler::new();
    compiler.compile_expression(expression);
    compiler.chunk.format_disassembly(name)
}

#[test]
fn precedence_survives_lowering_as_operand_order_on_the_stack() {
    let expression = binary(
        Operator::Plus,
        number(1.0),
        binary(Operator::Multiply, number(2.0), number(3.0)),
    );

    assert_eq!(
        disassemble(expression, "1 + 2 * 3"),
        "\
Disassemble: 1 + 2 * 3
0000    1 Constant 0 -> 1
0001    | Constant 1 -> 2
0002    | Constant 2 -> 3
0003    | Mul
0004    | Add
"
    );
}

#[test]
fn a_derived_comparison_lowers_to_its_primitive_plus_a_negation() {
    let expression = binary(Operator::LessOrEqual, number(5.0), number(3.0));

    assert_eq!(
        disassemble(expression, "5 <= 3"),
        "\
Disassemble: 5 <= 3
0000    1 Constant 0 -> 5
0001    | Constant 1 -> 3
0002    | Greater
0003    | Not
"
    );
}

#[test]
fn a_unary_operator_is_emitted_after_its_operand() {
    let expression = unary(Operator::Minus, number(7.0));

    assert_eq!(
        disassemble(expression, "-7"),
        "\
Disassemble: -7
0000    1 Constant 0 -> 7
0001    | Negate
"
    );
}

#[test]
fn literals_that_have_their_own_opcode_allocate_no_constant() {
    let expression = Expression {
        kind: ExpressionKind::LiteralBoolean(true),
        span: span(1),
    };

    let mut compiler = Compiler::new();
    compiler.compile_expression(expression);

    assert!(compiler.chunk.constants.is_empty());
    assert_eq!(
        compiler.chunk.format_disassembly("true"),
        "Disassemble: true\n0000    1 True\n"
    );
}

fn string(s: &str) -> Expression {
    Expression {
        kind: ExpressionKind::LiteralString(s.to_string()),
        span: span(1),
    }
}

fn identifier(name: &str) -> Expression {
    Expression {
        kind: ExpressionKind::Identifier(name.to_string()),
        span: span(1),
    }
}

fn statement(kind: StatementKind) -> Statement {
    Statement {
        kind,
        span: span(1),
    }
}

fn declaration(kind: DeclarationKind) -> Declaration {
    Declaration {
        kind,
        span: span(1),
    }
}

fn let_decl(name: &str, init: Expression) -> Declaration {
    declaration(DeclarationKind::Let {
        name: name.to_string(),
        ty: Type::Number,
        init,
    })
}

fn expression_statement(expression: Expression) -> Statement {
    statement(StatementKind::ExpressionStatement(expression))
}

fn block(declarations: Vec<Declaration>) -> Statement {
    statement(StatementKind::Block(declarations))
}

fn compile_statement(statement: Statement) -> Compiler {
    let mut compiler = Compiler::new();
    compiler.compile_statement(statement);
    compiler
}

fn compile_declaration(declaration: Declaration) -> Compiler {
    let mut compiler = Compiler::new();
    compiler.compile_declaration(declaration);
    compiler
}

#[test]
fn an_expression_statement_discards_the_value_it_leaves_behind() {
    let compiler = compile_statement(expression_statement(number(1.0)));

    assert_eq!(compiler.chunk.code, vec![Op::Constant(0), Op::Pop]);
}

#[test]
fn a_bare_return_still_produces_a_value_for_the_caller_to_pop() {
    let compiler = compile_statement(statement(StatementKind::Return(None)));

    assert_eq!(compiler.chunk.code, vec![Op::Null, Op::Return]);
}

#[test]
fn a_return_with_an_operand_leaves_that_operand_on_the_stack() {
    let compiler = compile_statement(statement(StatementKind::Return(Some(number(42.0)))));

    assert_eq!(compiler.chunk.code, vec![Op::Constant(0), Op::Return]);
}

#[test]
fn a_top_level_let_becomes_a_global_keyed_by_its_name() {
    let compiler = compile_declaration(let_decl("x", number(1.0)));

    assert_eq!(
        compiler.chunk.code,
        vec![Op::Constant(0), Op::DefineGlobal(1)]
    );
    assert_eq!(compiler.chunk.constants[1], Value::Str(Rc::from("x")));
    assert!(compiler.locals.is_empty());
}

#[test]
fn a_let_inside_a_block_becomes_a_stack_slot_with_no_name_in_the_constants() {
    let compiler = compile_statement(block(vec![let_decl("x", number(1.0))]));

    assert_eq!(compiler.chunk.code, vec![Op::Constant(0), Op::Pop]);
    assert_eq!(compiler.chunk.constants, vec![Value::Number(1.0)]);
}

#[test]
fn a_block_pops_exactly_the_locals_it_introduced() {
    let compiler = compile_statement(block(vec![
        let_decl("a", number(1.0)),
        let_decl("b", number(2.0)),
    ]));

    assert_eq!(
        compiler.chunk.code,
        vec![Op::Constant(0), Op::Constant(1), Op::Pop, Op::Pop]
    );
    assert!(compiler.locals.is_empty());
}

#[test]
fn an_identifier_resolves_to_the_innermost_local_that_shadows_it() {
    let compiler = compile_statement(block(vec![
        let_decl("x", number(1.0)),
        let_decl("x", number(2.0)),
        declaration(DeclarationKind::Statement(Box::new(expression_statement(
            identifier("x"),
        )))),
    ]));

    assert_eq!(
        compiler.chunk.code,
        vec![
            Op::Constant(0),
            Op::Constant(1),
            Op::GetLocal(1),
            Op::Pop,
            Op::Pop,
            Op::Pop,
        ]
    );
}

#[test]
fn a_local_goes_out_of_scope_when_its_block_ends() {
    let compiler = compile_statement(block(vec![
        declaration(DeclarationKind::Statement(Box::new(block(vec![let_decl(
            "inner",
            number(1.0),
        )])))),
        declaration(DeclarationKind::Statement(Box::new(expression_statement(
            identifier("inner"),
        )))),
    ]));

    assert_eq!(
        compiler.chunk.code,
        vec![Op::Constant(0), Op::Pop, Op::GetGlobal(1), Op::Pop]
    );
    assert_eq!(compiler.chunk.constants[1], Value::Str(Rc::from("inner")));
}

#[test]
fn assigning_to_an_unknown_name_falls_back_to_a_global_store() {
    let compiler = compile_statement(statement(StatementKind::Assign {
        target: identifier("x"),
        value: string("hi"),
    }));

    assert_eq!(compiler.chunk.code, vec![Op::Constant(0), Op::SetGlobal(1)]);
    assert_eq!(compiler.chunk.constants[0], Value::Str(Rc::from("hi")));
}

#[test]
fn an_assignment_evaluates_its_value_before_naming_its_target() {
    let compiler = compile_statement(block(vec![
        let_decl("x", number(1.0)),
        declaration(DeclarationKind::Statement(Box::new(statement(
            StatementKind::Assign {
                target: identifier("x"),
                value: number(2.0),
            },
        )))),
    ]));

    assert_eq!(
        compiler.chunk.code,
        vec![Op::Constant(0), Op::Constant(1), Op::SetLocal(0), Op::Pop]
    );
}

#[test]
fn an_increment_reads_adds_one_and_writes_back_to_the_same_slot() {
    let compiler = compile_statement(block(vec![
        let_decl("i", number(0.0)),
        declaration(DeclarationKind::Statement(Box::new(statement(
            StatementKind::Increment(identifier("i")),
        )))),
    ]));

    assert_eq!(
        compiler.chunk.code,
        vec![
            Op::Constant(0),
            Op::GetLocal(0),
            Op::Constant(1),
            Op::Add,
            Op::SetLocal(0),
            Op::Pop,
        ]
    );
    assert_eq!(compiler.chunk.constants[1], Value::Number(1.0));
}

#[test]
fn a_decrement_of_a_global_reuses_one_name_constant_for_both_the_read_and_the_write() {
    let compiler = compile_statement(statement(StatementKind::Decrement(identifier("g"))));

    assert_eq!(
        compiler.chunk.code,
        vec![Op::GetGlobal(0), Op::Constant(1), Op::Sub, Op::SetGlobal(0),]
    );
    assert_eq!(compiler.chunk.constants[0], Value::Str(Rc::from("g")));
}

#[test]
fn an_if_without_an_else_jumps_past_the_then_branch() {
    let compiler = compile_statement(statement(StatementKind::If {
        condition: Expression {
            kind: ExpressionKind::LiteralBoolean(true),
            span: span(1),
        },
        then_branch: Box::new(expression_statement(number(1.0))),
        else_branch: None,
    }));

    assert_eq!(
        compiler.chunk.code,
        vec![Op::True, Op::JumpIfFalse(4), Op::Constant(0), Op::Pop]
    );
}

#[test]
fn an_if_with_an_else_skips_the_else_after_running_the_then_branch() {
    let compiler = compile_statement(statement(StatementKind::If {
        condition: Expression {
            kind: ExpressionKind::LiteralBoolean(true),
            span: span(1),
        },
        then_branch: Box::new(expression_statement(number(1.0))),
        else_branch: Some(Box::new(expression_statement(number(2.0)))),
    }));

    assert_eq!(
        compiler.chunk.code,
        vec![
            Op::True,
            Op::JumpIfFalse(5),
            Op::Constant(0),
            Op::Pop,
            Op::Jump(7),
            Op::Constant(1),
            Op::Pop,
        ]
    );
}

#[test]
fn a_while_loop_jumps_back_to_before_its_condition() {
    let compiler = compile_statement(statement(StatementKind::While {
        condition: identifier("c"),
        body: Box::new(expression_statement(number(1.0))),
    }));

    assert_eq!(
        compiler.chunk.code,
        vec![
            Op::GetGlobal(0),
            Op::JumpIfFalse(5),
            Op::Constant(1),
            Op::Pop,
            Op::Loop(0),
        ]
    );
}

#[test]
fn a_for_loop_runs_its_init_once_and_its_step_after_the_body() {
    let compiler = compile_statement(statement(StatementKind::For {
        init: ForInit::Let(Box::new(let_decl("i", number(0.0)))),
        condition: Some(binary(Operator::LessThan, identifier("i"), number(3.0))),
        step: Some(ForStep::Increment(identifier("i"))),
        body: Box::new(expression_statement(identifier("i"))),
    }));

    assert_eq!(
        compiler.chunk.code,
        vec![
            Op::Constant(0),
            Op::GetLocal(0),
            Op::Constant(1),
            Op::Less,
            Op::JumpIfFalse(12),
            Op::GetLocal(0),
            Op::Pop,
            Op::GetLocal(0),
            Op::Constant(2),
            Op::Add,
            Op::SetLocal(0),
            Op::Loop(1),
            Op::Pop,
        ]
    );
}

#[test]
fn a_for_loop_without_a_condition_emits_no_exit_jump() {
    let compiler = compile_statement(statement(StatementKind::For {
        init: ForInit::None,
        condition: None,
        step: None,
        body: Box::new(expression_statement(number(1.0))),
    }));

    assert_eq!(
        compiler.chunk.code,
        vec![Op::Constant(0), Op::Pop, Op::Loop(0)]
    );
}

#[test]
fn a_call_pushes_the_callee_before_its_arguments() {
    let compiler = compile_statement(expression_statement(Expression {
        kind: ExpressionKind::Call {
            callee: "f".to_string(),
            args: vec![number(1.0), number(2.0)],
        },
        span: span(1),
    }));

    assert_eq!(
        compiler.chunk.code,
        vec![
            Op::GetGlobal(0),
            Op::Constant(1),
            Op::Constant(2),
            Op::Call(2),
            Op::Pop,
        ]
    );
    assert_eq!(compiler.chunk.constants[0], Value::Str(Rc::from("f")));
}

#[test]
fn a_function_compiles_into_its_own_chunk_stored_as_a_constant() {
    let compiler = compile_declaration(declaration(DeclarationKind::Function {
        name: "add".to_string(),
        params: vec![
            FunctionParam {
                name: "a".to_string(),
                param_type: Type::Number,
            },
            FunctionParam {
                name: "b".to_string(),
                param_type: Type::Number,
            },
        ],
        return_type: Type::Number,
        body: Box::new(block(vec![declaration(DeclarationKind::Statement(
            Box::new(statement(StatementKind::Return(Some(binary(
                Operator::Plus,
                identifier("a"),
                identifier("b"),
            ))))),
        ))])),
    }));

    assert_eq!(
        compiler.chunk.code,
        vec![Op::Constant(0), Op::DefineGlobal(1)]
    );

    let Value::Function(function) = &compiler.chunk.constants[0] else {
        panic!("expected a function constant");
    };

    assert_eq!(*function.name, "add".to_string());
    assert_eq!(function.arity, 2);
    assert_eq!(
        function.chunk.code,
        vec![
            Op::GetLocal(0),
            Op::GetLocal(1),
            Op::Add,
            Op::Return,
            Op::Null,
            Op::Return,
        ]
    );
}

#[test]
fn a_function_body_cannot_see_the_enclosing_compilers_locals() {
    let compiler = compile_statement(block(vec![
        let_decl("outer", number(1.0)),
        declaration(DeclarationKind::Function {
            name: "f".to_string(),
            params: Vec::new(),
            return_type: Type::Void,
            body: Box::new(block(vec![declaration(DeclarationKind::Statement(
                Box::new(expression_statement(identifier("outer"))),
            ))])),
        }),
    ]));

    let Value::Function(function) = &compiler.chunk.constants[1] else {
        panic!("expected a function constant");
    };

    assert_eq!(
        function.chunk.code,
        vec![Op::GetGlobal(0), Op::Pop, Op::Null, Op::Return]
    );
}

#[test]
fn every_instruction_carries_the_line_of_the_node_it_came_from() {
    let mut compiler = Compiler::new();
    compiler.compile_statement(Statement {
        kind: StatementKind::ExpressionStatement(Expression {
            kind: ExpressionKind::LiteralNumber(1.0),
            span: span(7),
        }),
        span: span(4),
    });

    assert_eq!(compiler.chunk.lines, vec![7, 4]);
}
