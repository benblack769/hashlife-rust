import init, { paniky, set_panic_hook_js, ExampleStruct, TreeDataWrapper } from './wasm_hashlife_bg.js';

async function run() {
    await init();


var filename = "example_spaceship.rle";
const RLE_STR = (
    "x = 12, y = 8, rule = B3/S23\n" +
    "5bob2o$4bo6bo$3b2o3bo2bo$2obo5b2o$2obo5b2o$3b2o3bo2bo$4bo6bo$5bob2o!\n"
);
var tree = TreeDataWrapper.make_from_rle(RLE_STR);
var canvas = document.getElementById("game-of-life-canvas");
var xsize = window.innerWidth- 10;
var ysize = window.innerHeight- 10;
var xstart = 0;
var ystart = 0;
var brightnessSelect = document.getElementById("brightness-select");
var cellCountDisplay = document.getElementById("cell-count");
var hashCountDisplay = document.getElementById("hash-count");
var ageDisplay = document.getElementById("universe-age");
var speedSelect = document.getElementById("speed-select");
var fpsSelect = document.getElementById("fps-select");
var garbageSelect = document.getElementById("garbage-select");
var filedata = RLE_STR;
var xyfilecoord = [12,8];
var inputFileLoader = document.getElementById("rle-file-input");
var resetBoundingButton = document.getElementById("reset-bounding-box")
var downloadButton = document.getElementById("download-rle")
var downloadButton = document.getElementById("download-rle")
var zoom_level = -1;
const myWorker = new Worker("worker.js", {type: "module"});
var last_step_time = 0;

const play = "⏵︎";
const pause = "⏸︎";
var play_pause = document.getElementById("play-pause")

function cellSize(){
    return Math.pow(2, Math.floor((zoom_level < 0 ? -zoom_level : 0)))//cellSizeSelect.value)
}
function zoomLevel(){
    return Math.max(0,Math.floor(zoom_level))
}
function zoomScale(){
    return Math.pow(2, zoomLevel())/cellSize() 
}
function brightness(){
    return Math.pow(2,0.25*brightnessSelect.value)
}
function roundToCell(size){
    return Math.floor(size/cellSize())*cellSize()
}
function render(){
    // console.log(tree.hash_count());
    // console.log(tree.num_live_cells());
    var map = tree.make_grayscale_map(xstart,ystart,xsize/cellSize(), ysize/cellSize(),cellSize(),zoomLevel(),brightness());
    // console.log(map);
    var clamped_data = new Uint8ClampedArray(map);
    // console.log(clamped_data);
    var img_data = new ImageData(clamped_data,roundToCell(xsize),roundToCell(ysize));
    // console.log(img_data);
    canvas.width = xsize;
    canvas.height = ysize;
    const canvasContext = canvas.getContext("2d");
    canvasContext.imageSmoothingEnabled = false;
    canvasContext.putImageData(img_data, 0, 0);
    cellCountDisplay.innerText = tree.num_live_cells();
    hashCountDisplay.innerText = tree.hash_count();
    ageDisplay.innerText = tree.get_age();
}
function clearCanvas(){
    var ctx = canvas.getContext('2d');
    
    // fill the entire canvas with black before drawing the circles
    ctx.fillStyle='black';
    ctx.fillRect(0,0,canvas.width,canvas.height);
}
const renderLoop = () => {
    const desired_interval = 1000/Math.pow(2,fpsSelect.value/2.);
    const cur_step_time = new Date().getTime();
    if (cur_step_time - last_step_time > desired_interval-5){
        myWorker.postMessage({
            type: "step_forward",
            amount: Math.pow(2,speedSelect.value),
        });
        last_step_time = cur_step_time;
        setTimeout(renderLoop, desired_interval);
    }
    else{
        setTimeout(renderLoop, desired_interval - (cur_step_time - last_step_time));
    }
};

function bound_zoom(zoom_level){
    zoom_level = Math.max(zoom_level, -5);
    zoom_level = Math.min(zoom_level, 12);
    return zoom_level;
}
function handle_wheel(event){
    // console.log(event);
    if(event.ctrlKey){
        console.log("zoomed! ",event.deltaY,event);
        var oldscale = zoomScale();
        var cenx = xstart + event.offsetX*oldscale;
        var ceny = ystart + event.offsetY*oldscale;
        zoom_level -= event.deltaY*0.03;
        zoom_level = bound_zoom(zoom_level)
        var newscale = zoomScale();
        xstart = cenx - event.offsetX*newscale;
        ystart = ceny - event.offsetY*newscale;
    }
    else{
        xstart += event.deltaX*zoomScale();
        ystart += event.deltaY*zoomScale();
    }
    render();
    event.stopPropagation();
}
function handleFileUpload() {
    clearCanvas();
    var file = inputFileLoader.files[0];
    filename = file.name;
    const reader = new FileReader();
    reader.onload = function(){
        filedata = reader.result;
        myWorker.postMessage({
            type: "set_rle",
            data: filedata,
        });
        //make sure to keep a local copy at all times
        tree = TreeDataWrapper.make_from_rle(filedata);
        parseBoundingBox()
        resetBoundingBox()
    }
    reader.readAsText(file);
}
function parseBoundingBox(){
    var boundsline = filedata.split('\n').filter((l)=>l[0] != "#")[0]
    const numregex = /\d+/g;
    xyfilecoord = boundsline.match(numregex);
}
function resetBoundingBox(){
    var filex = xyfilecoord[0];
    var filey = xyfilecoord[1];
    console.log(filex);
    console.log(filey);
    xstart = -filex/4;
    ystart = -filey/4;
    var zoomx = Math.log2(filex*2 / canvas.width);
    var zoomy = Math.log2(filey*2 / canvas.height);
    console.log(zoomx);
    console.log(zoomy);
    zoom_level = bound_zoom(Math.max(zoomx,zoomy));
    var scale = zoomScale();
    let xcen = scale*canvas.width/2;
    let ycen = scale*canvas.height/2;
    xstart = filex / 2 - xcen;
    ystart = filey / 2 - ycen;
    brightnessSelect.value = Math.max(1,Math.floor(zoom_level)*4)
}
function downloadText(text, filename){
  var element = document.createElement('a');
  element.setAttribute('href', 'data:text/plain;charset=utf-8,' + encodeURIComponent(text));
  element.setAttribute('download', filename);

  element.style.display = 'none';
  document.body.appendChild(element);

  element.click();

  document.body.removeChild(element);
}
function downloadRLE(){
    downloadText(tree.get_rle(),filename+"."+tree.get_age());
}
function onWindowResize(event){
    xsize = window.innerWidth- 10;
    ysize = window.innerHeight- 10;
    canvas.width = xsize;
    canvas.height = ysize;
}
function handleWebWorker(e){
    var workerData = e.data;
    console.log('Message received from worker');
    console.log(workerData.type);
    if (workerData.type === "ready"){
        //initialize web worker with default RLE
        myWorker.postMessage({
            type: "set_rle",
            data: RLE_STR,
        });
        //initialize web worker garbage select value
        handleGarbageSelect();
        //start calc-render loop
        renderLoop()
    }
    if (workerData.type === "serialized_tree"){
        tree = TreeDataWrapper.deserialize_treerepr(workerData.data);
        render();
        renderLoop();
    }
}
function handleGarbageSelect(e){
    myWorker.postMessage({
        type: "set_garbage_limit",
        amount: garbageSelect.value,
    })
}

brightnessSelect.addEventListener('change',render);
resetBoundingButton.addEventListener("click", resetBoundingBox, false);
downloadButton.addEventListener("click", downloadRLE, false);
garbageSelect.addEventListener('change', handleGarbageSelect);
inputFileLoader.addEventListener("change", handleFileUpload, false);
canvas.addEventListener("wheel", handle_wheel, false);
window.addEventListener('resize', onWindowResize);

myWorker.onmessage = handleWebWorker

canvas.width = xsize;
canvas.height = ysize;

set_panic_hook_js();
clearCanvas()
resetBoundingBox()

//make first render locally
render()

}
run()