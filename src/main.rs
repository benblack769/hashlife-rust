// #![deny(rust_2018_idioms)]
// #![feature(nll)]

use std::env;

use std::panic;
use std::cmp;
use std::fs;
use std::mem;
use std::assert;

mod point;
mod rle;
mod hashtable;

pub use crate::point::Point;
pub use crate::rle::*;

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
}