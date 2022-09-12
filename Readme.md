# Skeletal Animation Editor

This is a 2D skeletal animation editor was implemented with Rust and the Game Engine Bevy. A recent Bevy update added skeletal animation functionality to the game engine. This project does not use this functionality. It was put together using Bevy's parent child system and mesh rendering.

## User Manual

The editor allows the creation of a hierarchical bone structure and the generation of a 2D mesh from a simple png-file. Meshes can be bound to one or multiple bones and they will be deformed when the corresponding bones are moved, rotated or scaled. It is possible to create animations. An animation consists of keyframes. Between keyframes sufficient frames to create a fluent animation will be generated using interpolation. The nature of interpolation can be specified per keyframe by changing the easing function. The editor also supports animation layering and additive animation blending.

### Selection and Transformation

Bones as well as Sprites can be selected and transformed.

|             Input             |               Action               |
| ----------------------------- | ---------------------------------- |
| LMouse                        | Select closest entity / Confirm transformation |
| LShift + LMouse               | Add / Substract from selection     |
| LMouse + Drag                 | Select entities using rubber band  |
| G                             |      Move selected entities        |
| S                             |     Scale selected entities        |
| R                             |     Rotate selected entities       |
| Delete                        | Delete all selected entities       |


### Skeleton Creation

Use **LCtrl + LMouse** to create a new bone. The currently selected bone will automatically be assigned as parent bone.

### Skins

Inside the window labeled 'Skins' a graphics file can be selected. The listed files are stored in the folder './assets/img'. The values 'cols' and 'rows' can be adjusted to define the grid that will be used to generate the skins mesh. 'add skin' will create a regular skin. 'add as cloth' will create a physics-simulated cloth. Currently it isn't possible to pin/unpin a cloth's vertices or change the cloths shape. All cloths are rectangular and the top row of vertices is pinned.

### Bind / Unbind Skin

Create a skeleton along the shape of the unbound skin. Select both skin and bones and press **A**. The selected skin is now bound to the selected bones. To unbind a skin, select it, then press **LCtrl + A**.

### Animations

Inside the window labeled 'Animations' various animation settings can be adjusted, animations can be created and edited. Under **Animations** the method of blending animations can be changed. Currently there are two settings: layering and 4-way additive blending. 'layering' simply replaces parts of the animation on lower levels, if the current layer provides values for a given bone. 4-way additive blending merges 4 animations into one using the mouse position to determine the weight of each of the 4 animations. Layers with higher numbers are above layers with lower numbers.

The plots serve to adjust the timing of an animation. Keyframes can be moved with LControl + LMouse. Multiple plots can be displayed simultaneously. This is solely for ease editing and doesn't affect the animation.

|             Input             |               Action               |
| ----------------------------- | ---------------------------------- |
| Click on plot                 | Select animation / Select closest keyframe |
| LControl + LMouse + Drag      | Move a keyframe affecting the animations timing  |
| K                             | Add keyframe for selected bones    |
| P                             | Play / Pause animation             |

### Save and Load

LControl + Number: save animation (skeleton, skin, animation layers and settings) to one of 10 save slots
LAlt + Number: load previously saved animation

Currently WebAssembly doesn't support saving animations. Default animations can still be loaded.

### Show/Hide Debug Shapes

Displaying bones and meshes can be toggled.

|             Input             |               Action               |
| ----------------------------- | ---------------------------------- |
| B                             | Show / Hide bones                  |
| M                             | Show / Hide mesh vertices and edges|

![ease in out](img/pooh.gif)
![ease out elastic](img/pooh_elastic.gif)

## Installation

### WebAssembly

Generate WASM files with:

```
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/skeletal-animation-2D-editor.wasm
```