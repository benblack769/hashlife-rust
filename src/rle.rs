
use std::cmp;

pub use crate::point::Point;

pub fn iter_coords<F>(boardrow: &str, func: &mut F)
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
pub fn generate_rle_contents(points:& Vec<Point>) -> String{
    let mut s = String::new();
    if points.len() == 0{
        s.push('!');
        s.push('\n');
        return s;
    }
    let minx = points.iter().map(|p|p.x).min().unwrap();
    let mut sorted_points = points.clone();
    sorted_points.sort();
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
        if gap == 1{
            s.push('b');
        }
        else if gap > 1{
            s.push_str(gap.to_string().as_str());
            s.push('b');
        }
        s.push('o');
        x = p.x + 1;
    }
    s.push('!');
    s.push('\n');
    return s;
}
pub fn compress_os(orig_str: &str)->String{
    let mut s = String::new();
    let mut o_count = 0;
    for c in orig_str.chars(){
        if c == 'o'{
            o_count += 1;
        }
        else{
            if o_count == 1{
                s.push('o');
            }
            else if o_count > 1{
                s.push_str(o_count.to_string().as_str());
                s.push('o');
            }
            o_count = 0;

            s.push(c);
        }
    }
    s
}

pub fn write_rle(points:& Vec<Point>) -> String{
    let mut s: String = String::new();
    s.push_str("x = 0, y = 0, rule = B3/S23\n");
    s.push_str(compress_os(generate_rle_contents(points).as_str()).as_str());
    return split_string_to_lines(&s, 80);
}

fn cdiv(x:i64, y: i64) -> i64{
    (x+y-1)/y
}

pub fn split_string_to_lines(ins: &str, spacing:i64) -> String{
    let mut outs = String::new();
    for i in 0..(cdiv(ins.len() as i64,spacing)){
        let starti = i * spacing;
        let endi = cmp::min(starti + spacing, ins.len() as i64);
        let inslice = &ins[(starti as usize)..(endi as usize)];
        outs.push_str(inslice);
    }
    return outs;
}

pub fn parse_fle_file(file_contents: &str) -> Vec<Point> {
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
        let rle_tot_str = write_rle(&points);
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
        let rle_tot_str = write_rle(&points);
        assert_eq!(expected, rle_tot_str);
    }
}