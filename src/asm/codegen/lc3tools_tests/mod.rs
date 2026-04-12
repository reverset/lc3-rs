#[cfg(test)]
pub mod tests {
    use crate::{
        asm::codegen::{Codegen, lc3tools_codegen::Lc3ToolsCodegen},
        asm::parser::Parser,
        asm::tokenizer::Tokenizer,
    };

    #[test]
    fn test_rpn() {
        const EXPECTED: &str = include_str!("rpng.obj");

        let tokenizer = Tokenizer::new(include_str!("rpn.asm"));
        let tokens = tokenizer.tokenize().unwrap();
        let ast = Parser::new(tokens).parse().unwrap();
        let codegen = Lc3ToolsCodegen::new();
        let output = codegen.generate(ast);

        assert_eq!(String::from_utf8(output.bytes).unwrap(), EXPECTED);
    }
}
