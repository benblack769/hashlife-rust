
use std::env;
use png::{Encoder,ColorType};


use std::fs;
use hashlife_fast::{TreeData,Point, parse_fle_file, write_rle};

fn dump_points_to_str(points: &Vec<Point>)->String{
    let mut sorted_points = points.clone();
    sorted_points.sort();
    let mut mystr = String::new();
    for p in sorted_points.iter(){
        let pstr = format!("{x}\t{y}\n", x=p.x,y=p.y);
        mystr.push_str(&pstr);
    }
    mystr
}

fn save_png(fname:&str, xsize: usize, ysize: usize, data: &[u8]){
    let mut f = fs::File::create("foo.png").unwrap();
    let mut enc = Encoder::new(f, xsize as u32, ysize as u32);
    enc.set_color(ColorType::Grayscale);
    let mut writer = enc.write_header().unwrap();
    writer.write_image_data(data);
    writer.finish();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}\n\n", args);
    assert!(args.len() == 4);
    let in_filename = &args[1];
    let n_steps = args[2].parse::<u64>().unwrap();
    // let n_steps = args[2].parse::<u64>().unwrap();
    let out_filename = &args[3];

    let contents = fs::read_to_string(in_filename).unwrap();
    let points = parse_fle_file(&contents);
    let mut tree = TreeData::gather_all_points(&points);
    println!("finished gathering");
    tree.step_forward(n_steps);
    println!("finished stepping");
    let xsize = 400;
    let ysize = 400;
    save_png("arg.png",xsize,ysize,&tree.make_grayscale_map(Point{x:0,y:0}, xsize, ysize, 4, 16.)[..]);
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