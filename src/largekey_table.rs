/*
Fast storage and lookup table for 
well distributed keys of size u128.

Perfect for lookups where the key
is a hash.
*/


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
            else if entry.key == self.null_key{
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
    // pub fn iter<'a>(&self) -> std::vec::Vec<u128>::Iter{
    //     self.insert_order.iter()
    // }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_hash_insertions(){
        let basekey:u128 = 0x8fab04dd8336fe8b33e4424a0d9e3e97;
        let mut table: LargeKeyTable<i32> = LargeKeyTable::new(3, 0xcc, 0xccccccc);
        const MAX_CHECK: usize = 5;
        for i in 0..MAX_CHECK{
            table.add(basekey+(i*i) as u128, i as i32);
        }
        let mut x = 0;
        for j in 0..MAX_CHECK*MAX_CHECK{
            assert_eq!(table.get(basekey+j as u128).is_some(), x*x == j);
            if x*x == j{
                x += 1;
            }
        }
    }
}