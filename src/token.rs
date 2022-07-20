use std::rc::Rc;

#[derive(Debug, Clone, Copy, std::cmp::PartialEq)]
pub enum Functions {
    VariableAssign,

    SelfId,
    Capture,
    Recursive,
    AccessCall, // the dot operator
    UserMacroCall,
    UserFunctionCall,
    FunctionVariableAssign,

    Println,
    Print,
    Readln,
    Flush,

    Range,

    And,
    Or,
    Not,

    Equals,
    Gtr,
    Lss,

    Neg,
    Mod,
    Pow,
    Sqrt,

    For,
    Match,
    Break,
    If,

    Let,
    Ret,
    Dup,

    Random,

    Command,

    Exit,
}

// fn manticore_functions() -> Vec<String> {
//     vec![
//         // basic output
//         "print".to_string(),
//         "println".to_string(),
//         "flush".to_string(),
//         // program flow
//         "if".to_string(),
//         // block control
//         "call".to_string(),
//         "@".to_string(),
//         //".".to_string(),
//         "ret".to_string(),
//         "let".to_string(),
//         // stack functions
//         "dup".to_string(),
//         "rev".to_string(),
//         "shc".to_string(),
//         "rm".to_string(),
//         "sec".to_string(),
//         // string function
//         "concat".to_string(),
//         // heap control
//         "set".to_string(),
//         "var".to_string(),
//         "=".to_string(),
//         // basic repl control
//         "exit".to_string(),
//         // math functions
//         "neg".to_string(),
//         "sqrt".to_string(),
//         "pow".to_string(),
//         "mod".to_string(),
//         // list functions
//         "range".to_string(),
//         // loop functions
//         "for".to_string(),
//         "loop".to_string(),
//         "from".to_string(),
//         // url
//         "run_url".to_string(),
//         "store_url".to_string(),
//         "import_url".to_string(),
//         // import
//         "import".to_string(),
//         "store_import".to_string(),
//         // os control
//         "command".to_string(),
//         // vm function
//         "exit".to_string(),
//         // boolean op
//         "and".to_string(),
//         "or".to_string(),
//         "not".to_string(),
//         "equ".to_string(),
//         "gtr".to_string(),
//         "lss".to_string(),
//         // input
//         "readln".to_string(),
//         // random function
//         "randomf".to_string(),
//         "random_int".to_string(),
//         // token
//         "exist".to_string(),
//         // list commands
//         "push".to_string(),
//         "pop".to_string(),
//         "insert".to_string(),
//         "remove".to_string(),
//         "append".to_string(),
//     ]
// }

#[derive(Debug, Clone, std::cmp::PartialEq)]
pub enum Value {
    // Functions
    Identifier(String),
    Function(Functions),
    // Basix Types
    Integer(i128),
    Float(f64),
    String(String),
    Char(char),
    Symbol(char),
    Bool(bool),
    Block(Option<Rc<Vec<Token>>>),
    List(Option<Rc<Vec<Token>>>),

    // Empty
    Nothing,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub id: Option<String>,
    pub value: Value,
}

impl Token {
    pub fn precedence(&self) -> usize {
        match self.value {
            Value::Symbol('+') | Value::Symbol('-') => 4,
            Value::Symbol('/') | Value::Symbol('*') => 5,
            _ => 0,
        }
    }

    pub fn is_left_associative(&self) -> bool {
        match self.value {
            Value::Symbol('+') | Value::Symbol('-') => true,
            Value::Symbol('/') | Value::Symbol('*') => true,
            _ => true,
        }
    }

    pub fn get_value_as_string(&self) -> String {
        match &self.value {
            Value::Identifier(v) => v.to_string(),
            Value::Function(function) => match function {
                Functions::VariableAssign => "VariableAssign".to_string(),
                Functions::Equals => "Equals".to_string(),
                Functions::UserFunctionCall => "UserFunctionCall".to_string(),
                Functions::Println => "Println".to_string(),
                Functions::Range => "Range".to_string(),
                Functions::Mod => "Mod".to_string(),
                Functions::For => "For".to_string(),
                Functions::If => "If".to_string(),
                Functions::Print => "Print".to_string(),
                Functions::Break => "Break".to_string(),
                Functions::FunctionVariableAssign => "FunctionVariableAssign".to_string(),
                Functions::Flush => "Flush".to_string(),
                Functions::And => "And".to_string(),
                Functions::Or => "Or".to_string(),
                Functions::Not => "Not".to_string(),
                Functions::Gtr => "Gtr".to_string(),
                Functions::Lss => "Lss".to_string(),
                Functions::AccessCall => "AccessCall".to_string(),
                Functions::UserMacroCall => "UserMacroCall".to_string(),
                Functions::Readln => "Readln".to_string(),
                Functions::Neg => "Neg".to_string(),
                Functions::Pow => "Pow".to_string(),
                Functions::Sqrt => "Sqrt".to_string(),
                Functions::Match => "Match".to_string(),
                Functions::Let => "Let".to_string(),
                Functions::Ret => "Ret".to_string(),
                Functions::Random => "Random".to_string(),
                Functions::Command => "Command".to_string(),
                Functions::Exit => "Exit".to_string(),
                Functions::SelfId => "Self".to_string(),
                Functions::Recursive => "Recursive".to_string(),
                Functions::Dup => "Dup".to_string(),
                Functions::Capture => "Capture".to_string(),
            },
            Value::Integer(v) => v.to_string(),
            Value::Float(v) => v.to_string(),
            Value::String(v) => v.to_string(),
            Value::Symbol(v) => v.to_string(),
            Value::Bool(v) => v.to_string(),
            Value::Block(_) => "Block".to_string(),
            Value::List(_) => "List".to_string(),
            Value::Nothing => "Nothing".to_string(),
            Value::Char(c) => c.to_string(),
        }
    }

    pub fn get_type_as_string(&self) -> String {
        match &self.value {
            Value::Identifier(_) => "Identifier".to_string(),
            Value::Function(_) => "Function".to_string(),
            Value::Integer(_) => "Integer".to_string(),
            Value::Float(_) => "Float".to_string(),
            Value::String(_) => "String".to_string(),
            Value::Symbol(_) => "Symbol".to_string(),
            Value::Bool(_) => "Bool".to_string(),
            Value::Block(_) => "Block".to_string(),
            Value::List(_) => "List".to_string(),
            Value::Char(_) => "char".to_string(),
            Value::Nothing => "Nothing".to_string(),
        }
    }
}
