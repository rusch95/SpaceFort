0.1 Release - Most Basic Demo
==============================

* ~~Creatures have health, can attack, and can die~~
* ~~Units belong to a team~~
* ~~Refactoring~~


0.2 Release - Fun Demo
==============================

* ~~Network~~
* ~~Highlight selected units~~
* Click to select single unit
* Make attack correct and add chance
* Tileset
* Unit spawning in section when player joins
* Buildings that spawn units


0.3 Release - Cool / Technical Debt Demo
=============================

* Extend test suite
* Map Editor
* Ramps and Floors as property
* Macro up the toml loading
* Restore ascii to functionality
    - Important for hacker types
* Add a webclient
    - Makes it trivial to showcase the game to new people 
* Test Suite


0.4 Release - Actually a Game
==============================

* More creatures
* Start Menu
* Items
* Fix pathfinding int hack
* Early Procedural Generation


Feature Stack
=================

* Ambient Movement
* Add momentum changing cost to pathfinding
* Physics
* Liquids
* Clouds
* Save games
* Render opacity


Nice Things
================

* Remove players if they disconnect
* Add solo bin, so no need to do server / client shindig
* Have PlayerID and TeamID, so that multiple players can be on the same team


Bugs
=======

* Can't run release mode on windows because of OpenGL issue
* Chunking is a bit broken. Make proper tests and fix any small issues.
