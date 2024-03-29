# Skeletal Animation Editor

This is a 2D skeletal animation editor was implemented with Rust and the Game Engine Bevy. A recent Bevy update added skeletal animation functionality to the game engine. This project does not use this functionality. It has its own implementation of skeletal animation logics.

A WebAssembly demo of the application can be found here:
https://lordbenedikt.github.io/skeletal_animation_2d_remake/

This app was implemented as addition to my bachelor thesis, which can be viewed [here](thesis/thesis.pdf).


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

Inside the window labeled 'Skins' a graphics file can be selected (I found most of the graphics used inside of the app online and don't own them). The listed files are stored in the folder './assets/img'. Any custom PNG-file can be added by placing it inside of that folder. The values 'cols' and 'rows' can be adjusted to define the grid that will be used to generate the skins mesh. 'add skin' will create a regular skin. 'add as cloth' will create a physics-simulated cloth. Below 'Delaunay Triangulation' there is a second 'add skin' button that will use Delaunay Triangulation to generate a tightly fitted mesh for the image. Currently there is no algorithm implemented for triangle ordering, so self overlap can't be handled well. Currently it isn't possible to pin/unpin a cloth's vertices or change the cloths shape. All cloths are rectangular and the top row of vertices is pinned.

### Bind / Unbind Skin

Create a skeleton along the shape of the unbound skin. Select both skin and bones and press **A**. The selected skin is now bound to the selected bones. To unbind a skin, select it, then press **LCtrl + A**.
The weighting of each vertex relative to each bone can be adjusted in the **Adjust Vertex Weights** mode. It can be toggled with **W** or by clicking the button in the Skins-menu. A circle around the cursor will indicate the area of adjustment. With **Q** the weights for the vertices within the circle are reduced, with **E** they are increased. Currently this mode is still a work in progress and might not be intuitive to use yet.

### Animations

Inside the window labeled 'Animations' various animation settings can be adjusted, animations can be created and edited. Under **Animations** the method of blending animations can be changed. Currently there are two settings: layering and 4-way additive blending. 'layering' simply replaces parts of the animation on lower levels, if the current layer provides values for a given bone. 4-way additive blending merges 4 animations into one using the mouse position to determine the weight of each of the 4 animations. Layers with higher numbers are above layers with lower numbers.

The following 3 GIFs show first the lower layer of an animation, then the layer above, and finally the resulting combined animation. The top layer animation only includes the right arm of the character.

|             lower layer             | top layer            | resulting combined animation |
| ----------------------------- | ---------------------------------- | ---- |
|![lower layer animation](img/layering_1.gif)|![top layer animation](img/layering_0.gif) | ![combined animation](img/layering_2.gif) |
|![lower layer animation](img/layering_bones_1.gif)|![top layer animation](img/layering_bones_0.gif) | ![combined animation](img/layering_bones_2.gif) |

The following GIFs show an example for additive animation blending. Here four animations have been blended into one. The percentage, to which each animation influences the result, is determined by the mouse position.

|         combined animation        |       skeleton         |
| ----------------------------- | ---------------------------------- |
|![lower layer animation](img/additive_blending.gif)|![top layer animation](img/additive_blending_bones.gif) |

### Keyframe Plot

The plots serve to adjust the timing of an animation. Keyframes can be moved with LControl + LMouse. Multiple plots can be displayed simultaneously. This is solely for ease of editing and doesn't affect the animation.

|             Input             |               Action               |
| ----------------------------- | ---------------------------------- |
| Click on plot                 | Select animation / Select closest keyframe |
| LControl + LMouse + Drag      | Move a keyframe affecting the animations timing  |
| K                             | Add keyframe for applicable components |
| J                             | Replace keyframe for applicable components |
| P                             | Play / Pause animation             |

### Inverse Kinematics

It is possible to place a target for a bone. This bone and its parents, until the depth specified in the animation window, will now reach for this target using an inverse kinematics algorithm (cyclic coordinate descent). Reaching for a target has priority over the keyframe animation, so applicable bones will ignore it. Multiple targets for the same bones are not supported and will result in undefined beheaviour.

|             Input             |               Action               |
| ----------------------------- | ---------------------------------- |
| LAlt + LeftMouse              | Create a target, the selected bone will be the end effector that reaches for this target |

### Save and Load

Desktop version:
|             Input             |               Action               |
| ----------------------------- | ---------------------------------- |
| LControl + Number             | save animation (skeleton, skin, animation layers and settings) to one of 10 save slots        |
| LAlt + Number                 | load previously saved animation |

WebAssembly:
|             Input             |               Action               |
| ----------------------------- | ---------------------------------- |
| Save Button in animation menu             | download animation |
| Load Button in animation menu             | upload locally saved animation |
| LAlt + Number                 | load default animations |

### Show/Hide Debug Shapes

Displaying bones and meshes can be toggled.

|             Input             |               Action               |
| ----------------------------- | ---------------------------------- |
| B                             | Show / Hide bones                  |
| M                             | Show / Hide mesh vertices and edges|

|             linear            |               ease in out          |
| ----------------------------- | ---------------------------------- |
| ![linear](img/interpolate_linear.gif)  | ![ease in out](img/interpolate_ease_in_out.gif)   |

|            ease out elastic       |               ease in out back     |
| ----------------------------- | ---------------------------------- |
| ![ease out elastic](img/interpolate_ease_out_elastic.gif)  | ![ease in out back](img/interpolate_ease_in_out_back.gif)   |



### Controls Summary

<table>
  <tr>
    <td colspan="2"><b>Create Components</b></td>
  </tr>
  <tr>
    <td>LCtrl + LMouse</td>
    <td>Create a bone, selected bone will automatically be assigned as parent bone</td>
  </tr>
  <tr>
    <td>LAlt + LeftMouse</td>
    <td>Create a target, selected bone will be the end effector that reaches for this target</td>
  </tr>
</table>

<table>
  <tr>
    <td colspan="2"><b>Select / Unselect</b></td>
  </tr>
  <tr>
    <td>LMouse</td>
    <td>Select closest entity</td>
  </tr>
  <tr>
    <td>LShift + LMouse</td>
    <td>Add / Substract from selection</td>
  </tr>
  <tr>
    <td>LMouse + Drag</td>
    <td>Select entities using rubber band</td>
  </tr>
  <tr>
    <td>Click on plot</td>
    <td>Select animation / Select closest keyframe</td>
  </tr>
  <tr>
    <td>LControl + LMouse + Drag</td>
    <td>Move a keyframe affecting the animations timing</td>
  </tr>
</table>

<table>
  <tr>
    <td colspan="2"><b>Transforming</b></td>
  </tr>
  <tr>
    <td>G</td>
    <td>Move selected entities</td>
  </tr>
  <tr>
    <td>S</td>
    <td>Scale selected entities</td>
  </tr>
  <tr>
    <td>R</td>
    <td>Rotate selected entities</td>
  </tr>
  <tr>
    <td>Delete</td>
    <td>Delete selected entities</td>
  </tr>
  <tr>
    <td>K</td>
    <td>Add keyframe for applicable components</td>
  </tr>
  <tr>
    <td>J</td>
    <td>Replace keyframe for applicable component</td>
  </tr>
</table>

<table>
  <tr>
    <td colspan="2"><b>Other</b></td>
  </tr>
  <tr>
    <td>A</td>
    <td>Assign selected skins to selected bones</td>
  </tr>
  <tr>
    <td>LControl + A</td>
    <td>Unbind selected skins</td>
  </tr>
  <tr>
    <td>P</td>
    <td>Play / Pause animation</td>
  </tr>
  <tr>
    <td>B</td>
    <td>Show / Hide bones</td>
  </tr>
  <tr>
    <td>M</td>
    <td>Show / Hide mesh vertices and edges</td>
  </tr>
  <tr>
    <td>W</td>
    <td>Toggle adjust vertex weights mode</td>
  </tr>
  <tr>
    <td>Q / E</td>
    <td>Decrease / Increase vertex weights</td>
  </tr>
  <tr>
    <td>LMouse</td>
    <td>Confirm Transformation</td>
  </tr>
  <tr>
    <td>LControl + Number</td>
    <td>save animation (skeleton, skin, animation layers and settings) to one of 10 save slots</td>
  </tr>
  <tr>
    <td>LALT + Number</td>
    <td>load previously saved animation</td>
  </tr>
</table>

## Installation

### Rust

To compile and run the code you will need a working Rust installation. If you are new to Rust you can follow these instructions: https://www.rust-lang.org/tools/install

Once Rust is installed, simply run following command inside of the project folder for a test run:

```
cargo run
```

For better performance set the 'release' flag:

```
cargo run --release
```

### WebAssembly

To compile and run the code in your browser, you first need to add WASM support and install the wasm-server-runner tool:

```
rustup target install wasm32-unknown-unknown
cargo install wasm-server-runner
```

Test run the application with:
```
cargo run --target wasm32-unknown-unknown
```

If you want to compile and deploy the application to a website generate the WASM files with:

```
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/skeletal-animation-2D-editor.wasm
```

Then upload these files together with the assets folder and the index.html.