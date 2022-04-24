// #![feature(nll)]

use std::hash::Hasher;
use std::mem::size_of;

use typed_arena_nomut::Arena;
use crate::largekey_table::LargeKeyTable;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use metrohash::MetroHash128;
use crate::point::Point;

#[derive(Copy, Clone)]
pub struct QuadTreeValue{
    lt: u128,
    rt: u128,
    lb: u128,
    rb: u128,
}
impl QuadTreeValue{
    fn to_array(&self)->[u128;4]{
        [self.lt,self.rt,self.lb,self.rb]
    }
    fn from_array(arr: &[u128;4])->QuadTreeValue{
        QuadTreeValue{
            lt: arr[0],
            rt: arr[1],
            lb: arr[2],
            rb: arr[3],
        }
    }
}
#[derive(Copy, Clone)]
struct QuadTreeNode{
    v: QuadTreeValue,
    forward_key: u128,
    set_count: u64,
}
const NULL_KEY: u128 = 0xcccccccccccccccccccccccccccccccc;
const NULL_VALUE: QuadTreeValue = QuadTreeValue{
    lt: NULL_KEY,
    rt: NULL_KEY,
    lb: NULL_KEY,
    rb: NULL_KEY,
};
const NULL_NODE: QuadTreeNode = QuadTreeNode{
    v: NULL_VALUE,
    forward_key: NULL_KEY,
    set_count: 0xcccccccccccccccc,
};
fn node_is_raw(x:u128)->bool{
    // is the base, raw data if the top 64 bits are all 0
    // note that this should result in a collision with a real hash
    // with 1/2^64 probability
    (x >> 64) == 0 
}
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
        node_is_raw(self.lt)
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
    let q8 = x as u32;
    let q4 = (q8 | (q8 << 12)) & 0x000f000f;
    let q2 = (q4 | (q4 << 6)) &  0x03030303;
    let q1 = (q2 | (q2 << 3)) &  0x11111111;
    q1
    // let mut result: u32 = 0;
    // let mut v: u8 = x;
    // let mut idx = 0;
    // // `for` does not work in constant function as of 2021 stable release
    // while idx < 8 {
    //     result <<= 4;
    //     result |= (v >> 7) as u32;
    //     v <<= 1;
    //     idx+= 1;
    // }
    // result
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
fn to_4bit(x: u8) -> u32{
    bits_to_4bit(x)//bit4_mapping[x as usize]
}
fn pack_4bit_to_bits(x:u32)->u8{
    let g1 = x & 0x11111111;
    let g2 = ((g1 >> 3) | g1) & 0x03030303;
    let g4 = ((g2 >> 6) | g2) & 0x000f000f;
    let g8 = ((g4 >> 12) | g4) & 0x0000000ff;
    g8 as u8
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
fn step_forward_raw(d: QuadTreeValue, n_steps: u64) -> u128{
    let mut input_data = unpack_to_bit4(d);
    let mut operate_data = [0 as u64;16];
    for i in 0..n_steps{
        step_forward_automata_16x16(&input_data, &mut operate_data);
        input_data = operate_data;
    }
    pack_finished_bit4(input_data) as u128
}
fn transpose_quad(in_map:&[u128;16])->[u128;16]{
    //transpose 2x2 quads (each of which are 2x2) into a 4x4 grid
    let mut transposed_map = [0 as u128;16];
    for i1 in 0..2{
        for i2 in 0..2{
            for i3 in 0..2{
                for i4 in 0..2{
                    transposed_map[i1*8+i3*4+i2*2+i4] = in_map[i1*8+i2*4+i3*2+i4];
                }
            }
        }
    }
    transposed_map
}
fn is_on_4x4_border(i: usize)->bool{
    let x = i%4;
    let y = i/4;
    x == 0 || x == 3 || y == 0 || y == 3
}
fn slice(in_map:&[u128;16], x: usize, y: usize)->[u128;4]{
    let mut res = [0 as u128;4];
    for dy in 0..2{
        for dx in 0..2{
            res[dy*2+dx] = in_map[(dy+y)*4+dx+x];
        }
    }
    res
}
pub struct TreeData{
    map: LargeKeyTable<QuadTreeNode>,
    black_keys: Vec<u128>,
    root: u128,
    depth: u64,
}




impl TreeData{
    fn new() -> TreeData{
        const init_size_pow2: u8 = 1;
        const BLACK_BASE: u128 = 0;
        // const BLACK_L1_VAL: QuadTreeValue = QuadTreeValue{lt:BLACK_BASE, lb:BLACK_BASE, rt: BLACK_BASE,rb:BLACK_BASE};
        let mut tree_data = TreeData{
            map: LargeKeyTable::new(init_size_pow2,NULL_KEY,NULL_NODE),
            black_keys: vec![BLACK_BASE],
            root: BLACK_BASE,
            depth: 0,
        };
        // extend the tree so that increase_size can be called
        tree_data.root = tree_data.black_key(1);
        tree_data.depth = 1;
        tree_data
    }
    fn black_key(&mut self, depth:usize) -> u128{
        //cached method of retreiving the black key for a particular tree level
        match self.black_keys.get(depth){
            Some(x)=>*x,
            None=>{
                let prev_key = self.black_key(depth-1);
                let cur_value = QuadTreeValue::from_array(&[prev_key;4]);
                let cur_key = cur_value.key();
                self.map.add(cur_key, QuadTreeNode{
                    v: cur_value,
                    forward_key: prev_key,
                    set_count: 0,
                });
                self.black_keys.push(cur_key);
                cur_key
            },
        }
    }
    fn get_set_count(&self, d: &QuadTreeValue)->u64{
        d.to_array().iter().map(|x|{
            if d.is_raw(){
                (*x as u64).count_ones() as u64
            }
            else{
                self.map.get(*x).unwrap().set_count
            }
        }).sum()
    }
    fn step_forward_compute(&mut self,d: QuadTreeValue, depth: u64, n_steps: u64) -> u128{
        if d.is_raw(){
            assert_eq!(depth, 1);
            step_forward_raw(d, n_steps)
        }
        else{
            assert_ne!(depth, 1);
            self.step_forward_compute_recursive(d, depth, n_steps)
        }
    }

    fn step_forward_rec(&mut self,d: QuadTreeValue, depth: u64, n_steps: u64) -> u128{
        let full_steps = 2<<depth;
        assert!(n_steps <= full_steps, "num steps requested greater than full step, logic inaccurate");
        let key = d.key();
        let item = self.map.get(key);
        if n_steps == full_steps && item.is_some() && item.unwrap().forward_key != NULL_KEY{
            item.unwrap().forward_key
        }
        else{
            let newkey = self.step_forward_compute(d, depth, n_steps);
            // update the forward_key with the new key
            if item.is_none() || (n_steps == full_steps && item.unwrap().forward_key == NULL_KEY) {
                let set_key = if n_steps == full_steps {newkey} else {NULL_KEY};
                self.map.add(key, QuadTreeNode{
                    v: d,
                    forward_key: set_key,
                    set_count: self.get_set_count(&d),
                });
            }
            newkey
        }
    }
    fn add_array(&mut self, arr: [u128;4])->u128{
        let val = QuadTreeValue::from_array(&arr);
        let key = val.key();
        self.map.add(key, QuadTreeNode{
            v: val,
            forward_key: key,
            set_count: self.get_set_count(&val),
        });
        key
    }
    fn increase_depth(&mut self){
        let l1m = self.map.get(self.root).unwrap().v.to_array();
        let bkeyd1 = self.black_key((self.depth-1) as usize);
        let smap = [
            bkeyd1, bkeyd1, bkeyd1, bkeyd1,
            bkeyd1, l1m[0], l1m[1], bkeyd1, 
            bkeyd1, l1m[2], l1m[3], bkeyd1, 
            bkeyd1, bkeyd1, bkeyd1, bkeyd1,
        ];
        let depth0map = [
            self.add_array(slice(&smap, 0, 0)), self.add_array(slice(&smap, 2, 0)),
            self.add_array(slice(&smap, 0, 2)), self.add_array(slice(&smap, 2, 2)),
        ];
        let newkey = self.add_array(depth0map);
        self.root = newkey;
        self.depth += 1;
    }
    fn is_black(&self, key: u128)->bool{
        key == 0 || self.map.get(key).unwrap().set_count == 0
    }
    pub fn step_forward(&mut self, n_steps: u64){
        while self.depth < 2{
            self.increase_depth();
        }
        let max_steps = 2 << (self.depth);
        let cur_steps = std::cmp::min(max_steps, n_steps);
        let steps_left = n_steps - cur_steps;
        let init_map = self.map.get(self.root).unwrap().v.to_array().map(|x|self.map.get(x).unwrap().v.to_array());
        let arg_map = unsafe{std::mem::transmute::<[[u128;4]; 4], [u128;16]>(init_map)};
        let transposed_map = transpose_quad(&arg_map);
        let black_key_d2 = self.black_key((self.depth-2) as usize);
        let has_white_on_border: bool = transposed_map.iter()
            .enumerate()
            .filter(|(i,key)|is_on_4x4_border(*i))
            .any(|(i,key)|!self.is_black(*key));
        if has_white_on_border{
            self.increase_depth();
            self.step_forward(n_steps);
        }
        else{
            let newkey = self.step_forward_rec(self.map.get(self.root).unwrap().v, self.depth, cur_steps);
            self.root = newkey;
            self.depth -= 1;
            if steps_left != 0{
                self.step_forward(steps_left);
            }
        }
    }
    fn step_forward_compute_recursive(&mut self, d: QuadTreeValue, depth: u64, n_steps: u64) -> u128{
        let init_map = d.to_array().map(|x|self.map.get(x).unwrap().v.to_array());
        let arg_map = unsafe{std::mem::transmute::<[[u128;4]; 4], [u128;16]>(init_map)};
        let mut transposed_map = transpose_quad(&arg_map);
        let next_iter_full_steps = 2<<(depth-1);
        for bt in 0..2{
            let dt = std::cmp::min(next_iter_full_steps as i64,std::cmp::max(0, n_steps as i64-next_iter_full_steps*bt)) as u64;
            let mut result = [0 as u128;16];
            for i in 0..(3-bt){
                for j in 0..(3-bt){
                    let d1 = QuadTreeValue::from_array(&slice(&transposed_map, i as usize, j as usize));
                    result[(i*4+j) as usize] = self.step_forward_rec(d1,depth-1,dt);                    
                }
            }
            transposed_map = result;
        }
        let finald = QuadTreeValue::from_array(&slice(&transposed_map, 0, 0));
        // need to add finald to the table so that downstream users can look up its children
        let finalkey = finald.key();
        if self.map.get(finalkey).is_none(){
            self.map.add(finalkey,QuadTreeNode{
                v: finald,
                forward_key: NULL_KEY,
                set_count: self.get_set_count(&finald),
            });
        }
        finalkey
    }
    fn add_deps_to_tree(orig_table:&LargeKeyTable<QuadTreeNode>, new_table: &mut LargeKeyTable<QuadTreeNode>, root: u128){
        // if not raw value
        if !node_is_raw(root){
            let node = orig_table.get(root).unwrap();
            for newroot in node.v.to_array().iter(){
                TreeData::add_deps_to_tree(orig_table, new_table, *newroot);
            }
            new_table.add(root,node);
        }
    }
    fn garbage_collect(&mut self){
        let mut next_map = LargeKeyTable::new(self.map.table_size_log2,NULL_KEY,NULL_NODE);
        //make sure black keys are in new map
        TreeData::add_deps_to_tree(&self.map, &mut next_map, *self.black_keys.last().unwrap());
        TreeData::add_deps_to_tree(&self.map, &mut next_map, self.root);
        self.map = next_map;
    }
    
    fn gather_points_recurive(&mut self, prev_map: &HashMap<Point, u128>, depth: usize) -> HashMap<Point, u128>{
        let mut map: HashMap<Point, u128> = HashMap::new();
        for (oldp,oldkey) in prev_map.iter(){
            let newp = parent_point(*oldp);
            match map.entry(newp){
                //ignore the occupied case, as it means the value has already been filled
                Entry::Occupied(_)=>{},
                //if the entry is vacant, fill it entirely
                Entry::Vacant(entry)=>{
                    let child_keys = child_points(newp).map(|childp|
                        match prev_map.get(&childp) {
                            None=>self.black_key(depth-1),
                            Some(key)=>*key,
                        }
                    );
                    let value = QuadTreeValue::from_array(&child_keys);
                    let key = value.key();
                    self.map.add(key,QuadTreeNode{
                        v: value,
                        forward_key: NULL_KEY,
                        set_count: self.get_set_count(&value),
                    });
                    entry.insert(key);
                }
            }
        }
        map
    }
        
    pub fn gather_all_points(points: &Vec<Point>)->TreeData{
        let mut cur_map = gather_raw_points(&points);
        let mut tree = TreeData::new();
        let mut depth:u64 = 0;
        while cur_map.len() > 1{
            depth += 1;
            cur_map = tree.gather_points_recurive(&cur_map, depth as usize);
        }
        tree.root = *cur_map.values().next().unwrap();
        tree.depth = depth;
        tree
    }
    
    fn dump_points_recursive(&self, root: u128, depth: u64, cur_loc: Point, cur_points: & mut Vec<Point>){
        if depth == 0{
            assert!(node_is_raw(root));
            let mut cur_v = root as u64;
            for y in 0..8{
                for x in 0..8{
                    if (cur_v & 1) != 0{
                        cur_points.push(cur_loc + Point{x:x,y:y});
                    }
                    cur_v >>= 1;
                }
            }
        }
        else{
            assert!(!node_is_raw(root));
            let magnitude = 8<<(depth-1);
            let subvalue = self.map.get(root).unwrap();
            if subvalue.set_count != 0{
                for (i, subnode) in subvalue.v.to_array().iter().enumerate(){
                    let offset = Point{
                        x:((i%2) as i64) * magnitude,
                        y:((i/2) as i64) * magnitude,
                    };
                    self.dump_points_recursive(*subnode, depth-1, cur_loc + offset, cur_points);
                }
            }
        }
    }
    pub fn dump_all_points(&self) -> Vec<Point>{
        let mut res: Vec<Point> = Vec::new();
        self.dump_points_recursive(self.root, self.depth, Point{x:0,y:0}, &mut res);
        res
    }
}

fn point_8x8_loc(p: Point) -> u8{
    (p.y * 8 + (p.x % 8)) as u8
}
fn set_bit(bitidx: u8) -> u64{
    (1 as u64) << bitidx
}
fn gather_raw_points(points: &Vec<Point>) -> HashMap<Point, u128>{
    let mut map: HashMap<Point, u128> = HashMap::new();
    for p in points.iter(){
        let ploc = Point{x:p.x/8,y:p.y/8};
        *map.entry(ploc).or_insert(0) |= set_bit(point_8x8_loc(*p)) as u128;
    }
    map
}
fn is_raw(x: u128) -> bool {
    (x >> 64) == 0
}
fn parent_point(p:Point) -> Point {
    Point{x:p.x/2,y:p.y/2}
}
fn child_points(p:Point) -> [Point;4] {
    [
        Point{x:p.x*2+0, y:p.y*2+0},
        Point{x:p.x*2+1, y:p.y*2+0},
        Point{x:p.x*2+0, y:p.y*2+1},
        Point{x:p.x*2+1, y:p.y*2+1},
    ]
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
    #[test]
    fn test_bit4_op_back(){
        assert_eq!(pack_4bit_to_bits(0x10100111),0xa7);
    }
    #[test]
    fn test_transpose_quad(){
        let orig_arr: [u128;16] = [1,2,5,6,3,4,7,8,9,10,13,14,11,12,15,16];
        let expected: [u128;16] = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16];
        assert_eq!(transpose_quad(&orig_arr), expected);
    }

}