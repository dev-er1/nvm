// nvm-asm/tests/codegen_tests.rs
//
// Интеграционные тесты кодогенератора.
pub mod codegen_tests {
    mod encoder_tests;
    mod error_tests;
    mod generate_tests;

    use nvm_asm::codegen;
    use nvm_asm::codegen::err::CodegenError;
    use nvm_asm::lexer::Lexer;
    use nvm_asm::parser::Parser;
    use nvm_asm::src::SourceCode;
    use nvm_asm::str_pool::StrPool;
    use nvm_core::isa::instruction::Instruction;
    use nvm_core::isa::operand::{Operand, OperandKind};
    use nvm_core::isa::register::Register;

    // Полный конвейер: лексер + парсер + кодогенератор.
    pub fn codegen(src: &str) -> Result<Vec<Instruction>, CodegenError> {
        let source = SourceCode::new(src.to_string());
        let mut str_pool = StrPool::from_source(&source);
        let mut lexer = Lexer::new(source, &mut str_pool);
        let tokens = lexer.tokenize().to_vec();

        let mut parser = Parser::new(tokens);
        let ast = parser.parse().clone();

        codegen::generate(&ast, &str_pool)
    }

    // Операнд-регистр.
    pub fn reg(n: u8) -> Operand {
        Operand {
            kind: OperandKind::Register(Register(n)),
        }
    }

    // Операнд-immediate.
    pub fn imm(value: u64) -> Operand {
        Operand {
            kind: OperandKind::Immediate(value),
        }
    }

    // Сравнение операндов (типы не реализуют PartialEq).
    fn same_operand(actual: &Operand, expected: &Operand) -> bool {
        match (actual.kind, expected.kind) {
            (OperandKind::Register(ar), OperandKind::Register(er)) => ar.0 == er.0,
            (OperandKind::Immediate(ai), OperandKind::Immediate(ei)) => ai == ei,
            _ => false,
        }
    }

    // Проверяет, что фактический операнд равен ожидаемому.
    pub fn assert_operand_eq(actual: Option<Operand>, expected: Operand) {
        match actual {
            Some(actual) => assert!(
                same_operand(&actual, &expected),
                "expected {expected:?}, got {actual:?}"
            ),
            None => panic!("expected {expected:?}, got None"),
        }
    }

    // Проверяет, что операнд отсутствует.
    pub fn assert_operand_none(actual: Option<Operand>) {
        assert!(actual.is_none(), "expected no operand, got {actual:?}");
    }
}
