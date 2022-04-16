/*
Fast storage and lookup table for 
well distributed keys of size u128.

Perfect for lookups where the key
is a hash.
*/

use typed_arena_nomut::Arena;
use std::cell::RefCell;
use std::cell::Ref;


struct HashNodeData<'a,T>{
    key: u128,
    next: Option<&'a HashNodeData<'a,T>>,
    value: T,
}

pub struct LargeKeyTable<'a, T>{
    table: Vec<Option<&'a HashNodeData<'a, T>>>,
    lookup_mask: usize, 
    table_size_log2: u8,
}
// pub struct DefaultHashMap<'a,T>{
//     map: RefCell<LargeKeyTable<'a,T>>,
//     alloc: Arena<HashNodeData<'a,T>>,
//     n_elements: RefCell<usize>,// cache the size because alloc.len() is slow
// }

fn find_in_list<'a,T>(list: Option<&'a HashNodeData<'a,T>>, key: u128)->Option<&'a HashNodeData<'a,T>>{
    match list{
        None=>None,
        Some(x)=> if key == x.key {list} else {find_in_list(x.next, key)}
    }
}

impl<'a,T>  LargeKeyTable<'a,T>{
    pub fn new(next_size_log2:u8) -> LargeKeyTable<'a,T>{
        let next_size = 1 << next_size_log2;
        let next_mask = (!(0 as usize)) >> (8*(std::mem::size_of::<usize>()) - (next_size_log2 as usize) - 1);
        LargeKeyTable{
            table: vec![None;next_size],
            lookup_mask: next_mask,
            table_size_log2: next_size_log2,
        }
    }
    pub fn add(&mut self, key: u128, value: T, alloc: &'a Arena<HashNodeData<'a,T>>){
        let table_idx = (key as usize) & self.lookup_mask;
        self.table[table_idx] = Some(alloc.alloc(
            HashNodeData{
                key: key,
                next: self.table[table_idx],
                value: value,
            }
        ));
    } 
    // pub fn grow(&mut self){
    //     let next_size_log2 = self.table_size_log2 + 1;
    //     let next_size = 1 << next_size_log2;
    //     let next_mask = (!(0 as usize)) >> (8*(std::mem::size_of::<usize>()) - (next_size_log2 as usize) - 1);
    //     let new_table:Vec<Option<&'a HashNodeData<'a, T>>> = vec![None;next_size];
    //     for table_row in self.table.iter() {
    //         loop{
    //             match table_row{
    //                 None=>{break},
    //                 Some(e)=>{
    //                     let new_loc = (x.key as usize) & next_mask; 
    //                     let next_entry = e.next;
    //                     e.next = new_table[new_loc];
    //                     new_table[new_loc] = Some(e);

    //                 }
    //             }
    //         }
    //     }
    // }
    fn idx(&self, key: u128)->usize{
        (key as usize) & self.lookup_mask
    }
    pub fn get(&self, key: u128) -> Option<&'a T>{
        match find_in_list(self.table[self.idx(key)], key){
            Some(x)=> Some(&x.value),
            None=>None,
        }
    }
}

pub fn view<'a, T>(alloc:&'a Arena<HashNodeData<'a,T>>) -> Vec<&'a HashNodeData<'a,T>>{
    let mut vec: Vec<&'a HashNodeData<'a,T>> = Vec::with_capacity(alloc.len());
    for item in alloc.iter(){
        vec.push(item);
    }
    vec
}

// impl<'a,T>  DefaultHashMap<'a,T>{
//     pub fn new(next_size_log2:u8) -> DefaultHashMap<'a,T>{
//         DefaultHashMap{
//             map: RefCell::new(LargeKeyTable::new(next_size_log2)),
//             alloc: (Arena::new()),
//             n_elements: RefCell::new(0 as usize),
//         }
//     }
//     fn grow(&'a self){
//         let mut map = self.map.borrow_mut();
//         let next_size_log2 = map.table_size_log2+1;
//         let next_size = 1 << next_size_log2;
//         let next_mask = (!(0 as usize)) >> (8*(std::mem::size_of::<usize>()) - (next_size_log2 as usize) - 1);
//         let mut new_table: Vec<Option<&'a HashNodeData<'a,T>>> = vec![None;next_size];
//         for node in self.alloc.iter_mut(){
//             let loc = (node.key as usize) & next_mask;
//             node.next = new_table[loc];
//             new_table[loc] = Some(node);
//         }
//         map.table = new_table;
//         map.table_size_log2 = next_size_log2;
//         map.lookup_mask = next_mask;
//     }
//     pub fn add(&'a mut self, key: u128, value: T){
//         if self.map.borrow().table.len() / 2 < *self.n_elements.borrow(){
//             self.grow();
//         }
//         self.map.borrow_mut().add(key, value, &self.alloc);
//         *self.n_elements.borrow_mut() += 1;
//     }
//     pub fn get(&self, key: u128) -> Option<&'a T>{
//         self.map.borrow().get(key)
//     }
//     // pub fn iter(&self) -> typed_arena::IterMut<HashNodeData<'a,T>>{
//     //     self.alloc.borrow_mut().iter_mut()
//     // }
// }
