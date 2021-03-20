use cahn_lang::execute_source_to_string;

#[test]
fn basic_if_test() {
    let source = "
        let x := 3
        let y := 8

        if x < y {
            print 1000
        }

        if x > y {
            print 3000
        }

        print 2000
    ";

    let output = execute_source_to_string(source, "inline-test".into());
    assert_eq!(output, "1000\n2000\n");
}

#[test]
fn if_else_test() {
    let source = "if true {
    print 1000
    
    if false { print 2000 }
    else { print 3000 }

    print 4000

} else {
    
    print 5000
    
    if true { print 6000 }
    else { print 7000 }

    print 8000

}

print 9000";

    let output = execute_source_to_string(source, "inline-test".into());
    assert_eq!(output, "1000\n3000\n4000\n9000\n");
}
