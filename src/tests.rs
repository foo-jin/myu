macro_rules! generate_tests {
    () => {
        #[cfg(test)]
        mod generated {
            use super::*;

            const LTS: &str = r#"des (0,14,8)
(0,"tau",1)
(0,"tau",2)
(1,"tau",3)
(1,"tau",4)
(2,"tau",5)
(2,"tau",4)
(3,"b",0)
(3,"a",6)
(4,"tau",7)
(4,"tau",6)
(5,"a",0)
(5,"a",7)
(6,"tau",2)
(7,"b",1)"#;

            fn check_formula(formula: &str, expected: bool) {
                let lts = LTS.parse::<Lts>().unwrap();
                let f = formula.parse::<mc::Formula>().unwrap();
                dbg!(formula);
                let result = eval(&lts, &f);
                dbg!(&result);
                assert_eq!(result.contains(&0), expected)
            }

            #[test]
            fn boolean() {
                check_formula("false", false);
                check_formula("true", true);
                check_formula("(false && true)", false);
                check_formula("(true && false)", false);
                check_formula("(true && true)", true);
                check_formula("(false || true)", true);
                check_formula("(false || false)", false);
                check_formula("(true || false)", true);
                check_formula("(true || true)", true);
            }

            #[test]
            fn modal_operators() {
                check_formula("[tau]true", true);
                check_formula("<tau>[tau]true", true);
                check_formula("[tau]false", false);
                check_formula("<tau>[tau]false", false);
                check_formula("<tau>false", false);
            }

            #[test]
            fn fixpoints() {
                check_formula("nu X. X", true);
                check_formula("mu Y. Y", false);
                check_formula("nu X. mu Y. (X || Y)", true);
                check_formula("nu X. mu Y. (X && Y)", false);
                check_formula("nu X. (X && mu Y. Y)", false);
            }

            #[test]
            fn combined() {
                // all except 3, 5, 7
                check_formula("nu X. (<tau>X && mu Y. (<tau>Y || [a]false))", true);
                check_formula("nu X. <tau>X", true); // all except 3, 5, 7
                check_formula("nu X. mu Y. ( <tau>Y || <a>X)", true); // all except 7
                check_formula("nu X. mu Y. ( (<tau>Y || <a>Y) || <b>X)", true); // everything
                check_formula("mu X. ([tau]X && (<tau>true || <a>true))", false); // only 3, 5
            }
        }
    };
}
