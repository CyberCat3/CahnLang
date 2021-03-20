use std::{
    cell::RefCell,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
    rc::Rc,
};

use intmap::IntMap;

use crate::utils::hash_string;

#[derive(Debug)]
pub struct Interner {
    strings: RefCell<IntMap<(usize, usize)>>,
    big_string: RefCell<String>,
}

impl Drop for Interner {
    fn drop(&mut self) {
        println!("interner dropped");
    }
}

#[derive(Debug, Clone)]
pub struct RCInterner(Rc<Interner>);

impl Deref for RCInterner {
    type Target = Interner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RCInterner {
    pub fn new() -> Self {
        RCInterner(Rc::new(Interner {
            strings: RefCell::new(IntMap::new()),
            big_string: RefCell::new(String::new()),
        }))
    }

    pub fn intern<'a, 'b>(&'a self, str_to_intern: &'b str) -> Atom {
        let hash = hash_string(str_to_intern);

        let res = self.strings.borrow().get(hash).map(|(x, y)| (*x, *y));
        match res {
            Some((start_index, end_index)) => Atom::new(start_index, end_index, Rc::clone(&self.0)),

            None => {
                let start_index = self.big_string.borrow().len();
                self.big_string.borrow_mut().push_str(str_to_intern);
                let end_index = self.big_string.borrow().len();

                self.strings
                    .borrow_mut()
                    .insert(hash, (start_index, end_index));
                Atom::new(start_index, end_index, Rc::clone(&self.0))
            }
        }
    }
}

pub struct Atom {
    start_index: usize,
    end_index: usize,
    interner: Rc<Interner>,
}

impl Atom {
    fn new(start_index: usize, end_index: usize, interner: Rc<Interner>) -> Self {
        Atom {
            start_index,
            end_index,
            interner,
        }
    }

    pub fn interner(&self) -> Rc<Interner> {
        Rc::clone(&self.interner)
    }

    pub fn run_on_str<T, F: FnOnce(&str) -> T>(&self, func: F) -> T {
        let string = &self.interner.as_ref().big_string.borrow()[self.start_index..self.end_index];
        func(string)
    }

    pub fn cut(&self, cut_start: usize, cut_end: usize) -> Self {
        if self.start_index + cut_start > self.end_index {
            panic!("can't cut past endindex");
        }
        let new_start = self.start_index + cut_start;

        if self.end_index < cut_end {
            panic!("can't cut before zero");
        }
        if self.end_index - cut_end < new_start {
            panic!("can't cut before startindex");
        }
        let new_end = self.end_index - cut_end;

        let new_str = &self.interner.big_string.borrow()[new_start..new_end];
        let hash = hash_string(new_str);

        if !self.interner.strings.borrow().contains_key(hash) {
            self.interner
                .strings
                .borrow_mut()
                .insert(hash, (new_start, new_end));
        }

        Atom::new(new_start, new_end, self.interner.clone())
    }
}

impl fmt::Debug for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "Atom({}..{}: '{}')",
            self.start_index, self.end_index, self
        ))
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = &self.interner.as_ref().big_string.borrow()[self.start_index..self.end_index];
        f.write_str(string)
    }
}

impl Clone for Atom {
    fn clone(&self) -> Self {
        Self::new(self.start_index, self.end_index, Rc::clone(&self.interner))
    }
}

impl PartialEq for Atom {
    // Two atoms are never equal if they come from different interners
    fn eq(&self, other: &Self) -> bool {
        self.start_index == other.start_index
            && self.end_index == other.end_index
            && std::ptr::eq(self.interner.as_ref(), other.interner.as_ref())
    }
}
impl Eq for Atom {}

impl Hash for Atom {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.start_index);
        state.write_usize(self.end_index);
        state.write_usize(self.interner.as_ref() as *const _ as usize);
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::string_handling::StringInterner;
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

    #[test]
    fn interned_slices() {
        let interner = StringInterner::new();
        let atom = interner.intern("hej med");
        let atom2 = interner.intern("dig");
        let atom3 = atom.cut(0, 4);
        let atom4 = interner.intern("hej");
        println!(
            "Atom: {}\nAtom2: {}\nAtom3: {}\nAtom4: {}",
            atom, atom2, atom3, atom4
        );
        println!("big_string: {:?}", interner.big_string);
        assert_eq!(interner.big_string.borrow().clone(), "hej meddig");
        assert_eq!(atom3, atom4);
    }
}
