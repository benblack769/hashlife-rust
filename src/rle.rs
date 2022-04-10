
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

pub fn write_rle(points:& Vec<Point>) -> String{
    let mut s: String = String::new();
    s.push_str("x = 0, y = 0, rule = B3/S23\n");
    s.push_str(generate_rle_contents(points).as_str());
    s = split_string_to_lines(s, 80);
    return s;
}

fn cdiv(x:i64, y: i64) -> i64{
    (x+y-1)/y
}

pub fn split_string_to_lines(ins: String, spacing:i64) -> String{
    let mut outs = String::new();
    for i in 0..(cdiv(ins.len() as i64,spacing)){
        let starti = i * spacing;
        let endi = cmp::min(starti + spacing, ins.len() as i64);
        let inslice = &ins[(starti as usize)..(endi as usize)];
        outs.push_str(inslice);
    }
    return outs;
}

pub fn parse_fle_file(file_contents: String) -> Vec<Point> {
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
