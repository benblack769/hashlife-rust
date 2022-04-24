// #![deny(rust_2018_idioms)]
// #![feature(nll)]
// #![feature(const_eval_limit)]
// #![const_eval_limit = "0"]


use std::env;

use std::panic;
use std::cmp;
use std::fs;
use std::mem;
use std::assert;

mod point;
mod rle;
mod quadtree;
mod largekey_table;

pub use crate::point::Point;
pub use crate::quadtree::{TreeData};
pub use crate::rle::*;

fn dump_points_to_str(points: &Vec<Point>)->String{
    let mut mystr = String::new();
    for p in points.iter(){
        let pstr = format!("{x}\t{y}\n", x=p.x,y=p.y);
        mystr.push_str(&pstr);
    }
    mystr
}


fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}\n\n", args);
    assert!(args.len() == 3);
    let in_filename = &args[1];
    let out_filename = &args[2];

    let contents = fs::read_to_string(in_filename).unwrap();
    let points = parse_fle_file(contents);
    let mut tree = TreeData::gather_all_points(&points);
    // tree.step_forward(1);
    let out_points = tree.dump_all_points();
    let rle_tot_str = write_rle(&out_points);

    let orig_p_str = dump_points_to_str(&points);
    let new_p_str = dump_points_to_str(&out_points);
    fs::write("orig_points.txt", orig_p_str)
        .expect("failed to open points.txt file for writing");
    fs::write("new_points.txt", new_p_str)
        .expect("failed to open points.txt file for writing");

    fs::write(out_filename, rle_tot_str)
        .expect("failed to open output file for writing");
}
