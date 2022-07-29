use std::{ops::DerefMut, rc::Rc};

#[derive(Debug, Clone, Copy, std::cmp::PartialEq)]
pub enum Functions {
    VariableAssign,

    FunctionVariableAssign,
    MacroVariableAssign,

    SelfId,
    Include,
    Recursive,
    AccessCall, // the dot operator

    UserFunctionChain,
    UserMacroChain,

    UserMacroCall,
    UserFunctionCall,

    Println,
    Print,
    Readln,
    Flush,
    Clear,
    Getch,

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

    Add,
    Sub,
    Mul,
    Div,

    For,
    Match,
    Break,
    Continue,
    If,

    Let,
    Ret,

    PopStack,

    Dup,

    Random,

    Command,
    Sleep,

    Push,
    Pop,
    Insert,
    Remove,
    Append,

    Exit,
}

// fn manticore_functions() -> Vec<String> {
//     vec![
//         // basic output

//         // program flow

//         "ret".to_string(),

//         "rev".to_string(),
//         "shc".to_string(),
//         "rm".to_string(),
//         "sec".to_string(),
//         // string function

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
    UserFunction(String),
    UserMacro(String),
    // Basix Types
    Integer(i128),
    Float(f64),
    String(String),
    Char(char),
    Symbol(char),
    Bool(bool),
    Block(Rc<Vec<Token>>),
    List(Rc<Vec<Token>>),

    // Empty
    Nothing,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub value: Value,
}

impl Token {
    pub fn precedence(&self) -> usize {
        match self.value {
            Value::Function(Functions::VariableAssign) => 2,
            Value::Function(Functions::And) => 6,
            Value::Function(Functions::Or) => 7,
            Value::Function(Functions::Not) => 8,
            Value::Function(Functions::Equals)
            | Value::Function(Functions::Gtr)
            | Value::Function(Functions::Lss) => 9,
            Value::Function(Functions::Add) | Value::Function(Functions::Sub) => 12,
            Value::Function(Functions::Mul)
            | Value::Function(Functions::Div)
            | Value::Function(Functions::Mod) => 13,
            Value::Function(Functions::Neg) => 14,

            _ => 0,
        }
    }

    pub fn is_left_associative(&self) -> bool {
        match self.value {
            Value::Function(Functions::Neg) => false,
            Value::Function(Functions::Or) => true,
            Value::Function(Functions::And) => true,
            Value::Function(Functions::Not) => true,
            Value::Function(Functions::VariableAssign) => false,
            Value::Function(Functions::Add) | Value::Function(Functions::Sub) => true,
            Value::Function(Functions::Mul)
            | Value::Function(Functions::Div)
            | Value::Function(Functions::Mod) => true,
            _ => true,
        }
    }

    pub fn get_value_as_string(&self) -> String {
        match &self.value {
            Value::Identifier(v) => v.to_string(),
            Value::Function(function) => match function {
                Functions::VariableAssign => "VariableAssign".to_string(),
                Functions::Equals => "Equals".to_string(),
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
                Functions::Include => "Capture".to_string(),
                Functions::UserFunctionChain => "UserFunctionChain".to_string(),
                Functions::Add => "Add".to_string(),
                Functions::Sub => "Sub".to_string(),
                Functions::Mul => "Mul".to_string(),
                Functions::Div => "Div".to_string(),
                Functions::UserFunctionCall => "UserFunctionCall".to_string(),
                Functions::UserMacroChain => "UserMacroChain".to_string(),
                Functions::MacroVariableAssign => "MacroVariableAssign".to_string(),
                Functions::Continue => "Continue".to_string(),
                Functions::Push => "Push".to_string(),
                Functions::Pop => "Pop".to_string(),
                Functions::Insert => "insert".to_string(),
                Functions::Remove => "Remove".to_string(),
                Functions::Append => "Append".to_string(),
                Functions::PopStack => "PopStack".to_string(),
                Functions::Clear => "Clear".to_string(),
                Functions::Getch => "Getch".to_string(),
                Functions::Sleep => "Sleep".to_string(),
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
            Value::UserFunction(v) => format!("UserFunction: {}", &v),
            Value::UserMacro(v) => format!("UserMacro: {}", &v),
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
            Value::UserFunction(_) => "UserFunction".to_string(),
            Value::UserMacro(_) => "UserMacro".to_string(),
        }
    }
}
