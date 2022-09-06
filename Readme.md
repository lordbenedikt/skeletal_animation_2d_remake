# Skeletal Animation Editor

This is a 2D skeletal animation editor was implemented with Rust and the Game Engine Bevy. A recent Bevy update added skeletal animation functionality to the game engine. This project does not use this functionality. It was put together using Bevy's parent child system and mesh rendering.

## User Manual

The editor allows the creation of a hierarchical bone structure and the generation of a 2D mesh from a simple png-file. Meshes can be bound to one or multiple bones and they will be deformed when the corresponding bones are moved, rotated or scaled. It is possible to create animations. An animation consists of keyframes. Between keyframes sufficient frames to create a fluent animation will be generated using interpolation. The nature of interpolation can be specified per keyframe by changing the easing function. The editor also supports animation layering and additive animation blending.

### Selection and Transformation

Bones as well as Sprites can be selected and transformed.

LeftMouse : Select closest entity.
LeftMouse + Drag : Select entities using rubber band.
G : Move selected entities.
S : Scale selected entities.
R : Rotate selected entities.
|             Input             |               Action               |
| ----------------------------- | ---------------------------------- |
|              LeftMouse        |     Select closest entity.         |
| LeftMouse + Drag              | Select entities using rubber band. |
| G                             | Move selected entities             |
| S                             | Scale selected entities            |
| R                             | Rotate selected entities           |

### Skeleton Creation

LCtrl + LeftMouse : Create a bone. The currently selected bone will automatically be assigned as parent.
Delete : Delete all selected bones.

###

![ease in out](img/pooh.gif)
![ease out elastic](img/pooh_elastic.gif)