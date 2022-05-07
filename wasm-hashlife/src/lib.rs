mod utils;

use wasm_bindgen::prelude::*;
use hashlife_fast::{TreeData,Point, parse_fle_file, write_rle};
use crate::utils::set_panic_hook;
// // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// // allocator.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct TreeDataWrapper{
    tree: TreeData,
}
fn gray_to_rgba(g:&[u8])->Vec<u8>{
    g.iter().map(|x|[*x,*x,*x,255]).into_iter().flatten().collect()
}
#[wasm_bindgen]
impl TreeDataWrapper{
    pub fn new()->TreeDataWrapper{TreeDataWrapper{tree:TreeData::new()}}
    pub fn step_forward(&mut self, n_steps: u32){self.tree.step_forward(n_steps as u64);}
    pub fn num_live_cells(&mut self)->u64{self.tree.num_live_cells()}
    pub fn hash_count(&mut self)->usize{self.tree.hash_count()}
    pub fn get_rle(&self)->String{write_rle(&self.tree.dump_all_points())}
    pub fn make_from_rle(rle:&str)->TreeDataWrapper{TreeDataWrapper { tree: TreeData::gather_all_points(&parse_fle_file(rle)) }}
    pub fn garbage_collect(&mut self){self.tree.garbage_collect();}
    pub fn make_grayscale_map(&self, xstart:i32,ystart:i32, xsize: u32, ysize: u32, zoom: u8, brightness: f64) -> Vec<u8> {
        gray_to_rgba(&self.tree.make_grayscale_map(Point{x:xstart as i64,y:ystart as i64},xsize as usize,ysize as usize,zoom,brightness)[..])
    }

}
#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm-hashlife!");
}

#[wasm_bindgen]
pub fn set_panic_hook_js() {
    set_panic_hook();
}


#[wasm_bindgen]
pub fn paniky() {
    panic!("Hello, wasm-hashlife!");
}
#[wasm_bindgen]
pub struct ExampleStruct{
    xs: Vec<i32>,
    y: i32,
}
#[wasm_bindgen]
impl ExampleStruct{
    pub fn new()->ExampleStruct{
        ExampleStruct{
            xs:Vec::new(),
            y:0
        }
    }
    pub fn add(&mut self, val:i32){
        self.xs.push(val);
        self.y += val;
    }
    pub fn show(&self){
        alert(&format!("len: {}, sum: {}",self.xs.len(),self.y));
    }
}