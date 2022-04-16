// #![feature(nll)]

use std::hash::Hasher;
use std::mem::size_of;

use typed_arena_nomut::Arena;
pub use crate::largekey_table::LargeKeyTable;
use std::collections::HashMap;

use metrohash::MetroHash128;
pub use crate::point::Point;

#[derive(Copy, Clone)]
struct QuadTreeValue{
    pub lt: u128,
    pub lb: u128,
    pub rt: u128,
    pub rb: u128,
}
// #[derive(Copy, Clone)]
struct QuadTreeNode{
    pub v: QuadTreeValue,
    pub forward_key: u128,
    pub set_count: u64,
}
const NULL_KEY: u128 = 0xcccccccccccccccccccccccccccccccc;
impl QuadTreeValue{
    fn key(&self) -> u128 {
        let mut hasher = MetroHash128::new();
        let res: u128;
        unsafe{
        hasher.write( &std::mem::transmute::<QuadTreeValue, [u8;64]>(*self));
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
struct BlackKeyCache{
    pub black_keys: Vec<u128>
}

impl BlackKeyCache {
    pub fn new() -> BlackKeyCache{
        const BLACK_BASE: u128 = 0;
        BlackKeyCache{
            black_keys: vec![BLACK_BASE],
        }
    }
    pub fn black_key(&mut self, depth:usize) -> u128{
        //cached method of retreiving the black key for a particular tree level
        match self.black_keys.get(depth){
            Some(x)=>*x,
            None=>{
                let prev_key = self.black_key(depth-1);
                let cur_key = QuadTreeValue{
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
}

struct TreeData<'a>{
    pub map: LargeKeyTable<'a, QuadTreeNode>,
    pub black_key_cache: BlackKeyCache,
}

impl<'a> TreeData<'a>{
    fn new() -> TreeData<'a>{
        // 0 for the lower bits, 0 for the upper bits
        const init_size_pow2: u8 = 1;
        TreeData{
            map: LargeKeyTable::new(init_size_pow2),
            black_key_cache: BlackKeyCache::new(),
        }
    }
    fn get_set_count(&self, d: &QuadTreeValue)->u64{
        [d.lt,d.lb,d.rt,d.rb].iter().map(|x|{
            if d.is_raw(){
                (*x as u64).count_ones() as u64
            }
            else{
                self.map.get(*x).unwrap().set_count
            }
        }).sum()
    }
    // fn garbage_collect(&mut self, root: u128){
    //     let next_size = 1 << next_size_log2;
    //     let next_mask = (!(0 as usize)) >> (8*(size_of::<usize>()) - (next_size_log2 as usize) - 1);
    //     let mut next_map = HashMapWPrior{
    //         table: vec![None;next_size],
    //         table_lastaccessed: vec![0;next_size],
    //         black_keys: map.black_keys.clone(),
    //         lookup_mask: next_mask,
    //         table_size_log2: next_size_log2,
    //     };
    //     for item in allocator.iter(){
    //         let table_idx = (item.key_cached as usize) & next_map.lookup_mask;
    //         next_map.table[table_idx] = Some(allocator.alloc(HashNodeData{
    //             v: item.v,
    //             key_cached: item.key_cached,
    //             forward_key: item.forward_key,
    //             next: None,
    //             set_count: item.set_count,
    //         }));
    //     }
    //     next_map
    // }
}

fn point_8x8_loc(p: Point) -> u8{
    (p.y * 8 + (p.x % 8)) as u8
}
fn set_bit(bitidx: u8) -> u64{
    (1 as u64) << bitidx
}
// fn gather_points(points: Vec<Point>) -> HashMap<Point, u128>{
//     let 
// }
// fn gather_vecs(vec_locs: )
// fn find_key<'a>(allocator: &'a Arena<HashNodeData<'a>>)->HashMapWPrior<'a>{
//     let mut v: HashMapWPrior= HashMapWPrior::new();
//     v.table.resize(10, None);
//     let val:&'a HashNodeData<'a> = allocator.alloc(HashNodeData{
//         v: HashNodeItem{
//             lt: 0,
//             lb: 0,
//             rt: 0,
//             rb: 0,
//         },
//         key_cached: 0,
//         forward_key: 0,
//         next: None,
//         set_count: 0,
//     });
//     v.table[0] = Some(val);
//     let val2:&'a HashNodeData<'a> = allocator.alloc(HashNodeData{
//         v: HashNodeItem{
//             lt: 0,
//             lb: 0,
//             rt: 0,
//             rb: 0,
//         },
//         key_cached: 0,
//         forward_key: 0,
//         next: v.table[0],
//         set_count: 0,
//     });
//     v.table[0] = Some(val2);
//     v
// }

// fn find_key_def(){
//     let allocator: Arena<HashNodeData> = Arena::new();
//     let map = find_key(&allocator);
// }
