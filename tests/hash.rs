use std::hash::Hasher;

use ahash::AHasher;

fn hash<F>(func: F) -> u64
where
    F: FnOnce(&mut AHasher) -> (),
{
    let mut hasher = AHasher::default();
    func(&mut hasher);
    hasher.finish()
}

#[test]
fn hash_test() {
    let string1 = "hej med ";
    let string2 = "dig Mille";
    let string3 = format!("{}{}", string1, string2);
    println!("string1: {}", string1);
    println!("string2: {}", string2);
    println!("string3: {}", string3);
    println!("string1 hash: {}", hash(|h| h.write(string1.as_bytes())));
    println!("string1 hash: {}", hash(|h| h.write(string1.as_bytes())));
    println!("string2 hash: {}", hash(|h| h.write(string2.as_bytes())));
    println!("string3 hash: {}", hash(|h| h.write(string3.as_bytes())));
    println!(
        "string1 and string2 hash: {}",
        hash(|h| {
            h.write(string1.as_bytes());
            h.write(string2.as_bytes());
        })
    )
}
