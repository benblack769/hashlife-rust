
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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
 
    #[test]
    fn test_load_dump_points() {
        let contents = concat!(
            "x = 12, y = 8, rule = B3/S23\n",
            "5bob2o$4bo6bo$3b2o3bo2bo$2obo5b2o$2obo5b2o$3b2o3bo2bo$4bo6bo$5bob2o!\n"
        );
        let expected = concat!(
            "x = 0, y = 0, rule = B3/S23\n",
            "5bob2o$4bo6bo$3b2o3bo2bo$2obo5b2o$2obo5b2o$3b2o3bo2bo$4bo6bo$5bob2o!\n"
        );

        let points = parse_fle_file(contents);
        let mut tree = TreeData::gather_all_points(&points);
        tree.increase_depth();
        tree.increase_depth();
        // tree.step_forward(1);
        let out_points = tree.dump_all_points();
        let rle_tot_str = write_rle(&out_points);
        assert_eq!(expected, rle_tot_str);
    }
    
    #[test]
    fn test_load_dump_points_large() {
        let contents = concat!(
            "x = 12, y = 8, rule = B3/S23\n",
            "12bo8bo$bo2bo2b2o2bo25bo2b2o2bo2bo$6bo5bo7b3o3b3o7bo5bo$6bo5bo8bo5bo8bo5bo$6bo5bo8b7o8bo5bo$bo2bo2b2o2bo2b2o4bo7bo4b2o2bo2b2o2bo2bo$o8bo3b2o4b11o4b2o3bo8bo$o3bo9b2o17b2o9bo3bo$4o11b19o11b4o$16bobo11bobo$19b11o$19bo9bo$20b9o$24bo$20b3o3b3o$22bo3bo$$21b3ob3o$21b3ob3o$20bob2ob2obo$20b3o3b3o$21bo5bo!\n"
        );
        let expected = concat!(
            "x = 0, y = 0, rule = B3/S23\n",
            "12bo8bo$bo2bo2b2o2bo25bo2b2o2bo2bo$6bo5bo7b3o3b3o7bo5bo$6bo5bo8bo5bo8bo5bo$6bo5bo8b7o8bo5bo$bo2bo2b2o2bo2b2o4bo7bo4b2o2bo2b2o2bo2bo$o8bo3b2o4b11o4b2o3bo8bo$o3bo9b2o17b2o9bo3bo$4o11b19o11b4o$16bobo11bobo$19b11o$19bo9bo$20b9o$24bo$20b3o3b3o$22bo3bo$$21b3ob3o$21b3ob3o$20bob2ob2obo$20b3o3b3o$21bo5bo!\n"
        );

        let points = parse_fle_file(contents);
        let mut tree = TreeData::gather_all_points(&points);
        // tree.increase_depth();
        // tree.increase_depth();
        // tree.step_forward(1000);
        let out_points = tree.dump_all_points();
        let orig_p_str = dump_points_to_str(&points);
        let new_p_str = dump_points_to_str(&out_points);
         fs::write("orig_points.txt", orig_p_str)
            .expect("failed to open points.txt file for writing");
        fs::write("new_points.txt", new_p_str)
            .expect("failed to open points.txt file for writing");
    
        let rle_tot_str = write_rle(&out_points);
        assert_eq!(expected, rle_tot_str);
    }
}