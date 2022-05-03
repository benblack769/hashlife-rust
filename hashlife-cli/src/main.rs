
use std::env;

use std::fs;
use hashlife_fast::*;

fn dump_points_to_str(points: &Vec<Point>)->String{
    let mut sorted_points = points.clone();
    let minx = sorted_points.iter().map(|p|p.x).min().unwrap();
    let miny = sorted_points.iter().map(|p|p.y).min().unwrap();
    sorted_points.sort();
    let mut mystr = String::new();
    for p in sorted_points.iter(){
        let pstr = format!("{x}\t{y}\n", x=p.x-minx,y=p.y-miny);
        mystr.push_str(&pstr);
    }
    mystr
}


fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}\n\n", args);
    assert!(args.len() == 4);
    let in_filename = &args[1];
    let n_steps = args[2].parse::<u64>().unwrap();
    let out_filename = &args[3];

    let contents = fs::read_to_string(in_filename).unwrap();
    let points = parse_fle_file(&contents);
    let mut tree = TreeData::gather_all_points(&points);
    println!("finished gathering");
    tree.step_forward(n_steps);
    println!("finished stepping");
    let out_points = tree.dump_all_points();
    println!("finished dumping");
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