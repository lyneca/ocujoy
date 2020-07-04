# OcuJoy

Map Oculus Touch motion/rotation to a vJoy virtual joystick, primarily for Elite: Dangerous in VR.

## Usage

1. Install [vJoy](http://vjoystick.sourceforge.net/site/index.php/download-a-install/download).
2. Grab a release from the releases section, unzip it, and run it. A debug window will pop up.
3. Included in the release will be a .binds file, which is my personal input bindings.

   You may have to tweak some values that use vJoy to get the right axis and inversions:
   
   - Flight Thrust X, Y, Z
   - Galaxy Map X, Y, Z

You don't need to have anything (e.g. SteamVR) running before you run it, and you can
close/reopen OcuJoy whenever you want. It won't block other Oculus applications.

### Left Stick

The left stick is mapped to vJoy's X, Y and Z coordinates.

- Grab a point in space using the grip button, then drag away in any direction
  to increase a particular axis.
- Currently this is mapped on a slight exponential curve - I've found that this
  helps with flight precision.
- I use this for Flight Thrust, as in my control scheme I have increase/decrease
  throttle bound to `Left Touch Joystick Y`. This allows you to thrust in any
  direction using the controller, and use the throttle joystick when you don't want
  to keep gripping the controller.

### Right Stick

The right stick is mapped to vJoy's rX, rY and rZ axis.

- Grab a point in space using the grip button, then rotate the stick on any set
  of axes.
- I use this for pitch/roll/yaw. The rotation limits are set to match the in-game
  joystick rotation limits as closely as possible, so you should see the in-game
  joystick match your hand position.
- The rotation is mapped based on your _starting hand rotation_ - it doesn't matter
  which way your hand is pointing when you grip.
