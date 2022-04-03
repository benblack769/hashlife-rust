Simulation remains a considerable challenge for computers, and a very active area of reaserch (if you are not already familar with it, check out the youtube channel [Two minute papers](https://www.youtube.com/c/K%C3%A1rolyZsolnai)). One key trick that complex physics engines, such as fluid dynamics engines use is to model groups of particles as a single higher level abstractions such as [vorticies](https://en.wikipedia.org/wiki/Vortex). Another key trick is to partition particles into sectors where only low level interactions within a sector matter, so interactions between sectors can be ignored.

While physicists and computational scientists have developed some fabulous abstractions and partitioning strategies and deployed them effieciently on hardware, us computer scientists wonder if there is some principle that would allow these higher level abstractions to be automatically determined, perhaps through some sort of machine learning model.

With real physics, its really hard to develop a concrete training regime that we can have confidence will work. But we computer scientists are not limited to real physics, and can work with simpler physics models that are more approachable.

Today, we will look at [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) as an example physics regime. If you are not familiar with this system, you can play around with it [here](https://conwaylife.com/).

There are three properties of this system which are very useful for efficient simulation:

1. The particles are laid out on a grid. This makes for easy and efficient partitioning.
2. The particles values are discrete. This means we can used a hard hash to memoize the results, essentially a trivial machine learning method.
3. The "speed of light", i.e. the fastest that information can travel, is known and quite slow. This means we do not need to make any assumptions when partioning or building abstractions, out results will be exact.


