use cahn_lang::execute_source_to_string;

#[test]
fn basic_if_test() {
    let source = "
        let x := 3
        let y := 8

        if x < y then
            print 1000
        end

        if x > y then
            print 3000
        end

        print 2000
    ";

    let output = execute_source_to_string(source, "inline-test".into());
    assert_eq!(output, "1000\n2000\n");
}

#[test]
fn if_else_test() {
    let source = "
        if true then
            print 1000
        if false then
            print 2000
        else
            print 3000
        end
        print 4000
    else
        print 5000
        if true then
            print 6000
        else
            print 7000
        end
        print 8000
    end

    print 9000";

    let output = execute_source_to_string(source, "inline-test".into());
    assert_eq!(output, "1000\n3000\n4000\n9000\n");
}
