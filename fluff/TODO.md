0.1 Release - Most Basic Demo
==============================

* ~~Creatures have health, can attack, and can die~~
* ~~Units belong to a team~~
* ~~Refactoring~~


0.2 Release - Fun Demo
==============================

* Tileset
* Ambient Movement
* More creatures
* Refactor/Extend, esp. attacking, actions, and goals
* Polish Selection UI
* Test Suite
* Restore ascii to functionality
    - Important for hacker types
* Add a webclient
    - Makes it trivial to showcase the game to new people 
* ~~Network~~


0.3 Release - Actually a Game
==============================

* Start Menu
* Items
* Buildings
* Macro up the toml loading
* Fix pathfinding int hack
* Early Procedural Generation
* Better Map Editor
* Ramps and Floors as property


Feature Stack
=================

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
