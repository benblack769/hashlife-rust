Simulation remains a considerable challenge for computers, and a very active area of reaserch (if you are not already familar with it, check out the youtube channel [Two minute papers](https://www.youtube.com/c/K%C3%A1rolyZsolnai)). One key trick that complex physics engines, such as fluid dynamics engines use is to model groups of particles as a single higher level abstractions such as [vorticies](https://en.wikipedia.org/wiki/Vortex). Another key trick is to partition particles into sectors where only low level interactions within a sector matter, so interactions between sectors can be ignored.

While physicists and computational scientists have developed some fabulous abstractions and partitioning strategies and deployed them effieciently on hardware, us computer scientists wonder if there is some principle that would allow these higher level abstractions to be automatically determined, perhaps through some sort of machine learning model.

With real physics, its really hard to develop a concrete training regime that we can have confidence will work. But we computer scientists are not limited to real physics, and can work with simpler physics models that are more approachable.

Today, we will look at [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) as an example physics regime. If you are not familiar with this system, you can play around with it [here](https://conwaylife.com/).

There are three properties of this system which are very useful for efficient simulation:

1. The particles are laid out on a grid. This makes for easy and efficient partitioning.
2. The particles values are discrete. This means we can used a hard hash to memoize the results, essentially a trivial machine learning method.
3. The "speed of light", i.e. the fastest that information can travel, is known and quite slow. This means we do not need to make any assumptions when partioning or building abstractions, out results will be exact.


<style  type="text/css" rel="stylesheet">

@keyframes cf4FadeInOut {
0% {
opacity:1;
}
7.6% {
opacity:1;
}
7.7% {
opacity:0;
}
99.9% {
opacity:0;
}
100% {
opacity:1;
}
}

#cf4a {
position:relative;
height:10cm;
width:10cm;
margin:0 auto;
}
#cf4a img {
position:absolute;
left:0;
}

#cf4a img{
animation-name: cf4FadeInOut;
animation-timing-function: ease-in-out;
animation-iteration-count: infinite;
animation-duration: 7.5s;
}

#cf4a img:nth-of-type(1) {
animation-delay: 7.5s;
}

#cf4a img:nth-of-type(2) {
animation-delay: 7.0s;
}

#cf4a img:nth-of-type(3) {
animation-delay: 6.5s;
}

#cf4a img:nth-of-type(4) {
animation-delay: 6.0s;
}

#cf4a img:nth-of-type(5) {
animation-delay: 5.5s;
}

#cf4a img:nth-of-type(6) {
animation-delay: 5.0s;
}

#cf4a img:nth-of-type(7) {
animation-delay: 4.5s;
}

#cf4a img:nth-of-type(8) {
animation-delay: 4.0s;
}

#cf4a img:nth-of-type(9) {
animation-delay: 3.5s;
}

#cf4a img:nth-of-type(10) {
animation-delay: 3.0s;
}

#cf4a img:nth-of-type(11) {
animation-delay: 2.5s;
}

#cf4a img:nth-of-type(12) {
animation-delay: 2.0s;
}

#cf4a img:nth-of-type(13) {
animation-delay: 1.5s;
}

</style>
<div id="cf4a">
<img src="https://raw.githubusercontent.com/Farama-Foundation/PettingZoo/master/docs/slideshow_svgs/checkerboard_12.svg"/>
<img src="https://raw.githubusercontent.com/Farama-Foundation/PettingZoo/master/docs/slideshow_svgs/checkerboard_11.svg"/>
<img src="https://raw.githubusercontent.com/Farama-Foundation/PettingZoo/master/docs/slideshow_svgs/checkerboard_10.svg"/>
<img src="https://raw.githubusercontent.com/Farama-Foundation/PettingZoo/master/docs/slideshow_svgs/checkerboard_9.svg"/>
<img src="https://raw.githubusercontent.com/Farama-Foundation/PettingZoo/master/docs/slideshow_svgs/checkerboard_8.svg"/>
<img src="https://raw.githubusercontent.com/Farama-Foundation/PettingZoo/master/docs/slideshow_svgs/checkerboard_7.svg"/>
<img src="https://raw.githubusercontent.com/Farama-Foundation/PettingZoo/master/docs/slideshow_svgs/checkerboard_6.svg"/>
<img src="https://raw.githubusercontent.com/Farama-Foundation/PettingZoo/master/docs/slideshow_svgs/checkerboard_5.svg"/>
<img src="https://raw.githubusercontent.com/Farama-Foundation/PettingZoo/master/docs/slideshow_svgs/checkerboard_4.svg"/>
<img src="https://raw.githubusercontent.com/Farama-Foundation/PettingZoo/master/docs/slideshow_svgs/checkerboard_3.svg"/>
<img src="https://raw.githubusercontent.com/Farama-Foundation/PettingZoo/master/docs/slideshow_svgs/checkerboard_2.svg"/>
<img src="https://raw.githubusercontent.com/Farama-Foundation/PettingZoo/master/docs/slideshow_svgs/checkerboard_1.svg"/>
<img src="https://raw.githubusercontent.com/Farama-Foundation/PettingZoo/master/docs/slideshow_svgs/checkerboard_0.svg"/>

</div>
