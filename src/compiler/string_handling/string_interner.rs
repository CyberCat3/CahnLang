use std::{
    cell::RefCell,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
    rc::Rc,
};

use ahash::AHasher;
use intmap::IntMap;

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
        let hash = {
            let mut hasher = AHasher::default();
            hasher.write(str_to_intern.as_bytes());
            hasher.finish()
        };

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
    // Two atoms are never equal, if they come from different interners
    fn eq(&self, other: &Self) -> bool {
        let ptr_self = self.interner.as_ref() as *const _;
        let ptr_other = other.interner.as_ref() as *const _;

        self.start_index == other.start_index
            && self.end_index == other.end_index
            && ptr_self == ptr_other
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
