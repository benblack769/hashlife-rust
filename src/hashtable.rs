#![feature(nll)]

use std::hash::Hasher;
use std::mem::size_of;

use typed_arena_nomut::Arena;

use metrohash::MetroHash128;

#[derive(Copy, Clone)]
struct HashNodeItem{
    pub lt: u128,
    pub lb: u128,
    pub rt: u128,
    pub rb: u128,
}
struct HashNodeData<'a>{
    pub v: HashNodeItem,
    pub key_cached: u128,
    pub forward_key: u128,
    pub next: Option<&'a HashNodeData<'a>>,
    pub set_count: u64,
}
const NULL_KEY: u128 = 0xcccccccccccccccccccccccccccccccc;
impl HashNodeItem{
    fn key(&self) -> u128 {
        let mut hasher = MetroHash128::new();
        let res: u128;
        unsafe{
        hasher.write( &std::mem::transmute::<HashNodeItem, [u8;64]>(*self));
        res = std::mem::transmute::<(u64,u64), u128>( hasher.finish128());
        }
        res
    }
    fn is_null(&self)->bool{
        self.lt != NULL_KEY
    }
    fn is_raw(&self)->bool{
        // is the base, raw data if the top 64 bits are all 0
        // note that this should result in a collision with a real hash
        // with 1/2^64 probability
        (self.lt >> 64) == 0 
    }
}

struct HashMapWPrior<'a>{
    pub table: Vec<Option<&'a HashNodeData<'a>>>,
    pub table_lastaccessed: Vec<u32>,
    pub black_keys: Vec<u128>,
    pub lookup_mask: usize, 
    pub table_size_log2: u8,
}
fn find_in_list<'a>(list: Option<&'a HashNodeData<'a>>, key: u128)->Option<&'a HashNodeData<'a>>{
    match list{
        None=>None,
        Some(x)=> if key == x.key_cached {list} else {find_in_list(x.next, key)}
    }
}

impl<'a>  HashMapWPrior<'a>{
    fn new() ->HashMapWPrior<'a>{
        // 0 for the lower bits, 0 for the upper bits
        const BLACK_BASE: u128 = 0;
        const init_size_pow2: u8 = 1;
        const init_size: usize = 1 << init_size_pow2;
        const init_mask: usize = 0x3;
        HashMapWPrior{
            table: vec![None;init_size],
            table_lastaccessed: vec![0;init_size],
            black_keys: vec![BLACK_BASE],
            lookup_mask: init_mask,
            table_size_log2: init_size_pow2,
        }
    }
    fn black_key(&mut self, i:usize) -> u128{
        //cached method of retreiving the black key for a particular tree level
        match self.black_keys.get(i){
            Some(x)=>*x,
            None=>{
                let prev_key = self.black_key(i-1);
                let cur_key = HashNodeItem{
                    lt: prev_key,
                    lb: prev_key,
                    rt: prev_key,
                    rb: prev_key,
                }.key();
                self.black_keys.push(cur_key);
                cur_key
            },
        }
    }
    fn get(&self, key: u128)->&HashNodeData<'a>{
        let table_loc = (key as usize) & self.lookup_mask;
        match find_in_list(self.table[table_loc],key){
            None=>panic!("failed to find key in table!"),
            Some(x)=>x,
        }
    }
    fn get_set_count(&self,d: &HashNodeItem)->u64{
        [d.lt,d.lb,d.rt,d.rb].iter().map(|x|{
            if d.is_raw(){
                (*x as u64).count_ones() as u64
            }
            else{
                self.get(*x).set_count
            }
        }).sum()
    }
    fn add(&mut self, item: &'a HashNodeData<'a>) -> u128{
        let key = item.key_cached;
        let table_idx = (key as usize) & self.lookup_mask;
        self.table[table_idx] = Some(item);
        self.table_lastaccessed[table_idx] = 0;
        key
    }
    fn realloc<'b>(map: &'a mut HashMapWPrior<'a>, allocator: &'b Arena<HashNodeData<'b>>, next_size_log2:u8) -> HashMapWPrior<'b>{
        let next_size = 1 << next_size_log2;
        let next_mask = (!(0 as usize)) >> (8*(size_of::<usize>()) - (next_size_log2 as usize) - 1);
        let mut next_map = HashMapWPrior{
            table: vec![None;next_size],
            table_lastaccessed: vec![0;next_size],
            black_keys: map.black_keys.clone(),
            lookup_mask: next_mask,
            table_size_log2: next_size_log2,
        };
        for item in allocator.iter(){
            let table_idx = (item.key_cached as usize) & next_map.lookup_mask;
            next_map.table[table_idx] = Some(allocator.alloc(HashNodeData{
                v: item.v,
                key_cached: item.key_cached,
                forward_key: item.forward_key,
                next: None,
                set_count: item.set_count,
            }));
        }
        next_map
    }
}
fn find_key<'a>(allocator: &'a Arena<HashNodeData<'a>>)->HashMapWPrior<'a>{
    let mut v: HashMapWPrior= HashMapWPrior::new();
    v.table.resize(10, None);
    let val:&'a HashNodeData<'a> = allocator.alloc(HashNodeData{
        v: HashNodeItem{
            lt: 0,
            lb: 0,
            rt: 0,
            rb: 0,
        },
        key_cached: 0,
        forward_key: 0,
        next: None,
        set_count: 0,
    });
    v.table[0] = Some(val);
    let val2:&'a HashNodeData<'a> = allocator.alloc(HashNodeData{
        v: HashNodeItem{
            lt: 0,
            lb: 0,
            rt: 0,
            rb: 0,
        },
        key_cached: 0,
        forward_key: 0,
        next: v.table[0],
        set_count: 0,
    });
    v.table[0] = Some(val2);
    v
}
