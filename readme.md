# Voxelith

Voxelith is a development game inspired by Teardown and Minecraft. It is currently more of a tech demo for me to learn computer graphics. The project was coded in Rust using wgpu. 

# Images
## Shadows
<img alt="image" src="https://github.com/user-attachments/assets/2fab083f-b82e-4a72-8018-68984bc78124" />
## Example Performance
<img alt="image" src="https://github.com/user-attachments/assets/4f3b8f67-57d2-461e-81a8-2e24f8f1716e" />
Most of the lag here is still caused by the volumetric lighting system, showing that, because of rendering optimisations, it is able to handle large worlds.
## Example Volumetric Lighting
<img alt="image" src="https://github.com/user-attachments/assets/72a72036-aef9-4f68-9c82-b634dcce5b0c" />
Volumetric lighting settings are also editable, so you can set them to be more aggressive or more relaxed. The idea is to later code it to be biome-dependent. 
## Moving Shadows. 
<img alt="image" src="https://github.com/user-attachments/assets/e84fb1ab-07a8-436c-8cd1-db93040471a3" />
Entities or objects (including terrain) that move have moving shadows that update with frame-level accuracy.

# Features And Todo

## Current features.
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

## Work in progress.
 - LOD systems, so the GPU doesn't render full quality all at once.
 - Mesh editor.
 - See non-sun-based shadow in volumetric lighting. 
 - Full lighting system
 - Physics system. 
 - Fix for volumetric lighting blowing out the whole scene's colours. 
