
use std::env;

use std::cmp::Ordering;
use std::panic;
use std::cmp;
use std::fs;
use std::mem;
use std::assert;
use std::collections;


// fn square(x:i64) -> i64{
//     x*x
// }

#[derive(Copy, Clone,Hash,PartialEq,Eq)]
struct Point{
    pub x:i64,
    pub y:i64,
}

impl Ord for Point{
    fn cmp(&self, other: &Self) -> Ordering{
        match self.y.cmp(&other.y){
            Ordering::Less=>Ordering::Less,
            Ordering::Greater=>Ordering::Greater,
            Ordering::Equal=>self.x.cmp(&other.x)
        }
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}



fn iter_coords<F>(boardrow: &str, func: &mut F)
where
    F: FnMut(i64)
{
    let mut pos: i64 = 0;
    let mut prefixnum: i64 = 0;
    let mut prefixset = false;
    for c in boardrow.chars(){
        if c.is_numeric(){
            prefixnum = prefixnum * 10 + c.to_digit(10).unwrap() as i64;
            prefixset = true;
        }
        else {
            let repeat = if prefixset {prefixnum} else {1};
            if c == 'b'{
                // do nothing, blank
            } else if c == 'o'{
                for i in pos..(pos+repeat){
                    func(i);
                }
            } else if c == '!' {
                break;
            }
            else{
                panic!("RLE file incorrectly formatted, only 'b' and 'o' allowed.")
            }
            pos += repeat;
            prefixset = false;
            prefixnum = 0;
        }
    }
}
fn generate_rle_contents(points:& Vec<Point>) -> String{
    let mut s = String::new();
    if points.len() == 0{
        return s;
    }
    let minx = points.iter().map(|p|p.x).min().unwrap();
    let mut sorted_points = points.clone();
    sorted_points.sort();//(|p1,p2|{p1.y < p2.y || (p1.y == p2.y && p1.x < p2.x)});
    let firstp = points.get(0).unwrap();
    let mut y = firstp.y;
    let mut x = minx;
    for p in sorted_points{
        while y < p.y {
            s.push('$');
            //reset x to the inital x value in that line
            x = minx;
            y += 1;
        }
        let gap = p.x - x;
        if gap == 2{
            s.push('b');
        }
        else if gap > 2{
            s.push_str(gap.to_string().as_str());
            s.push('b');
        }
        s.push('o');
        x = p.x;
    }
    s.push('!');
    s.push('\n');
    return s;
}

fn write_rle(points:& Vec<Point>) -> String{
    let mut s: String = String::new();
    s.push_str("x = 0, y = 0, rule = B3/S23\n");
    s.push_str(generate_rle_contents(points).as_str());
    s = split_string_to_lines(s, 80);
    return s;
}

fn cdiv(x:i64, y: i64) -> i64{
    (x+y-1)/y
}

fn split_string_to_lines(ins: String, spacing:i64) -> String{
    let mut outs = String::new();
    for i in 0..(cdiv(ins.len() as i64,spacing)){
        let starti = i * spacing;
        let endi = cmp::min(starti + spacing, ins.len() as i64);
        let inslice = &ins[(starti as usize)..(endi as usize)];
        outs.push_str(inslice);
    }
    return outs;
}

fn parse_fle_file(file_contents: String) -> Vec<Point> {
    let mut line_iter = file_contents.lines(); 
    // skips comments and metadata
    while let Some(line) = line_iter.next() {
        if !line.starts_with("#C"){
            break;
        }
    }
    let mut points: Vec<Point> = Vec::new();
    // reads in input
    let mut y:i64 = 0;
    while let Some(line) = line_iter.next() {
        for boardline in line.split_terminator('$') {
            iter_coords(boardline, &mut|x|{
                points.push(Point{
                    x:x,
                    y:y,
                });
            });
            y += 1;
        }
    }
    return points;
}

fn life_forward_fn(
    sum:u8,
    curval:u8
) -> u8{
    if sum == 3{
        1
    }
    else if sum == 4{
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
    let mask:u64 = 0x0101010101010101;
    let bit1set = sums & mask;
    let bit2set = (sums >> 1) & mask;
    let bit4set = (sums >> 2) & mask;
    let ge3 = bit1set & bit2set;
    let eq4 = bit4set & !bit1set & !bit2set;
    let eq3 = ge3 & !bit4set;
    let res = (eq4&orig_vals) | eq3;
    res
}

fn step_forward_automata_8x8(prevmap: &[u64;8], nextmap: &mut[u64;8]){
    //masking by this row mask allows for 
    let rowmask = 0x0011111111111100 as u64;
    let summedmap = prevmap.map(|row|row + (row<<8) + (row>>8));
    for y in 1..(8-1){
        let csum = summedmap[y-1] + summedmap[y] + summedmap[y+1];
        let row_result = calc_result_bitsize(csum,prevmap[y]);
        //println!("0x{:016x}    0x{:016x}", csum, row_result);
        nextmap[y] = row_result & rowmask;
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}\n\n", args);
    assert!(args.len() == 3);
    let in_filename = &args[1];
    let out_filename = &args[2];

    let contents = fs::read_to_string(in_filename).unwrap();
    let points = parse_fle_file(contents);
    let rle_tot_str = write_rle(&points);

    fs::write(out_filename, rle_tot_str)
        .expect("failed to open output file for writing");
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_fast_step_forward() {
        let value_map:[u64;8] = [
            0x0100000101010001,
            0x0100010101010001,
            0x0100000001010001,
            0x0101000101010000,
            0x0100000100010000,
            0x0100010100010000,
            0x0100010100010000,
            0x0100010100010000,
        ];
        let mut out_value_map = [0 as u64;8];
        let u8_in_map = unsafe {
            std::mem::transmute::<[u64; 8], [u8;64]>(value_map)
        };
        let mut u8_out_map = [0 as u8;64];
        step_forward_automata_8x8(&value_map,&mut out_value_map);
        step_forward_automata(&u8_in_map,&mut u8_out_map, 8, 8);
        let u8_packed_out = unsafe {
            std::mem::transmute::<[u64; 8], [u8;64]>(out_value_map)
        };
        assert_eq!(u8_packed_out, u8_out_map);

    }
    #[test]
    fn test_cmprison(){
        let sumval: u64 = 0x0304010207050004;
        let curval: u64 = 0x0001000000000001;
        let expected: u64 = 0x0101000000000001;
        let actual = calc_result_bitsize(sumval, curval);
        assert_eq!(actual, expected);
    }
}