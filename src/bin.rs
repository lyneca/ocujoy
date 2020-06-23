extern crate ovr_sys;
use ctrlc::set_handler;
use ovr_sys::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use vjoyrs::{Axis, Joystick};

use ggez::nalgebra::{Point3, Rotation3};
use ggez::{event, graphics, nalgebra as na, Context, GameResult};

use std::f32::consts::PI;

const RANGE: f32 = 32768.0;
const MAX_ANGLE: f32 = PI / 4.0;

fn ovr_try<F>(f: F) -> Result<(), Box<ovrErrorInfo>>
where
    F: FnOnce() -> ovrResult,
{
    let result = f();
    if OVR_SUCCESS(result) {
        Ok(())
    } else {
        let mut info = Box::new(unsafe { ::std::mem::zeroed() });
        unsafe { ovr_GetLastErrorInfo(&mut *info as *mut _) }
        Err(info)
    }
}

#[derive(Debug)]
struct EulerRotation {
    pitch: f32,
    roll: f32,
    yaw: f32,
}

impl EulerRotation {
    fn new(pitch: f32, roll: f32, yaw: f32) -> EulerRotation {
        EulerRotation { pitch, roll, yaw }
    }
}

impl From<Rotation3<f32>> for EulerRotation {
    fn from(mut item: Rotation3<f32>) -> EulerRotation {
        item.renormalize();
        let angles = item.euler_angles();
        EulerRotation {
            pitch: angles.0,
            roll: angles.1,
            yaw: angles.2,
        }
    }
}

#[derive(Debug, Clone)]
struct Transform {
    pos: Point3<f32>,
    rot: na::UnitQuaternion<f32>,
}

impl Transform {
    fn new(pose: ovrPosef) -> Transform {
        let rot = na::UnitQuaternion::new_normalize(na::Quaternion::new(
            pose.Orientation.w,
            pose.Orientation.x,
            pose.Orientation.y,
            pose.Orientation.z,
        ));
        let pos: Point3<f32> = Point3::from_slice(&[
            pose.Position.x * 400.0 + 400.0,
            pose.Position.y * -300.0 + 300.0,
            pose.Position.z * 100.0 + 100.0,
        ]);
        Transform { pos, rot }
    }
    fn default() -> Transform {
        Transform {
            pos: Point3::origin(),
            rot: na::UnitQuaternion::identity(),
        }
    }
}

struct MainState {
    session: ovrSession,
    joystick: Joystick,
    left: Transform,
    right: Transform,
    leftRef: Option<Transform>,
    rightRef: Option<Transform>,
    pitch: f32,
    roll: f32,
    yaw: f32,
}

impl MainState {
    fn new(session: ovrSession, joystick: Joystick) -> GameResult<MainState> {
        Ok(MainState {
            session,
            joystick,
            left: Transform::default(),
            right: Transform::default(),
            leftRef: None,
            rightRef: None,
            pitch: 0.0,
            roll: 0.0,
            yaw: 0.0,
        })
    }
}

pub fn minmax(val: f32, min: f32, max: f32) -> f32 {
    val.max(min).min(max)
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        unsafe {
            let mut poses: [ovrPoseStatef; 2] = ::std::mem::zeroed();
            let device: [i32; 2] = [2, 4];
            let mut input_state: ovrInputState = ::std::mem::zeroed();

            // Get controller input state
            ovr_GetInputState(
                self.session,
                ovrControllerType_Touch,
                &mut input_state as *mut ovrInputState,
            );

            let grips = input_state.HandTrigger;
            ovr_GetDevicePoses(
                self.session,
                &device as *const i32,
                2,
                0.0,
                poses.as_mut_ptr(),
            );

            self.left = Transform::new(poses[0].ThePose);
            self.right = Transform::new(poses[1].ThePose);

            if grips[1] > 0.5 {
                if let None = self.rightRef {
                    self.rightRef = Some(self.right.clone());
                }
            } else {
                self.rightRef = None;
            }

            if let Some(right_ref_point) = &self.rightRef {
                let diff = right_ref_point.rot.clone().inverse() * self.right.rot;
                let angles = diff.euler_angles();
                self.pitch = minmax(angles.0, -MAX_ANGLE, MAX_ANGLE);
                self.roll = -minmax(angles.2, -MAX_ANGLE, MAX_ANGLE);
                self.yaw = -minmax(angles.1, -MAX_ANGLE, MAX_ANGLE);
                // println!(
                //     "{:?}",
                //     right_ref_point.rot * right_ref_point.rot.clone().inverse()
                // );
                println!("{:?}\t|\t{:?}\t|\t{:?}", self.pitch, self.roll, self.yaw);
                self.joystick
                    .set_axis(
                        Axis::X,
                        (self.pitch / MAX_ANGLE * RANGE / 2.0 + RANGE / 2.0) as i32,
                    )
                    .expect("Could not set axis");
                self.joystick
                    .set_axis(
                        Axis::Y,
                        (self.roll / MAX_ANGLE * RANGE / 2.0 + RANGE / 2.0) as i32,
                    )
                    .expect("Could not set axis");
                self.joystick
                    .set_axis(
                        Axis::Z,
                        (self.yaw / MAX_ANGLE * RANGE / 2.0 + RANGE / 2.0) as i32,
                    )
                    .expect("Could not set axis");
            } else {
                self.joystick
                    .set_axis(Axis::X, (RANGE / 2.0) as i32)
                    .expect("Could not set axis");
                self.joystick
                    .set_axis(Axis::Y, (RANGE / 2.0) as i32)
                    .expect("Could not set axis");
                self.joystick
                    .set_axis(Axis::Z, (RANGE / 2.0) as i32)
                    .expect("Could not set axis");
            }
            Ok(())
        }
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());
        let red_circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            na::Point2::new(0.0, 0.0),
            5.0,
            2.0,
            graphics::Color::new(1.0, 0.0, 0.0, 1.0),
        )?;
        let green_circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            na::Point2::new(0.0, 0.0),
            5.0,
            2.0,
            graphics::Color::new(0.0, 1.0, 0.0, 1.0),
        )?;

        for (i, axis) in [self.pitch, self.roll, self.yaw].iter().enumerate() {
            let length = 30f32;
            let p1 = na::Point2::new(i as f32 * 50.0 + 325.0, 350.0);
            let p2 = p1 + na::Vector2::new(axis.cos(), axis.sin()) * length;
            let line = graphics::Mesh::new_line(ctx, &[p1, p2], 2.0, ggez::graphics::WHITE)?;
            graphics::draw(ctx, &line, graphics::DrawParam::default())?;
        }

        graphics::draw(
            ctx,
            &green_circle,
            (na::Point2::new(self.left.pos.x, self.left.pos.y),),
        )?;
        graphics::draw(
            ctx,
            &red_circle,
            (na::Point2::new(self.right.pos.x, self.right.pos.y),),
        )?;
        graphics::present(ctx)?;
        Ok(())
    }
    fn quit_event(&mut self, _ctx: &mut Context) -> bool {
        unsafe {
            println!("Quitting");
            ovr_Destroy(self.session);
            ovr_Shutdown();
        }
        false
    }
}

fn main() -> GameResult {
    unsafe {
        let mut params: ovrInitParams = ::std::mem::zeroed();
        params.Flags |= ovrInit_RequestVersion + 0x00000010;
        params.RequestedMinorVersion = OVR_MINOR_VERSION;
        ovr_try(|| ovr_Initialize(&params as *const _)).unwrap();
        let mut session: ovrSession = ::std::mem::zeroed();
        let mut luid: ovrGraphicsLuid = ::std::mem::zeroed();
        ovr_try(|| ovr_Create(&mut session as *mut _, &mut luid as *mut _)).unwrap();
        assert!(!session.is_null());
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        set_handler(move || stop_clone.store(true, Ordering::SeqCst))
            .expect("Error setting SIGINT handler");
        let cb = ggez::ContextBuilder::new("super_simple", "ggez");
        let (ctx, event_loop) = &mut cb.build()?;
        let mut joystick = Joystick::new(1);
        joystick.acquire();
        let state = &mut MainState::new(session, joystick)?;
        event::run(ctx, event_loop, state)
    }
}
