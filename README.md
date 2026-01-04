![demo](examples/demo.gif)
# ecosim
this was a fun rust project i've been working on in my free time. its a sim of the "real world" with rudimentary plants, herbivores, and predators. they all follow a set of rules, and you can see how their population changes over time. 

## chain
  - plants grow over time and spread accross the grid
  - herbivores eat plants, and grow (in population)
  - predators eat herbivores

again, this is all pretty self explanatory, and you can check the graphs for changing info as the sim plays. 

## fun features
  - you can track a singular speci by clicking on it and you can see it's seperate stats
  - there are numerous graphs for:
    - herbivore vs predator population
    - births and deaths per tick
    - average plant density
    - average energy per species
  - there's seeds so you can run the same sim at different times, just be sure to save it somewhere
  - you can check if the # of species will lag on your computer or not (if unsure, just go for a way lower number)


## controls
  - **Enter** – start  
  - **Click** – track 
  - **Space** – pause / resume  
  - **R** – restart with the same settings  
  - **N** – generate a new random seed  
  - **+ / -** – change simulation speed  
  - **S** – save simulation  
  - **L** – load simulation  
  - **Esc** – quit

## examples
  - ima update this later

## Running it locally
clone the repo
   - ```git clone https://github.com/aryasalem09/ecosim.git ```
   - ```cd ecosim```
run this command
  - ```cargo run --release``` (just so it's smooth, you can run ```cargo run``` aswell)

