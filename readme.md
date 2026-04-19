# Voxelith

Voxelith is a development game inspired by Teardown and Minecraft. It is currently more of a tech demo for me to learn computer graphics. The project was coded in Rust using wgpu. 

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
 - 256MB GPU mesh buffers for rendered content, with automatic defragmentation.
 - Automatic creation of new mesh buffers when they are too full. 
 - An instance-based system for entity rendering.
 - Automactic chunk loading and unloading.
 - Infinite world.

### Work in progress.
 - LOD systems, so the GPU doesn't render full quality all at once.
 - Mesh editor.
 - See non-sun-based shadow in volumetric lighting. 
 - Full lighting system
 - Physics system. 
 - Fix for volumetric lighting blowing out the whole scene's colours. 
  
devlogs for project : https://github.com/Ai-Kiwi/ai-kiwi-devlog/tree/main/voxelith 
