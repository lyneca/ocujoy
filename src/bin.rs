extern crate ovr_sys;
use ctrlc::set_handler;
use ovr_sys::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ggez::nalgebra::{Point3, Rotation3};
use ggez::{event, graphics, nalgebra as na, Context, GameResult};

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
    fn from(item: Rotation3<f32>) -> EulerRotation {
        let angles = item.euler_angles();
        EulerRotation {
            pitch: angles.0,
            roll: angles.1,
            yaw: angles.2,
        }
    }
}

#[derive(Debug)]
struct Transform {
    pos: Point3<f32>,
    rot: EulerRotation,
}

impl Transform {
    fn new(pose: ovrPosef) -> Transform {
        let rot: Rotation3<f32> = na::UnitQuaternion::new_normalize(na::Quaternion::new(
            pose.Orientation.w,
            pose.Orientation.x,
            pose.Orientation.y,
            pose.Orientation.z,
        ))
        .to_rotation_matrix();
        let pos: Point3<f32> = Point3::from_slice(&[
            pose.Position.x * 400.0 + 400.0,
            pose.Position.y * -300.0 + 300.0,
            pose.Position.z * 100.0 + 100.0,
        ]);
        Transform {
            pos: pos,
            rot: EulerRotation::from(rot),
        }
    }
    fn default() -> Transform {
        Transform {
            pos: Point3::from_slice(&[0.0, 0.0, 0.0]),
            rot: EulerRotation::new(0.0, 0.0, 0.0),
        }
    }
}

struct MainState {
    session: ovrSession,
    left: Transform,
    right: Transform,
}

impl MainState {
    fn new(session: ovrSession) -> GameResult<MainState> {
        Ok(MainState {
            session,
            left: Transform::default(),
            right: Transform::default(),
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        unsafe {
            let mut poses: [ovrPoseStatef; 2] = ::std::mem::zeroed();
            let device: [i32; 2] = [2, 4];
            ovr_GetDevicePoses(
                self.session,
                &device as *const i32,
                2,
                0.0,
                poses.as_mut_ptr(),
            );
            self.left = Transform::new(poses[0].ThePose);
            self.right = Transform::new(poses[1].ThePose);
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

        for (i, axis) in [self.left.rot.pitch, self.left.rot.roll, self.left.rot.yaw]
            .iter()
            .enumerate()
        {
            let length = 30f32;
            let p1 = na::Point2::new(i as f32 * 50.0 + 325.0, 300.0);
            let p2 = p1 + na::Vector2::new(axis.cos(), axis.sin()) * length;
            let line = graphics::Mesh::new_line(ctx, &[p1, p2], 2.0, ggez::graphics::WHITE)?;
            graphics::draw(ctx, &line, graphics::DrawParam::default())?;
        }
        for (i, axis) in [
            self.right.rot.pitch,
            self.right.rot.roll,
            self.right.rot.yaw,
        ]
        .iter()
        .enumerate()
        {
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
        params.Flags |= ovrInit_RequestVersion;
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
        let state = &mut MainState::new(session)?;
        event::run(ctx, event_loop, state)
    }
}
