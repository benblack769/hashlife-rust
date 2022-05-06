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

#[derive(Clone)]
pub struct LargeKeyTable<T: Copy>{
    table: Vec<HashNodeData<T>>,
    n_elements: usize,
    null_value: T,
    null_key: u128,
    lookup_mask: usize, 
    pub table_size_log2: u8,
}
enum PossibleIdx {
    Found(usize),
    Empty(usize),
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
            n_elements: 0,
            null_value: null_value,
            null_key: null_key,
            lookup_mask: next_mask,
            table_size_log2: initial_capacity_log2,
        }
    }
    pub fn len(&self)->usize{
        self.n_elements
    }
    #[inline]
    fn get_idx(&self, key: u128) -> PossibleIdx{
        let mut curkey = key >> 24;
        //quadratic probing for now
        let mut curoffset: usize = 0;
        loop {
            let idx = ((key as usize) + curoffset) & self.lookup_mask;
            let entry = &self.table[idx];
            if key == entry.key{
                return PossibleIdx::Found(idx);
            }
            else if entry.key == self.null_key{
                return PossibleIdx::Empty(idx);
            }
            curoffset += (curkey as usize) & 0xff;
            curkey >>= 1;
        }
    }
    #[inline]
    pub fn get(&self, key: u128) -> Option<T>{
        match self.get_idx(key){
            PossibleIdx::Found(idx)=>Some(self.table[idx].value),
            PossibleIdx::Empty(_)=>None
        }
    }
    #[inline]
    pub fn get_mut(&mut self, key: u128) -> Option<&mut T>{
        match self.get_idx(key){
            PossibleIdx::Found(idx)=>Some(&mut self.table[idx].value),
            PossibleIdx::Empty(_)=>None
        }
    }
    fn _grow(&mut self){
        let mut new_table: LargeKeyTable<T> = LargeKeyTable::new(self.table_size_log2 + 1,self.null_key, self.null_value);
        for entry in self.table.iter(){
            if entry.key != self.null_key{
                new_table.add(entry.key, entry.value);
            }
        }
        assert!(self.n_elements == new_table.n_elements);
        self.table = new_table.table;
        self.n_elements = new_table.n_elements;
        self.lookup_mask = new_table.lookup_mask;
        self.table_size_log2 = new_table.table_size_log2;
    }
    #[inline]
    pub fn add(&mut self, key: u128, value: T){
        match self.get_idx(key){
            PossibleIdx::Found(idx)=>{
                self.table[idx].value = value;
            },
            PossibleIdx::Empty(idx)=>{
                self.n_elements += 1;
                self.table[idx] = HashNodeData{
                    key: key,
                    value: value,
                };
                if self.n_elements >= self.table.len()/2{
                    self._grow();
                }
            }
        }
    }
    pub fn iter_mut<F>(&mut self, func: &mut F)
    where
        F: FnMut(&u128, &mut T)
    {
        for item in self.table.iter_mut(){
            if item.key != self.null_key{
                func(&item.key, &mut item.value);
            }
        }
    }
    
    pub fn iter<F>(&self, func: &mut F)
    where
        F: FnMut(&u128, &T)
    {
        for item in self.table.iter(){
            if item.key != self.null_key{
                func(&item.key, &item.value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_hash_insertions(){
        let basekey:u128 = 0x8fab04dd8336fe8b33e4424a0d9e3e97;
        let mut table: LargeKeyTable<i32> = LargeKeyTable::new(1, 0xcc, 0xccccccc);
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