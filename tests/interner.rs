use cahn_lang::compiler::string_handling::StringInterner;

#[test]
fn test_interner() {
    let interner = StringInterner::new();
    let atom1 = interner.intern("hej med dig");
    let atom2 = interner.intern("hvordan går det?");
    let atom3 = interner.intern("rigtig fint");
    let atom4 = interner.intern("hej med dig");
    println!("atom1: {:?}", atom1);
    println!("atom2: {:?}", atom2);
    println!("atom3: {:?}", atom3);
    println!("atom4: {:?}", atom4);
    assert_eq!(atom1, atom4);

    let second_interner = StringInterner::new();
    let atom5 = second_interner.intern("wooow");
    let atom6 = second_interner.intern("hej med dig");
    println!("atom5: {:?}", atom5);
    println!("atom6: {:?}", atom6);
    assert_ne!(atom4, atom6);
}
