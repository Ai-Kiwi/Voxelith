# Voxelith

Voxelith is a development game inspired by Teardown and Minecraft. It is currently more of a tech demo for me to learn computer graphics rendering. The project was coded in Rust using wgpu. 

## Images

### Shadows
<img alt="image" src="https://github.com/user-attachments/assets/2fab083f-b82e-4a72-8018-68984bc78124" />

### Example Performance
<img width="2560" height="1440" alt="image" src="https://github.com/user-attachments/assets/20b922c9-87be-4391-b2c0-209385a6636a" />
Running on a rtx 3060 with i5-14400f at 1440p. This shows how it is able to handle large worlds, as no LOD system is currently here as well as no chunk unloading, everything on screen is full quality. The idea is with LOD added and saving of chunks for much faster loading it will be able to handle large worlds. 

### Example Volumetric Lighting
<img alt="image" src="https://github.com/user-attachments/assets/72a72036-aef9-4f68-9c82-b634dcce5b0c" />
Volumetric lighting settings are also editable, so you can set them to be more aggressive or more relaxed. The idea is to later code it to be biome-dependent. 

### Moving Shadows. 
<img alt="image" src="https://github.com/user-attachments/assets/e84fb1ab-07a8-436c-8cd1-db93040471a3" />
Entities or objects (including terrain) that move have moving shadows that update with frame-level accuracy.

## Features And Todo

### Current features.
 - Real time editable terrain with low latency.
 - Chunk Generation.
 - Multithreaded approach for rendering, chunk generation, mesh creation and game logic.
 - Volumetric lighting
 - Cascaded Shadow Maps using LOD levels.
 - Transparency system. 
 - Multi-draw indirect rendering approach.
   - Used for entites and terrain, terrain is one draw call and each entity type is one draw call. Lose objects are also grouped in with terrain so all that is 1 call. (terrain and lose objects are still split by buffer number and if transparent)
 - 256MB GPU mesh buffers for rendered content, with automatic defragmentation.
   - Shifts meshs left in buffer, if data to the left is to small to hold chunk data. (prevents unneeded moves) 
 - Automatic creation of new mesh buffers when they are too full. 
 - An instance-based system for entity rendering.
 - Automactic chunk loading and unloading.
 - Infinite world.

### Work in progress.
 - lose objects with physics (objects rendering and stored, no physics yet)
 - Physics system for entites.
 - Mesh editor.
 - Full lighting system
 - (postponed) LOD systems, so the GPU doesn't render full quality all at once.
 - See non-sun-based shadow in volumetric lighting. 
 - Fix for volumetric lighting blowing out the whole scene's colours. 

devlogs for project : https://github.com/Ai-Kiwi/ai-kiwi-devlog/tree/main/voxelith 

implantation notes/to change
 - Mesh buffer defragmentation moves mesh by mesh, doesn't support bulk moves.
   - Possible fix is to keep running total as to move, then move in one call. 
   - Another approach is to build a list of new data from first moved location in move buffer. 
   - Both these ideas have to make sure to keep smaller then move buffer. (Could move to cpu but best to keep in gpu)
 - Naming for internal terrain/lose objects is weird.
 - Buffer defrag system needs to move out mesh file in wgpu.
 - wgpu RenderState has to much in it. Should be made up of smaller structs.
 - Shadows shouldn't be item for each one, should be a buffer array instead of something along those lines.
 - optimized system for colistion detection used 3d tree system to test "regions"
 - region system for chunk loading
 - region system for things such as detecting chunks in range, to unload or to render

Plans for entity system
 - load/unload ranges
 - range for when entities would be lazy 
 - mob cap, different types eg hostile, passive etc
 - LOD system, they do less the further away you go. Would use buffer ranges for activation. So it doesn't keep turning on and off.
 - region system so entities so quickly can fetch which entities are nearby.
Currently unsure how to store chunk region data so that it doesn't refetch so will figure out when coming back to. Approach I can think of atm are.
 - sort entities by chunk. Means that there would be to many reargements on move and acould be O(n^2)
 - store list of entities in each chunk. Would be ~2 ram fetches per entity meaning CPU prefetch would be horrible. This would add up dramatically. Could just store location in vec of enttiys (which is what I would do) to lower but still ram fetches and not quite as fast. (worth noting have to remember to remove or change this location as well if actual entity one gets moved)d
