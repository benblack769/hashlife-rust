// #![feature(nll)]

use std::hash::Hasher;
use std::mem::size_of;

use typed_arena_nomut::Arena;
pub use crate::largekey_table::LargeKeyTable;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use metrohash::MetroHash128;
pub use crate::point::Point;

#[derive(Copy, Clone)]
struct QuadTreeValue{
    pub lt: u128,
    pub lb: u128,
    pub rt: u128,
    pub rb: u128,
}
#[derive(Copy, Clone)]
struct QuadTreeNode{
    pub v: QuadTreeValue,
    pub forward_key: u128,
    pub set_count: u64,
}
const NULL_KEY: u128 = 0xcccccccccccccccccccccccccccccccc;
const NULL_VALUE: QuadTreeValue = QuadTreeValue{
    lt: NULL_KEY,
    lb: NULL_KEY,
    rt: NULL_KEY,
    rb: NULL_KEY,
};
const NULL_NODE: QuadTreeNode = QuadTreeNode{
    v: NULL_VALUE,
    forward_key: NULL_KEY,
    set_count: 0xcccccccccccccccc,
};
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

fn life_forward_fn(
    sum:u8,
    curval:u8
) -> u8 {
    if sum == 3 {
        1
    }
    else if sum == 4 {
        curval
    }
    else{
        0
    }
}

fn step_forward_automata(prevmap: &[u8], nextmap: &mut [u8], xsize:usize, ysize: usize){
    for y in 1..(ysize-1){
        let ymaps = [
            &prevmap[((y-1)*xsize)..((y+0)*xsize)],
            &prevmap[((y+0)*xsize)..((y+1)*xsize)],
            &prevmap[((y+1)*xsize)..((y+2)*xsize)],
        ];
        //sums the first elements
        for x in 1..(xsize-1){
            let csum:u8 = ymaps.iter().map(|v|{let sr:u8 = v[(x-1)..(x+1+1)].iter().sum();sr}).sum();
            let nextval = life_forward_fn(csum, ymaps[1][x]);
            nextmap[y*xsize+x] = nextval;
        }
    }
}

fn calc_result_bitsize(sums:u64, orig_vals:u64)->u64{
    //can support either 8 bit or 4 bit packing
    let mask = 0x1111111111111111 as u64;
    let bit1set = sums & mask;
    let bit2set = (sums >> 1) & mask;
    let bit4set = (sums >> 2) & mask;
    let ge3 = bit1set & bit2set;
    let eq4 = bit4set & !bit1set & !bit2set;
    let eq3 = ge3 & !bit4set;
    let res = (eq4&orig_vals) | eq3;
    res
}

fn step_forward_automata_16x16(prevmap: &[u64;16], nextmap: &mut[u64;16]){
    //masking by this row mask allows for 
    let rowmask = 0x0111111111111110 as u64;
    let summedmap = prevmap.map(|row|row + (row<<4) + (row>>4));
    for y in 1..(16-1){
        let csum = summedmap[y-1] + summedmap[y] + summedmap[y+1];
        let row_result = calc_result_bitsize(csum,prevmap[y]);
        //println!("0x{:016x}    0x{:016x}", csum, row_result);
        nextmap[y] = row_result & rowmask;
    }
}

const fn bits_to_4bit(x:u8)->u32{
    let mut result: u32 = 0;
    let mut v: u8 = x;
    let mut idx = 0;
    // `for` does not work in constant function as of 2021 stable release
    while idx < 8 {
        result <<= 4;
        result |= (v >> 7) as u32;
        v <<= 1;
        idx+= 1;
    }
    result
}
const fn generate_bit_to_4bit_mapping()->[u32;256]{
    let mut cached_map = [0 as u32;256];
    let mut i = 0;
    // using a while loop because `for`, `map`, etc, 
    // do not work in constant function as of 2021 stable release
    while i < 256{
        cached_map[i] = bits_to_4bit(i as u8);
        i += 1;
    }
    cached_map
}
const bit4_mapping:[u32;256] = generate_bit_to_4bit_mapping();
pub fn to_4bit(x: u8) -> u32{
    bit4_mapping[x as usize]
}
fn pack_4bit_to_bits(x:u32)->u8{
    let mut result: u8 = 0;
    let mut v: u32 = x;
    for i in 0..8{
        result <<= 1;
        result |= (v >> 28) as u8;
        v <<= 4;
    }
    result
}

fn unpack_to_bit4(d: QuadTreeValue) -> [u64;16]{
    let dataarr = [d.lt as u64,d.lb as u64,d.rt as u64,d.rb as u64];
    let dataarr_bytes = unsafe{std::mem::transmute::<[u64; 4], [u8;32]>(dataarr)};
    let mut blocked_bytes = [0 as u8;32];
    for y in 0..16 {
        blocked_bytes[y*2] = dataarr_bytes[y];
        blocked_bytes[y*2+1] = dataarr_bytes[y+16];
    }
    let unpacked_i32s = blocked_bytes.map(to_4bit);
    unsafe{std::mem::transmute::<[u32; 32], [u64;16]>(unpacked_i32s)}
}
fn pack_finished_bit4(data: [u64;16]) -> u64{
    let data_bytes = unsafe{std::mem::transmute::<[u64; 16], [u8;128]>(data)};
    let mut inner_bytes = [0 as u8; 32];
    for y in 0..8{
        inner_bytes[y*4..][0..4].clone_from_slice(&data_bytes[4+y*8..][2..6]);
    }
    let inner_blocks = unsafe{std::mem::transmute::<[u8; 32], [u32;8]>(inner_bytes)};
    let packed_inner_blocks = inner_blocks.map(pack_4bit_to_bits);
    unsafe{std::mem::transmute::<[u8; 8], u64>(packed_inner_blocks)}
}
fn step_forward_raw(d: QuadTreeValue, n_steps: u32) -> u128{
    let mut input_data = unpack_to_bit4(d);
    let mut operate_data = [0 as u64;16];
    for i in 0..n_steps{
        step_forward_automata_16x16(&input_data, &mut operate_data);
        input_data = operate_data;
    }
    pack_finished_bit4(input_data) as u128
}

struct TreeData{
    map: LargeKeyTable<QuadTreeNode>,
    black_key_cache: BlackKeyCache,
}


impl TreeData{
    fn new() -> TreeData{
        const init_size_pow2: u8 = 1;
        TreeData{
            map: LargeKeyTable::new(init_size_pow2,NULL_KEY,NULL_NODE),
            black_key_cache: BlackKeyCache::new(),
        }
    }
    fn get_set_count(&self, d: &QuadTreeValue)->u64{
        [d.lt,d.lb,d.rt,d.rb].iter().map(|x|{
            if d.is_raw(){
                (*x as u64).count_ones() as u64
            }
            else{
                self.map.get(*x).set_count
            }
        }).sum()
    }
    pub fn black_key(&mut self, depth:usize) -> u128{
        self.black_key_cache.black_key(depth)
    }
    pub fn step_forward(&mut self,d: QuadTreeValue, depth: u32, n_steps: u32) -> u128{
        let key = d.key();
        // if n_steps == 
        let item = self.map.get(key);
        {
            Some(e)=>e.forward_key,
            None=> {
                if d.is_raw(){
                    step_forward_raw(d, n_steps)
                }
                else{
                    self.step_forward_recursive(d, n_steps)
                }
            }
        }
    }
    fn step_forward_recursive(&mut self,d: QuadTreeValue, n_steps: u32) -> u128{
        let init_map = [d.lt,d.lb,d.rt,d.rb].map(|x|self.map.get(x).v);
        let key_map = 
    }
    // fn step_forward_raw(d: QuadTreeValue) -> u128{
        
    // }

    // pub fn unpack_raw()
    // pub fn add_data(d: QuadTreeValue) -> u128{
    //     let key = d.key();
    //     if d.is_raw(){
            
    //     }
    // }
    
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
fn gather_raw_points(points: Vec<Point>) -> HashMap<Point, u128>{
    let mut map: HashMap<Point, u128> = HashMap::new();
    for p in points.iter(){
        *map.entry(*p).or_insert(0) |= set_bit(point_8x8_loc(*p)) as u128;
    }
    map
}
fn is_raw(x: u128) -> bool{
    (x >> 64) == 0
}
fn parent_point(p:Point)->Point{
    Point{x:p.x/2,y:p.y/2}
}
fn child_points(p:Point)->[Point;4]{
    [
        Point{x:p.x*2+0, y:p.y*2+0},
        Point{x:p.x*2+0, y:p.y*2+1},
        Point{x:p.x*2+1, y:p.y*2+0},
        Point{x:p.x*2+1, y:p.y*2+1},
    ]
}
fn gather_points_recurive(tree: &mut TreeData, prev_map: HashMap<Point, u128>, depth: usize) -> HashMap<Point, u128>{
    let mut map: HashMap<Point, u128> = HashMap::new();
    for (oldp,oldkey) in prev_map.iter(){
        let newp = parent_point(*oldp);
        match map.entry(newp){
            //ignore the occupied case
            Entry::Occupied(_)=>{},
            //if the entry is vacant, fill it entirely
            Entry::Vacant(entry)=>{
                let child_keys = child_points(newp).map(|childp|
                    match prev_map.get(&childp) {
                        None=>tree.black_key(depth),
                        Some(key)=>*key,
                    }
                );
                let value = QuadTreeValue{
                    lt: child_keys[0],
                    lb: child_keys[1],
                    rt: child_keys[2],
                    rb: child_keys[3],
                };
                // tree
                
            }
        }
    }
    map

}
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



#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_step_forward_16x16() {
        let value_map:[u64;16] = [
            0x1001110110011101,
            0x1011110110111101,
            0x1000110110001101,
            0x1101110000000000,
            0x1001010000000000,
            0x1011010000000000,
            0x1011010000000000,
            0x1011010000000000,
            0x1001110100000000,
            0x1001010000000000,
            0x1011010000000000,
            0x1011010000000000,
            0x1011010011001110,
            0x1011110111001111,
            0x1000110111001101,
            0x1101110011001101,
        ];
        let expected_out:[u64;16] = [
            0x0000000000000000,
            0x0010000000100000,
            0x0000000111000100,
            0x0111000000000000,
            0x0000011000000000,
            0x0000011000000000,
            0x0000011000000000,
            0x0000010000000000,
            0x0000010000000000,
            0x0000010000000000,
            0x0000011000000000,
            0x0000011000000100,
            0x0000010101001000,
            0x0010000000110000,
            0x0000000000110000,
            0x0000000000000000,
        ];
        let mut out_value_map = [0 as u64;16];
        step_forward_automata_16x16(&value_map,&mut out_value_map);
        // step_forward_automata(&u8_in_map,&mut u8_out_map, 8, 8);
        // let u8_packed_out = unsafe {
        //     std::mem::transmute::<[u64; 8], [u8;64]>(out_value_map)
        // };
        assert_eq!(out_value_map, expected_out);

    }
    #[test]
    fn test_cmprison(){
        let sumval: u64 = 0x3412750434127504;
        let curval: u64 = 0x0100000101000001;
        let expected: u64 = 0x1100000111000001;
        let actual = calc_result_bitsize(sumval, curval);
        assert_eq!(actual, expected);
    }
    #[test]
    fn test_bit4_op(){
        assert_eq!(to_4bit(0xa7),0x10100111);
    }
}