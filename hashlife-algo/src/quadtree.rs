use std::hash::Hasher;

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
    fn is_raw(&self)->bool{
        node_is_raw(self.lt)
    }
}
fn calc_result_bitsize(sums:u64, orig_vals:u64)->u64{
    //can support either 8 bit or 4 bit packing
    let mask = 0x1111111111111111 as u64;
    let bit1set = sums;
    let bit2set = (sums >> 1);
    let bit4set = (sums >> 2);
    let ge3 = bit1set & bit2set;
    let eq4 = bit4set & !bit1set & !bit2set;
    let eq3 = ge3 & !bit4set;
    let res = ((eq4&orig_vals) | eq3) & mask;
    res
}
fn sum_row(row: u64)->u64{
    row + (row<<4) + (row>>4)
}
fn step_forward_automata_16x16(prevmap: &[u64], nextmap: &mut[u64], step_num: usize){
    //masking by this row makes sure that extra bits on end don't get set (not technically inaccurate, just confusing)
    debug_assert!(step_num < 4);
    let rowmask = 0x0111111111111110 as u64;
    let mut s1 = sum_row(prevmap[step_num]);
    let mut s2 = sum_row(prevmap[step_num+1]);
    let mut csum = s1 + s2;
    for y in (1+step_num)..(16-1-step_num){
        let s3 = sum_row(prevmap[y+1]);
        csum += s3;
        let row_result = calc_result_bitsize(csum,prevmap[y]);
        nextmap[y] = row_result & rowmask;
        csum -= s1;
        s1 = s2;
        s2 = s3;
    }
}

const fn bits_to_4bit(x:u16)->u64{
    let q16 = x as u64;
    let q8 = (q16 | (q16 << 24)) & 0x000000ff000000ff;
    let q4 = (q8 | (q8 << 12))   & 0x000f000f000f000f;
    let q2 = (q4 | (q4 << 6))   &  0x0303030303030303;
    let q1 = (q2 | (q2 << 3))   &  0x1111111111111111;
    q1
}
const MAP_SIZE:usize = 1<<16;
const fn generate_bit_to_4bit_mapping()->[u64;MAP_SIZE]{
    let mut cached_map = [0 as u64;MAP_SIZE];
    let mut i = 0;
    // using a while loop because `for`, `map`, etc,
    // do not work in constant function as of 2021 stable release
    while i < MAP_SIZE{
        cached_map[i] = bits_to_4bit(i as u16);
        i += 1;
    }
    cached_map
}
const BIT4_MAPPING:[u64;MAP_SIZE] = generate_bit_to_4bit_mapping();
fn to_4bit(x: u16) -> u64{
    BIT4_MAPPING[x as usize]
}
fn pack_4bit_to_bits(x:u32)->u8{
    let g1 = x & 0x11111111;
    let g2 = ((g1 >> 3) | g1) & 0x03030303;
    let g4 = ((g2 >> 6) | g2) & 0x000f000f;
    let g8 = ((g4 >> 12) | g4) & 0x0000000ff;
    g8 as u8
}
fn unpack_bytes_from_word(x: u32)->u64{
    let v1 = x as u64;
    let v2 = (v1 | (v1 << 16)) & 0x0000ffff0000ffff;
    let v4 = (v2 | (v2 << 8)) & 0x00ff00ff00ff00ff;
    v4
}
fn interleave_bytes(v1: u32, v2: u32)-> u64{
    unpack_bytes_from_word(v1) | (unpack_bytes_from_word(v2) << 8)
}

fn unpack_to_bit4(d: QuadTreeValue) -> [u64;16]{
    let dataarr = d.to_array().map(|x|x as u64);
    // let interleaved = [
    //     interleave_bytes((dataarr[0] >> 0) as u32, (dataarr[1] >> 0) as u32),
    //     interleave_bytes((dataarr[0] >> 32) as u32, (dataarr[1] >> 32) as u32),
    //     interleave_bytes((dataarr[2] >> 0) as u32, (dataarr[3] >> 0) as u32),
    //     interleave_bytes((dataarr[2] >> 32) as u32, (dataarr[3] >> 32) as u32),
    // ];
    let mut blocked_bit4 = [0 as u64;16];
    for b in 0..2{
        let mut a1 = dataarr[b*2+0];
        let mut a2 = dataarr[b*2+1];
        for y in 0..8 {
            let v = ((a1 & 0x00ff) | (a2 << 8) & 0xff00) as u16;
            blocked_bit4[b*8+y] = to_4bit(v);
            a1 >>= 8;
            a2 >>= 8;
        }
    }
    blocked_bit4
}
fn get_inner_8x8(data: &[u64])->[u32;8]{
    let mut inner_words = [0 as u32; 8];
    for y in 0..8{
        inner_words[y] = (data[y+4] >> 16) as u32;
    }
    inner_words
}
fn pack_finished_bit4(data: [u32;8]) -> u64{
    let packed_inner_blocks = data.map(pack_4bit_to_bits);
    unsafe{std::mem::transmute::<[u8; 8], u64>(packed_inner_blocks)}
}
fn step_forward_raw(d: QuadTreeValue, n_steps: u64) -> u128{
    assert!(n_steps <= 4);
    let mut data1 = unpack_to_bit4(d);
    let mut data2 = [0 as u64;16];
    for step in 0..n_steps{
        if step%2 == 0{
            step_forward_automata_16x16(&data1[..], &mut data2[..], step as usize);
        }
        else{
            step_forward_automata_16x16(&data2[..], &mut data1[..], step as usize);
        }
    }
    let final_data =  if n_steps%2 == 0 {&data1[..]} else {&data2[..]};
     pack_finished_bit4(get_inner_8x8(final_data)) as u128
}
fn transpose_quad(im:&[u128;16])->[u128;16]{
    //transpose 2x2 quads (each of which are 2x2) into a 4x4 grid
    [
        im[0], im[1], im[4], im[5],
        im[2], im[3], im[6], im[7],
        im[8], im[9], im[12],im[13],
        im[10],im[11],im[14],im[15],
    ]
}

fn is_on_4x4_border(i: usize)->bool{
    [
        0, 1, 2, 3,
        4,       7,
        8,       11,
        12,13,14,15,
    ].iter().any(|x|*x == i)
}
fn slice(in_map:&[u128;16], x: usize, y: usize)->[u128;4]{
    [
        in_map[(0+y)*4+0+x], in_map[(0+y)*4+1+x],
        in_map[(1+y)*4+0+x], in_map[(1+y)*4+1+x],
    ]
}
fn rep_bytes(v: u8)->u64{
    let v1 = v as u64;
    let v2 = v1 | (v1 << 32);
    let v4 = v2 | (v2 << 16);
    let v8 = v4 | (v4 << 8);
    v8
}
fn get_gray_mask(d: i64)-> u64{
    let nds = 1<<(d+2);
    let xmask = rep_bytes((1<<nds) - 1);
    let ymask = ((1 as u64)<<(nds*8))- 1;
    let mask = xmask & ymask;
    mask
}
fn get_subchunk(v: u64, d: i64, x: u8, y: u8)->u64{
    let nds = 1<<(d+2);
    let xshift = v >> (nds * x);
    let yshift = xshift >> (nds*8*y);
    get_gray_mask(d) & yshift 
}

pub struct TreeData{
    map: LargeKeyTable<QuadTreeNode>,
    black_keys: Vec<u128>,
    root: u128,
    depth: u64,
    offset: Point,
}


impl TreeData{
    fn new() -> TreeData{
        const INIT_SIZE_POW2: u8 = 1;
        const BLACK_BASE: u128 = 0;
        let mut tree_data = TreeData{
            map: LargeKeyTable::new(INIT_SIZE_POW2,NULL_KEY,NULL_NODE),
            black_keys: vec![BLACK_BASE],
            root: BLACK_BASE,
            depth: 0,
            offset: Point{x:0,y:0},
        };
        // extend the tree so that increase_size() method can be called
        tree_data.root = tree_data.black_key(1);
        tree_data.depth = 1;
        // call increase depth so that the tree is at very least depth 2, useful for proper recursion
        tree_data.increase_depth();
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
        if d.is_raw(){
            d.to_array().iter().map(|x|(*x as u64).count_ones() as u64).sum()
        }
        else{
            d.to_array().iter().map(|x|self.map.get(*x).unwrap().set_count).sum()
        }
    }
    fn step_forward_compute(&mut self,d: QuadTreeValue, depth: u64, n_steps: u64) -> u128{
        if d.is_raw(){
            assert_eq!(depth, 0);
            step_forward_raw(d, n_steps)
        }
        else if self.get_set_count(&d) == 0{
            //if it is black, return a black key
            //TODO: check if there is a better way to do this....
            self.black_key((depth) as usize)
        }
        else{
            assert_ne!(depth, 0);
            self.step_forward_compute_recursive(d, depth, n_steps)
        }
    }

    fn step_forward_rec(&mut self,d: QuadTreeValue, depth: u64, n_steps: u64) -> u128{
        let full_steps = 4<<depth;
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
        if self.map.get(key).is_none(){
            self.map.add(key, QuadTreeNode{
                v: val,
                forward_key: NULL_KEY,
                set_count: self.get_set_count(&val),
            });
        }
        key
    }
    pub fn increase_depth(&mut self){
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
        let magnitude = (8<<(self.depth-2)) as i64;
        self.offset = self.offset + Point{x:-magnitude,y:-magnitude};
    }
    fn is_black(&self, key: u128)->bool{
        key == 0 || self.map.get(key).unwrap().set_count == 0
    }
    pub fn step_forward(&mut self, n_steps: u64){
        while self.depth < 3{
            self.increase_depth();
        }
        let max_steps = 4 << (self.depth-1);
        let cur_steps = std::cmp::min(max_steps, n_steps);
        let steps_left = n_steps - cur_steps;
        let init_map = self.map.get(self.root).unwrap().v.to_array().map(|x|self.map.get(x).unwrap().v.to_array());
        let arg_map = unsafe{std::mem::transmute::<[[u128;4]; 4], [u128;16]>(init_map)};
        let transposed_map = transpose_quad(&arg_map);
        let has_white_on_border: bool = transposed_map.iter()
            .enumerate()
            .filter(|(i,_)|is_on_4x4_border(*i))
            .any(|(_,key)|!self.is_black(*key));
        if has_white_on_border{
            self.increase_depth();
            self.step_forward(n_steps);
        }
        else{
            self.increase_depth();
            let newkey = self.step_forward_rec(self.map.get(self.root).unwrap().v, self.depth-1, cur_steps);
            self.root = newkey;
            self.depth -= 1;
            let magnitude = (8<<(self.depth-1)) as i64;
            self.offset = self.offset + Point{x:magnitude,y:magnitude};
            if steps_left != 0{
                self.step_forward(steps_left);
            }
        }
    }
    fn step_forward_compute_recursive(&mut self, d: QuadTreeValue, depth: u64, n_steps: u64) -> u128{
        assert!(n_steps <= (4<<depth));
        let init_map = d.to_array().map(|x|self.map.get(x).unwrap().v.to_array());
        let arg_map = unsafe{std::mem::transmute::<[[u128;4]; 4], [u128;16]>(init_map)};
        let mut transposed_map = transpose_quad(&arg_map);
        let finalarr = if n_steps == 0{
            slice(&transposed_map, 1, 1)
        }
        else{
            let next_iter_full_steps = 4<<(depth-1);
            for bt in 0..2{
                let dt = std::cmp::min(next_iter_full_steps as i64,std::cmp::max(0, n_steps as i64-next_iter_full_steps*bt)) as u64;
                let mut result = [NULL_KEY;16];
                for x in 0..(3-bt){
                    for y in 0..(3-bt){
                        let d1 = QuadTreeValue::from_array(&slice(&transposed_map, x as usize,y as usize));
                        result[(y*4+x) as usize] = self.step_forward_rec(d1,depth-1,dt);
                    }
                }
                transposed_map = result;
            }
            slice(&transposed_map, 0, 0)
        };
        let finald = QuadTreeValue::from_array(&finalarr);
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
            if node.forward_key != NULL_KEY && !node_is_raw( node.forward_key){
                TreeData::add_deps_to_tree(orig_table, new_table, node.forward_key);
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
        for oldp in prev_map.keys(){
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
        while cur_map.len() > 1 || depth < 3{
            depth += 1;
            cur_map = tree.gather_points_recurive(&cur_map, depth as usize);
        }
        let magnitude = (8<<(depth-1)) as i64;
        let rootp = *cur_map.keys().next().unwrap();
        tree.root = *cur_map.values().next().unwrap();
        tree.depth = depth;
        tree.offset = rootp.times(magnitude);
        tree
    }
        
    fn iter_grayscale_points<F>(&self, root: u128, depth: i64, cur_loc: Point, fun:&mut F)
    where
        F: FnMut(i64,Point,u64)->bool
    {
        // let area =  (1 as u64)<<(2*(depth+3));
        if depth <= -3{
            let count = (root as u64) & 1;
            fun(depth, cur_loc, count);
        }
        else if depth <= 0{
            assert!(node_is_raw(root));
            let min_depth = -3;
            let val = root as u64;
            let magnitude = 1<<(depth+2);
            // dsize*dsize, but the compiler optimizes the division better
            if fun(depth, cur_loc, val.count_ones() as u64) && depth > min_depth {
                for y in 0..2{
                    for x in 0..2{
                        let offset = Point{x:x, y:y}.times(magnitude); 
                        self.iter_grayscale_points(get_subchunk(val, depth, x as u8, y as u8) as u128, depth-1,cur_loc+offset, fun);
                    }
                }
            }
        }
        else{
            assert!(!node_is_raw(root));
            let magnitude = 1<<(depth+2);
            let subvalue = self.map.get(root).unwrap();
            if fun(depth, cur_loc, subvalue.set_count){
                for (i, subnode) in subvalue.v.to_array().iter().enumerate(){
                    let offset = Point{
                        x:((i%2) as i64),
                        y:((i/2) as i64),
                    }.times(magnitude);
                    self.iter_grayscale_points(*subnode, depth-1,cur_loc+offset, fun);
                }
            }
        }
    }
    pub fn dump_all_points(&self) -> Vec<Point>{
        let mut res: Vec<Point> = Vec::new();
        self.iter_grayscale_points(self.root, self.depth as i64, self.offset, &mut|depth,p,count|{
            if count == 0{
                return false;
            }
            if depth == -3{
                res.push(p);
            }
            return true;
        });
        res
    }
    
    pub fn make_grayscale_map(&self, offset:Point, xsize: usize, ysize: usize, zoom: u8, brightness: f64) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::new();
        res.resize(xsize*ysize, 0);
        const B2: u8 = 16;
        let brightness_int = (brightness * (1<<B2) as f64) as u64;
        self.iter_grayscale_points(self.root, self.depth as i64, offset.neg() + self.offset, &mut|depth,p,count|{
            let magnitude:i64 = (1<<(depth+3));
            let t = p.div(magnitude);
            if count == 0{
                false
            }
            else if t.x >= xsize as i64 || t.y >= ysize as i64 || t.x+1 <= 0 || t.y + 1 <= 0{
                false
            }
            else if zoom as i64 >= depth+3{
                let area_log2 = zoom*2;
                res[(t.y*(xsize as i64)+t.x) as usize] = std::cmp::min(255, (255*brightness_int*count) >> (B2 + area_log2)) as u8;
                false
            }
            else{
                true
            }
        });
        res
    }
}

fn point_8x8_loc(p: Point) -> u8{
    ((p.y % 8)*8 + (p.x % 8)) as u8
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
        let mut out_value_map = [0 as u64; 16];
        step_forward_automata_16x16(&value_map, &mut out_value_map, 0);
        assert_eq!(out_value_map, expected_out);

    }
    #[test]
    fn test_step_forward_glider() {
        let value_map:[u64;16] = [
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000010000000,
            0x0000000001000000,
            0x0000000111000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
        ];
        let expected_out:[u64;16] = [
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000101000000,
            0x0000000011000000,
            0x0000000010000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
        ];
        let mut out_value_map = [0 as u64; 16];
        step_forward_automata_16x16(&value_map, &mut out_value_map, 0);
        assert_eq!(out_value_map, expected_out);

    }
    #[test]
    fn test_get_inner_8() {
        let map16x16:[u64;16] = [
            0x1001110110011101,
            0x1011110110111101,
            0x1000110110001101,
            0x1101110000000000,
            0x1001010000000000,
            0x1011010000000000,
            0x1000110111001101,
            0x1011010000000000,
            0x1011010000000000,
            0x1011010011001110,
            0x1001110100000000,
            0x1001010000000000,
            0x1011010000000000,
            0x1011010000000000,
            0x1011110111001111,
            0x1101110011001101,
        ];
        let expecteded_8x8:[u32;8] = [
            0x01000000,
            0x01000000,
            0x11011100,
            0x01000000,
            0x01000000,
            0x01001100,
            0x11010000,
            0x01000000,
        ];
        assert_eq!(get_inner_8x8(&map16x16[..]), expecteded_8x8);
    }
    #[test]
    fn test_packbits(){
        let maps:[[u32;8];4] = [
            [
                0x01000000,
                0x01000000,
                0x11011100,
                0x01000000,
                0x01000000,
                0x01001100,
                0x11010000,
                0x01000000,
            ],
            [
                0x11010000,
                0x01000000,
                0x11011100,
                0x01000000,
                0x01000000,
                0x01000000,
                0x01001100,
                0x01000000,
            ],
            [
                0x01000000,
                0x11011100,
                0x01000000,
                0x01000000,
                0x01000000,
                0x01001100,
                0x11010000,
                0x01000000,
            ],
            [
                0x01000000,
                0x01000000,
                0x11011100,
                0x01000000,
                0x01000000,
                0x01001100,
                0x01000000,
                0x11010000,
            ]
        ];
        let expected_map: [u64;16] = [
            0x1101000001000000,
            0x0100000001000000,
            0x1101110011011100,
            0x0100000001000000,
            0x0100000001000000,
            0x0100000001001100,
            0x0100110011010000,
            0x0100000001000000,
            0x0100000001000000,
            0x0100000011011100,
            0x1101110001000000,
            0x0100000001000000,
            0x0100000001000000,
            0x0100110001001100,
            0x0100000011010000,
            0x1101000001000000,
        ];
        let bits64 = maps.map(|x|pack_finished_bit4(x));
        let expectedbits64: [u64; 4] = [
            0x40D04C4040DC4040,
            0x404C404040DC40D0,
            0x40D04C404040DC40,
            0xD0404C4040DC4040,
        ];

        assert_eq!(bits64, expectedbits64);
        let value = QuadTreeValue::from_array(&bits64.map(|x| x as u128));
        let unpacked = unpack_to_bit4(value);
        assert_eq!(unpacked, expected_map);
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
        let orig_arr: [u128;16] =  [
            1, 2, 3, 4,
            5, 6, 7, 8,
            9, 10,11,12,
            13,14,15,16
        ];
        let expected: [u128;16] =[
            1, 2, 5, 6,
            3, 4, 7, 8,
            9, 10,13,14,
            11,12,15,16
        ];
        assert_eq!(transpose_quad(&orig_arr), expected);
    }
    #[test]
    fn test_slice(){
        let orig_arr: [u128;16] = [
            1, 2, 5, 6,
            3, 4, 7, 8,
            9, 10,13,14,
            11,12,15,16
        ];
        let expected: [u128;4] = [
            4, 7,
            10, 13,
        ];
        assert_eq!(slice(&orig_arr,1,1), expected);
    }
    #[test]
    fn test_rep_bytes(){
        assert_eq!(rep_bytes(3),0x0303030303030303 as u64);
    }
    #[test]
    fn test_get_graymask(){
        assert_eq!(get_gray_mask(0),  0x000000000f0f0f0f);
        assert_eq!(get_gray_mask(-1), 0x0000000000000303);
        assert_eq!(get_gray_mask(-2), 0x0000000000000001);
    }
    #[test]
    fn test_get_chunk(){
        assert_eq!(get_subchunk(0x5432109876543210, 0, 0, 0),0x06040200);
        assert_eq!(get_subchunk(0x5432109876543210, 0, 0, 1),0x04020008);
        assert_eq!(get_subchunk(0x5432109876543210, 0, 1, 0),0x07050301);
        assert_eq!(get_subchunk(0x5432109876543210, 0, 1, 1),0x05030109);
        assert_eq!(get_subchunk(0x06040201, -1, 0, 0),0x0201);
        assert_eq!(get_subchunk(0x06040201, -1, 0, 1),0x0200);
        assert_eq!(get_subchunk(0x06040201, -1, 1, 0),0x0000);
        assert_eq!(get_subchunk(0x06040201, -1, 1, 1),0x0101);
    }

}
