0.1 Release - Most Basic Demo
==============================

* ~~Creatures have health, can attack, and can die~~
* ~~Units belong to a team~~
* ~~Refactoring~~


0.2 Release - Fun Demo
==============================

* ~~Network~~
* ~~Highlight selected units~~
* ~~Click to select single unit~~
* ~~Make attack correct~~
* Items
* Buildings that spawn units


0.3 Release - Cool / Technical Debt Demo
=============================

* ~~Fix click~~
* ~~Test Suite~~
* Macro up the toml loading
* ~~Restore ascii to functionality~~
    - Important for hacker types
* Map Editor
* Ramps and Floors as property
* ~~Investigate switching to SDL2~~
    - Nah
* Resource collection and cost
* Tech tree
* Improved attack system


0.4 Release - Actually a Game
==============================

* Tileset
* More creatures
* Start Menu 
* Better UI 
* Fix pathfinding int hack
* Early Procedural Generation


0.5 Release - Delayed Things I Think Are Good
==============================

* Ambient Movement
* Liquids
* Add a webclient
    - Makes it trivial to showcase the game to new people 
* ASCII with full game support 
* Save games

Feature Stack
=================

* Clouds
* Add momentum changing cost to pathfinding
* Physics
* Render opacity


Nice Things
================

* Remove players if they disconnect
* Add solo bin, so no need to do server / client shindig
* ~~Have PlayerID and TeamID, so that multiple players can be on the same team~~
* Switch TeamID to enum None, Nature, Team(ID), Evil


Bugs
=======

* Can't run release mode on windows because of OpenGL issue
* Chunking is a bit broken. Make proper tests and fix any small issues.
* Currently, if a task is subsummed by an assigned task, the task is dropped. Fix it so the task
ownership system actually works.
