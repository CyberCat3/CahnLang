use std::{fmt, iter, ptr};

use intmap::IntMap;

use crate::hash_string;

use super::{Value, VM};

#[derive(Debug)]
pub enum HeapValue {
    String(String),
}

#[derive(Debug)]
pub struct HeapValueHeader {
    pub is_marked: bool,
    pub next_heap_val: *mut HeapValueHeader,
    pub payload: HeapValue,
}

impl fmt::Display for HeapValueHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.payload {
            HeapValue::String(string) => f.write_str(&string),
        }
    }
}

#[derive(Debug)]
pub struct MemoryManager {
    heap_vals: *mut HeapValueHeader,

    intern_string_map: IntMap<*mut HeapValueHeader>,

    total_allocs: u32,
    total_deallocs: u32,
}

impl MemoryManager {
    pub fn new() -> Self {
        MemoryManager {
            heap_vals: ptr::null_mut(),
            total_allocs: 0,
            total_deallocs: 0,
            intern_string_map: IntMap::new(),
        }
    }

    pub fn alloc_string<'a, 'b, 'c>(&'a mut self, vm: &'b VM<'c>, string: String) -> Value {
        let string_hash = hash_string(&string);
        let val = match self.intern_string_map.get(string_hash) {
            // if the string is already allocated, return that
            Some(ptr) => Value::Heap(*ptr),

            // else allocate it and put it in the intern map
            None => {
                let ptr = self.alloc(vm, HeapValue::String(string));
                self.intern_string_map.insert(string_hash, ptr);
                Value::Heap(ptr)
            }
        };
        // print!("allocated string, intern map is now: [");
        // self.intern_string_map
        //     .iter()
        //     .for_each(|(hash, heap_string_ptr)| {
        //         print!("({}: {:?}), ", hash, unsafe { &**heap_string_ptr }.payload)
        //     });
        // println!("]");
        val
    }

    fn alloc<'a, 'b, 'c>(&'a mut self, vm: &'b VM<'c>, val: HeapValue) -> *mut HeapValueHeader {
        let heap_val = HeapValueHeader {
            is_marked: false,
            next_heap_val: self.heap_vals,
            payload: val,
        };
        // move to heap
        let val_pointer = Box::into_raw(Box::new(heap_val));
        // set start of linked list
        self.heap_vals = val_pointer;

        self.total_allocs += 1;

        // println!("MemoryManager allocated: {:?}", unsafe { &*val_pointer });

        if self.should_gc() {
            //     println!("=============GC START==========");
            //     println!("Stack:");
            //     vm.stack
            //         .iter()
            //         .for_each(|val| println!("    {}: {:?}", val.fmt(&vm), val));

            let roots = vm
                .stack
                .iter()
                .map(|val| match val {
                    Value::Heap(ptr) => Some(*ptr),
                    _ => None,
                })
                .flatten()
                .chain(iter::once(val_pointer));

            self.gc(roots);
        }
        val_pointer
    }

    fn should_gc(&self) -> bool {
        true
    }

    pub fn gc<T: Iterator<Item = *mut HeapValueHeader>>(&mut self, roots: T) {
        // println!("\nAll Objects:");
        let mut ptr = self.heap_vals;
        unsafe {
            while !ptr.is_null() {
                // (*ptr).is_marked = false;
                // println!("    obj: {:?}", *ptr);
                ptr = (*ptr).next_heap_val;
            }
        }

        // println!("Marking...");
        // let mut mark_count = 0;
        roots.for_each(|root| {
            self.mark(root);
            // mark_count += 1;
        });
        // println!("Total marked: {}", mark_count);
        // println!("Sweeping...");
        // let tdallocs = self.total_deallocs;
        self.sweep();
        // println!("Total swept: {}", self.total_deallocs - tdallocs);
        // println!("=============GC DONE==========");
    }

    fn mark(&mut self, ptr: *mut HeapValueHeader) {
        unsafe {
            // return if we're already marked, so we don't get
            // infinite recursion in case of reference cycles
            if (*ptr).is_marked {
                return;
            }
            (*ptr).is_marked = true;
            // println!("MemoryManager marked: {}", *ptr);
        }
    }

    fn dealloc(&mut self, ptr: *mut HeapValueHeader) {
        let bbox = unsafe { Box::from_raw(ptr) };
        // println!("MemoryManager deallocated: {}", bbox);

        match bbox.payload {
            HeapValue::String(str) => {
                let hash = hash_string(&str);
                let removed_value = self.intern_string_map.remove(hash);
                assert!(
                    removed_value.is_some(),
                    "heap string was deallocated, but wasn't removed from intern table, intern map: {:?}", self.intern_string_map
                );
            }
        }

        self.total_deallocs += 1;
    }

    // deallocates all unmarked heap values from memory.
    // in the docs, heap value and object are used interchangeably.
    fn sweep(&mut self) {
        unsafe {
            // move the heap_vals pointer to the first marked heap value,
            // or, in case every object was swept, set it to null.
            while !self.heap_vals.is_null() && !(*self.heap_vals).is_marked {
                let next = (*self.heap_vals).next_heap_val;
                self.dealloc(self.heap_vals);
                self.heap_vals = next;
            }
            // if there are any objects left.
            if !self.heap_vals.is_null() {
                // this algorithm consists of two pointers:
                // base_ptr points to the last, reachable object.
                // current_ptr points to the object we are currently considering.

                // base ptr is equal to heap_vals, as we just ensured it points to a marked object.
                let mut base_ptr = self.heap_vals;
                // current pointer is simply the next object in the list.
                let mut current_ptr = (*self.heap_vals).next_heap_val;

                // while we haven't reached the of the object linked list
                while !current_ptr.is_null() {
                    // check if we're currently looking at a reachable object
                    if (*current_ptr).is_marked {
                        // if we are, unmark it
                        (*current_ptr).is_marked = false;
                        // and set base_ptr to the current_ptr, as this
                        // is now the last reachable object.
                        // also set current_ptr to the next object in the chain.
                        base_ptr = current_ptr;
                        current_ptr = (*current_ptr).next_heap_val;
                    } else {
                        // if the object is not marked, we save the adress of the unreachable object.
                        let unreachable_heal_val = current_ptr;
                        // then we move current_ptr to the next object in the list.
                        current_ptr = (*current_ptr).next_heap_val;
                        // now we change the adress of next_heap_val on the last reachable object,
                        // so the unreachable object is no longer pointed to, and is thus no
                        // longer part of the list.
                        (*base_ptr).next_heap_val = current_ptr;
                        // at last deallocate the unreachable heap value
                        self.dealloc(unreachable_heal_val);
                    }
                }
            }
        }
    }

    pub fn dealloc_all(&mut self) {
        while !self.heap_vals.is_null() {
            // pointer to the current heap value
            let ptr = self.heap_vals;
            unsafe {
                // set heap val to the next heap value
                self.heap_vals = (*ptr).next_heap_val;
                // free current heap value
                self.dealloc(ptr);
            }
        }
    }
}

impl Drop for MemoryManager {
    fn drop(&mut self) {
        println!(
            "Dropping memory manager, stats: ( total_allocs: {}, total_deallocs: {} )",
            self.total_allocs, self.total_deallocs
        );
        self.dealloc_all();
        println!(
            "Memory manager dropped, stats: ( total_allocs: {}, total_deallocs: {} )",
            self.total_allocs, self.total_deallocs
        );
    }
}
