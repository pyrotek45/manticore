

#[cfg(test)]
mod tests {

    use crate::lexer;
    use crate::manticorevm;
    use crate::parser;
    
    fn test_vm(input: &str, output: &str) {

        let mut lexer = lexer::Lexer::new_from_string(input);
        lexer.parse();
        let mut vm = manticorevm::ManitcoreVm::new(&[], "");
        let mut parser = parser::Parser::new();


        let shunted = parser.shunt(&lexer.block_stack[0]).clone();

        for i in shunted {
            vm.execute_token(&i);
            if vm.exit_loop {
                break;
            }
        }
        if let Some(results) = vm.execution_stack.pop() {
            assert_eq!(results.value, output);
        } else {
            panic!("didnt get last token");
        }

    }

    #[test]
    fn math_stuff() {
        test_vm("1 + 2", "3");
        test_vm("1 * 2", "2");
        test_vm("1 - 2", "-1");
        test_vm("10 / 2", "5");

        test_vm("1 + 2", "3");
        test_vm("1 + 2 * 4 + 7", "16");
        test_vm("1 - 3 * 25 - 8 + 5 * (4 * 5) - 2", "16");
        test_vm("20 + 20 - 20 * 20 - 20 * 10 / 20 + 3 * 1 / 5", "-369.4");
        test_vm("1 + 1 - 2 * ( 34 - 20 ) * ( 12 + 34) / 4", "-320");

        test_vm("pow(5 5)", "3125");
        test_vm("pow(5 2)", "25");

        test_vm("sqrt(10)", "3.1622777");
        test_vm("sqrt(15)", "3.8729835");

        test_vm("neg(10)", "-10");
        test_vm("neg(15)", "-15");

    }

    #[test]
    fn string_stuff() {
        test_vm(r#" "hello world!" "#,"hello world!");
        test_vm(" \"hello world!\n\"","hello world!\n");
        test_vm(" \"hello world!\t\"","hello world!\t");
        test_vm(" \"\thello world!\t\n\"","\thello world!\t\n");
    }

    #[test]
    fn compare_stuff() {
        test_vm(r#"if equ(1 1) {true} {false};"#,"true");
        test_vm(r#"if equ(1 0) {true} {false};"#,"false");

        test_vm(r#"if gtr(1 1) {true} {false};"#,"false");
        test_vm(r#"if gtr(1 0) {true} {false};"#,"true");
        test_vm(r#"if gtr(0 1) {true} {false};"#,"false");

        test_vm(r#"if lss(1 1) {true} {false};"#,"false");
        test_vm(r#"if lss(1 0) {true} {false};"#,"false");
        test_vm(r#"if lss(0 1) {true} {false};"#,"true");

        test_vm(r#"if equ(1 1) {println("wow")};"#,"break");

    }

    #[test]
    fn logic_stuff() {
        test_vm(r#"if or(true   true) {true} {false};"#,"true");
        test_vm(r#"if or(true  false) {true} {false};"#,"true");
        test_vm(r#"if or(false  true) {true} {false};"#,"true");
        test_vm(r#"if or(false false) {true} {false};"#,"false");

        test_vm(r#"if and(true   true) {true} {false};"#,"true");
        test_vm(r#"if and(true  false) {true} {false};"#,"false");
        test_vm(r#"if and(false  true) {true} {false};"#,"false");
        test_vm(r#"if and(false false) {true} {false};"#,"false");

        test_vm(r#"if not(true)  {true} {false};"#,"false");
        test_vm(r#"if not(false) {true} {false};"#,"true");
    }

    #[test]
    fn simple_function_stuff() {
        test_vm(r#"square:={x:~ x:*x}; square(2)"#,"4");
        test_vm(r#"square:={x:~ x:*x}; square(10)"#,"100");

        test_vm(r#"addone:={x:~ x:+1}; addone(10)"#,"11");
        test_vm(r#"addone:={x:~ x:+1}; addone(29)"#,"30");

        test_vm(r#"neq: = {x: y: ~ not( equ(x: y) )}; neq(1 1)"#,"false");
        test_vm(r#"neq: = {x: y: ~ not( equ(x: y) )}; neq(1 0))"#,"true");

        test_vm(r#"geq: = {x: y: ~ if or(gtr(x: y) equ(x: y)) {true} {false};}; geq(1 0)"#,"true");
        test_vm(r#"geq: = {x: y: ~ if or(gtr(x: y) equ(x: y)) {true} {false};}; geq(1 1)"#,"true");
        test_vm(r#"geq: = {x: y: ~ if or(gtr(x: y) equ(x: y)) {true} {false};}; geq(1 2)"#,"false");

        test_vm(r#"leq: = {x: y: ~ if or(lss(x: y) equ(x: y)) {true} {false};}; leq(1 0)"#,"false");
        test_vm(r#"leq: = {x: y: ~ if or(lss(x: y) equ(x: y)) {true} {false};}; leq(1 1)"#,"true");
        test_vm(r#"leq: = {x: y: ~ if or(lss(x: y) equ(x: y)) {true} {false};}; leq(1 2)"#,"true");

        test_vm(r#"abs: = {x: ~ if lss(x: 0) {neg(x)} {x};}; abs(neg(10))"#,"10");
        test_vm(r#"a: = {x: ~ x}; a(10)"#,"10");

        // make functions from other functions
        test_vm(r#"
        square:={x:~ x:*x};
        newsquare: = square;
        newsquare(10)
         "#,"100");
    }

    #[test]
    fn less_simple_function_stuff() {
        test_vm(r#"
        sum: = {
            list: ~
            sum: = 0;
            for x: list: {
                sum: = (x: + sum);
            };
            sum
        };
        sum([1 2 3])
        "#,"6");
        
        test_vm(r#"
        dog: = {name: = "roxas";};
        dog.name:
        "#,"roxas");

        test_vm(r#"
        person: = {name: ~ self};
        bob: = person("bob");
        bob.name:
        "#,"bob");

        // pass functions as parameters 
        test_vm(r#"
        dothing: = {
            x: y: z: ~
            if equ(x: "one") {y()};
            if equ(x: "two") {z()};
        };
        
        item1: = {"one executed"};
        item2: = {"two executed"};
        test: = "one";
        
        dothing(test: item1: item2)
        "#,"one executed");

        test_vm(r#"
        dothing: = {
            x: y: z: ~
            if equ(x: "one") {y()};
            if equ(x: "two") {z()};
        };
        
        item1: = {"one executed"};
        item2: = {"two executed"};
        test: = "two";
        
        dothing(test: item1: item2)
        "#,"two executed");


        


    }

    #[test]
    fn closure_stuff() {
        // create functions from functions
        test_vm(r#"
        myfunc: = {
            x: ~
            let(
                {y: ~ concat(x:  y:)}
            )
        };
        
        coolifier: = myfunc("cool ");
        
        coolifier("man that cray")
        "#,"cool man that cray");

        test_vm(r#"
        y: = 10;
        addten: = let({
            x: ~
            y: + x:
        });
        
        addten(10)
        "#,"20");

    }

    #[test]
    fn syntax_sugers() {

        test_vm(r#"
        do: = {
            {x: ~ x}
        };
        do.run("next")
        "#,"next");

        test_vm(r#"
        do: = {
            { x: ~ x}
        };
        do.run("wow","next")
        "#,"next");

        test_vm(r#"
        @{1 + 1}
        "#,"2");
        
        test_vm(r#"
        {1 + 1}
        "#,"block");


    }
   
    #[test]
    fn variable_stuff() {
        test_vm(r#"
        x: = "bingo";
        x:
        "#,"bingo");

        test_vm(r#"
        x: = 10;
        x:
        "#,"10");

        test_vm(r#"
        x: = 10;
        y: = 20;
        x: + y:
        "#,"30");

        test_vm(r#"
        x: = {};
        x:
        "#,"block");

        test_vm(r#"
        x: = [];
        x:
        "#,"list");

        test_vm(r#"
        x: = 1.2;
        x:
        "#,"1.2");

        test_vm(r#"
        x: = _;
        x:
        "#,"_");

    }

    #[test]
    fn block_stuff() {


        test_vm(r#"
        {x:=10;}.x:
        "#,"10");

        test_vm(r#"
        @{1 + 1}
        "#,"2");
        
    }

    #[test]
    fn scope_stuff() {
        test_vm(r#"
        y: = 50;
        addone: = {
            x: ~
            y: = 1;
            x: + y
        };
        addone(1)
        "#,"2");

        test_vm(r#"
        y: = 50;
        addone: = {
            x: ~
            y: = 1;
            x: + y
        };
        addone(1)
        y:
        "#,"50");
        
    }



}