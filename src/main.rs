use std::fs;
use std::process::Command;

#[derive(Debug, PartialEq)]
enum Token {
    Number(i32),
    Plus,
    Minus,
    Multiply,
    Divide,
    LeftParen,
    RightParen,
}

#[derive(Debug)]
enum AstNode {
    Number(i32),
    BinaryOp(Box<AstNode>, char, Box<AstNode>),
}

struct Compiler {
    tokens: Vec<Token>,
    ast: Option<AstNode>,
    ir: Vec<String>,
    asm: Vec<String>,
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            tokens: Vec::new(),
            ast: None,
            ir: Vec::new(),
            asm: Vec::new(),
        }
    }

    fn tokenize(&mut self, input: &str) {
        let mut chars = input.chars().peekable();
        while let Some(&c) = chars.peek() {
            match c {
                '0'..='9' => {
                    let mut number = 0;
                    while let Some(&c) = chars.peek() {
                        if c.is_digit(10) {
                            number = number * 10 + c.to_digit(10).unwrap() as i32;
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    self.tokens.push(Token::Number(number));
                }
                '+' => {
                    self.tokens.push(Token::Plus);
                    chars.next();
                }
                '-' => {
                    self.tokens.push(Token::Minus);
                    chars.next();
                }
                '*' => {
                    self.tokens.push(Token::Multiply);
                    chars.next();
                }
                '/' => {
                    self.tokens.push(Token::Divide);
                    chars.next();
                }
                '(' => {
                    self.tokens.push(Token::LeftParen);
                    chars.next();
                }
                ')' => {
                    self.tokens.push(Token::RightParen);
                    chars.next();
                }
                ' ' | '\n' | '\t' => {
                    chars.next();
                }
                _ => panic!("Unexpected token: {}", c),
            }
        }
    }

    fn parse(&mut self) {
        self.ast = Some(self.parse_expression());
    }

    fn parse_expression(&mut self) -> AstNode {
        let mut left = self.parse_term();
        while let Some(token) = self.tokens.first() {
            match token {
                Token::Plus | Token::Minus => {
                    let op = match self.tokens.remove(0) {
                        Token::Plus => '+',
                        Token::Minus => '-',
                        _ => unreachable!(),
                    };
                    let right = self.parse_term();
                    left = AstNode::BinaryOp(Box::new(left), op, Box::new(right));
                }
                _ => break,
            }
        }
        return left;
    }

    fn parse_term(&mut self) -> AstNode {
        let mut left = self.parse_factor();
        while let Some(token) = self.tokens.first() {
            match token {
                Token::Multiply | Token::Divide => {
                    let op = match self.tokens.remove(0) {
                        Token::Multiply => '*',
                        Token::Divide => '/',
                        _ => unreachable!(),
                    };

                    let right = self.parse_factor();
                    left = AstNode::BinaryOp(Box::new(left), op, Box::new(right));
                }
                _ => break,
            }
        }
        return left;
    }

    fn parse_factor(&mut self) -> AstNode {
        match self.tokens.remove(0) {
            Token::Number(n) => AstNode::Number(n),
            Token::LeftParen => {
                let expr = self.parse_expression();
                assert_eq!(self.tokens.remove(0), Token::RightParen);
                expr
            }
            _ => panic!("Unexpected token!"),
        }
    }

    fn generate_ir(&mut self) {
        self.ir.clear();
        if let Some(ast) = self.ast.take() {
            self.ir = Self::generate_ir_node(&ast);
            self.ast = Some(ast);
        }
    }

    //fn generate_ir_node(&mut self, node: &AstNode) -> Vec<String> {
    //    match node {
    //        AstNode::Number(n) => self.ir.push(format!("PUSH {}", n)),
    //        AstNode::BinaryOp(left, op, right) => {
    //            self.generate_ir_node(left);
    //            self.generate_ir_node(right);
    //            match op {
    //                '+' => self.ir.push("ADD".to_string()),
    //                '-' => self.ir.push("SUB".to_string()),
    //                '*' => self.ir.push("MUL".to_string()),
    //                '/' => self.ir.push("DIV".to_string()),
    //                _ => panic!("Unexpected operator"),
    //            }
    //        }
    //    }
    //}

    fn generate_ir_node(node: &AstNode) -> Vec<String> {
        match node {
            AstNode::Number(n) => vec![format!("PUSH {}", n)],
            AstNode::BinaryOp(left, op, right) => {
                let mut ir = Self::generate_ir_node(left);
                ir.extend(Self::generate_ir_node(right));
                ir.push(match op {
                    '+' => "ADD".to_string(),
                    '-' => "SUB".to_string(),
                    '*' => "MUL".to_string(),
                    '/' => "DIV".to_string(),
                    _ => panic!("Unexpected operator"),
                });
                ir
            }
        }
    }

    fn generate_asm(&mut self) {
        self.asm.clear();
        self.asm.push("global _start".to_string());
        self.asm.push("section .text".to_string());
        self.asm.push("_start:".to_string());

        for ir_instruction in &self.ir {
            match ir_instruction.as_str() {
                "ADD" => {
                    self.asm.push("    pop rbx".to_string());
                    self.asm.push("    pop rax".to_string());
                    self.asm.push("    add rax, rbx".to_string());
                    self.asm.push("    push rax".to_string());
                }
                "SUB" => {
                    self.asm.push("    pop rbx".to_string());
                    self.asm.push("    pop rax".to_string());
                    self.asm.push("    sub rax, rbx".to_string());
                    self.asm.push("    push rax".to_string());
                }
                "MUL" => {
                    self.asm.push("    pop rbx".to_string());
                    self.asm.push("    pop rax".to_string());
                    self.asm.push("    imul rbx".to_string());
                    self.asm.push("    push rax".to_string());
                }
                "DIV" => {
                    self.asm.push("    pop rbx".to_string());
                    self.asm.push("    pop rax".to_string());
                    self.asm.push("    xor rdx, rdx".to_string());
                    self.asm.push("    idiv rbx".to_string());
                    self.asm.push("    push rax".to_string());
                }
                _ => {
                    if ir_instruction.starts_with("PUSH") {
                        let num = ir_instruction.split_whitespace().nth(1).unwrap();
                        self.asm.push(format!("    push {}", num));
                    }
                }
            }
        }
        self.asm.push("    pop rdi".to_string());
        self.asm.push("    call print_int".to_string());

        // Exit the program
        self.asm.push("    mov rax, 60".to_string());
        self.asm.push("    xor rdi, rdi".to_string());
        self.asm.push("    syscall".to_string());

        // Add a function to print integers
        self.asm.push("print_int:".to_string());
        self.asm.push("    mov rax, rdi".to_string());
        self.asm.push("    mov rcx, 0".to_string());
        self.asm.push("    mov rbx, 10".to_string());
        self.asm.push("    push 0".to_string());
        self.asm.push("divide_loop:".to_string());
        self.asm.push("    xor rdx, rdx".to_string());
        self.asm.push("    div rbx".to_string());
        self.asm.push("    add dl, '0'".to_string());
        self.asm.push("    dec rsp".to_string());
        self.asm.push("    mov [rsp], dl".to_string());
        self.asm.push("    inc rcx".to_string());
        self.asm.push("    test rax, rax".to_string());
        self.asm.push("    jnz divide_loop".to_string());
        self.asm.push("    mov rax, 1".to_string());
        self.asm.push("    mov rdi, 1".to_string());
        self.asm.push("    mov rsi, rsp".to_string());
        self.asm.push("    mov rdx, rcx".to_string());
        self.asm.push("    syscall".to_string());
        self.asm.push("    add rsp, rcx".to_string());
        self.asm.push("    inc rsp".to_string());
        self.asm.push("    ret".to_string());
    }

    fn compile(&mut self, input: &str) {
        self.tokenize(input);
        self.parse();
        self.generate_ir();
        self.generate_asm();
    }

    fn save_asm(&self, filename: &str) {
        fs::write(filename, self.asm.join("\n")).expect("Unable to write file");
    }
}

fn main() {
    let input = fs::read_to_string("input.xyz").expect("Unable to read file");
    let mut compiler = Compiler::new();
    compiler.compile(&input);
    compiler.save_asm("output.asm");

    Command::new("nasm")
        .args(&["-f", "elf64", "output.asm"])
        .status()
        .expect("Failed to assemble");

    Command::new("ld")
        .args(&["output.o", "-o", "output"])
        .status()
        .expect("Failed to link");

    println!("Compilation complete. Executable 'output' created.")
}
