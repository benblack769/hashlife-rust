/*
Fast storage and lookup table for 
well distributed keys of size u128.

Perfect for lookups where the key
is a hash.
*/

use typed_arena_nomut::Arena;
use std::cell::RefCell;
use std::cell::Ref;


#[derive(Clone,Copy)]
struct HashNodeData<T:Copy>{
    key: u128,
    value: T,
}

pub struct LargeKeyTable<T: Copy>{
    table: Vec<HashNodeData<T>>,
    insert_order: Vec<u128>,
    null_value: T,
    null_key: u128,
    lookup_mask: usize, 
    table_size_log2: u8,
}

impl<T: Copy> LargeKeyTable<T>{
    pub fn new(initial_capacity_log2:u8, null_key: u128, null_value: T) -> LargeKeyTable<T>{
        let next_size = 1 << initial_capacity_log2;
        // twos-compliment masking
        let next_mask = next_size - 1;
        let init_data = HashNodeData{
            key: null_key,
            value: null_value.clone(),
        };
        LargeKeyTable{
            table: vec![init_data;next_size],
            insert_order: Vec::new(),
            null_value: null_value,
            null_key: null_key,
            lookup_mask: next_mask,
            table_size_log2: initial_capacity_log2,
        }
    }
    fn null_entry(&self) -> HashNodeData<T>{
        HashNodeData{
            key: self.null_key,
            value: self.null_value,
        }
    }
    pub fn get(&self, key: u128) -> Option<T>{
        let mut curkey = key as u64;
        let mut entry;
        loop{
            let idx = (curkey as usize) & self.lookup_mask;
            entry = &self.table[idx];
            if key == entry.key{
                return Some(entry.value);
            }
            else if key == self.null_key{
                return None;
            }
            curkey >>= 1;
        }
    }
    fn _add(&mut self, key: u128, value: T){
        let mut smallkey = key as usize;
        let mut entry: &mut HashNodeData<T>;
        loop{
            let idx = smallkey & self.lookup_mask;
            entry = &mut self.table[idx];
            if self.null_key == entry.key{
                break;
            }
            smallkey >>= 1;
        }
        *entry = HashNodeData{
            key: key,
            value: value,
        };
    }
    fn _grow(&mut self){
        let mut new_table: LargeKeyTable<T> = LargeKeyTable::new(self.table_size_log2 + 1,self.null_key, self.null_value);
        for entry in self.table.iter(){
            if entry.key != self.null_key{
                new_table._add(entry.key, entry.value);
            }
        }
        self.table = new_table.table;
        self.insert_order = new_table.insert_order;
        self.lookup_mask = new_table.lookup_mask;
        self.table_size_log2 = new_table.table_size_log2;
    }
    pub fn add(&mut self, key: u128, value: T){
        if self.insert_order.len() > self.table.len()/2{
            self._grow();
        }
        self.insert_order.push(key);
        self._add(key, value);
    }
    // pub fn add(&mut self, key: u128, value: T, alloc: &'a Arena<HashNodeData<T>>){
    //     let table_idx = (key as usize) & self.lookup_mask;
    //     self.table[table_idx] = Some(alloc.alloc(
    //         HashNodeData{
    //             key: key,
    //             next: self.table[table_idx],
    //             value: value,
    //         }
    //     ));
    // } 
    // pub fn grow(&mut self){
    //     let next_size_log2 = self.table_size_log2 + 1;
    //     let next_size = 1 << next_size_log2;
    //     let next_mask = (!(0 as usize)) >> (8*(std::mem::size_of::<usize>()) - (next_size_log2 as usize) - 1);
    //     let new_table:Vec<Option<&'a HashNodeData< T>>> = vec![None;next_size];
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
    // pub fn get(&self, key: u128) -> Option<&'a T>{
    //     match find_in_list(self.table[self.idx(key)], key){
    //         Some(x)=> Some(&x.value),
    //         None=>None,
    //     }
    // }
}

// pub fn view< T>(alloc:&'a Arena<HashNodeData<T>>) -> Vec<&'a HashNodeData<T>>{
//     let mut vec: Vec<&'a HashNodeData<T>> = Vec::with_capacity(alloc.len());
//     for item in alloc.iter(){
//         vec.push(item);
//     }
//     vec
// }

// impl<T>  DefaultHashMap<T>{
//     pub fn new(next_size_log2:u8) -> DefaultHashMap<T>{
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
//         let mut new_table: Vec<Option<&'a HashNodeData<T>>> = vec![None;next_size];
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
//     // pub fn iter(&self) -> typed_arena::IterMut<HashNodeData<T>>{
//     //     self.alloc.borrow_mut().iter_mut()
//     // }
// }
