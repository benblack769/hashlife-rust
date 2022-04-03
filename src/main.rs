
use std::env;

use std::cmp::Ordering;
use std::panic;
use std::cmp;
use std::fs;
use std::assert;

// fn square(x:i64) -> i64{
//     x*x
// }

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

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Point {}



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

fn write_rle(points:&mut Vec<Point>) -> String{
    let mut s: String = String::new();
    if points.len() == 0{
        return s;
    }
    let minx = points.iter().map(|p|p.x).min().unwrap();
    points.sort();//(|p1,p2|{p1.y < p2.y || (p1.y == p2.y && p1.x < p2.x)});
    let firstp = points.get(0).unwrap();
    let mut y = firstp.y;
    let mut x = minx;
    for p in points{
        while y < p.y{
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

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}\n\n", args);
    assert!(args.len() == 3);
    let in_filename = &args[1];
    let out_filename = &args[2];

    let contents = fs::read_to_string(in_filename).unwrap();
    let mut line_iter = contents.lines(); 
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
    let rle_raw_str = write_rle(&mut points);
    let rle_lined_str = split_string_to_lines(rle_raw_str, 80);
    let rle_tot_str = format!("x = 871, y = 854, rule = B3/S23\n{rle_lined_str}");

    // let mut output = fs::File::create(out_filename).expect("failed to open output file for writing");
    // write!(output, "{}", rle_tot_str);
    fs::write(out_filename, rle_tot_str).expect("failed to open output file for writing");


    // println!(contents);
    // let secret_number = rand::thread_rng().gen_range(1..101);

    // println!("The secret number is: {}", secret_number);

    // println!("Please input your guess.");


    // loop{
    //     let mut guess = String::new();
    //     io::stdin()
    //         .read_line(&mut guess)
    //         .expect("Failed to read line");

    //     println!("You guessed: {}", guess);
        
    //     let int_guess = match guess.trim().parse::<i32>() {
    //         Ok(num) => num,
    //         Err(_) => {
    //             println!("Guess is not an integer, try again.");
    //             continue;
    //         },
    //     };
        
    //     match int_guess.cmp(&secret_number) {
    //         Ordering::Less => println!("Too small!"),
    //         Ordering::Greater => println!("Too big!"),
    //         Ordering::Equal =>  {
    //             println!("You win!");
    //             break;
    //         }
    //     }
    // }
}