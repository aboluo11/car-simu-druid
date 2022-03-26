use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use druid::kurbo::BezPath;
use druid::piet::{FontFamily, ImageFormat, InterpolationMode, Text, TextLayoutBuilder};
use druid::widget::{prelude::*, Controller, Painter};
use druid::{
    Affine, AppLauncher, Color, FontDescriptor, LocalizedString, Point, Rect, TextLayout,
    WindowDesc, KbKey, WidgetExt, KeyEvent, Lens,
};
use car_simu_lib::{Map, SCALE};

#[derive(Clone)]
struct Car {
    inner: car_simu_lib::Car
}

impl Deref for Car {
    type Target = car_simu_lib::Car;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Car {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Data for Car {
    fn same(&self, other: &Self) -> bool {
        self.lt.origin == other.lt.origin && self.lt.rotation_matrix == other.lt.rotation_matrix &&
        self.rt.origin == other.rt.origin && self.rt.rotation_matrix == other.rt.rotation_matrix
    }
}

struct CustomController {
    down: Option<KbKey>,
    t: u64,
    frames: u64,
    successive: bool,
}

impl Controller<Car, Painter<Car>> for CustomController {
    fn event(&mut self, child: &mut Painter<Car>, ctx: &mut EventCtx, event: &Event, car: &mut Car, env: &Env) {
        match &event {
            Event::KeyDown(ke) => {
                if let Some(_) = self.down {
                    return;
                }
                match ke.key {
                    KbKey::ArrowUp => {
                        self.down = Some(KbKey::ArrowUp);
                        ctx.request_anim_frame();
                    }
                    KbKey::ArrowDown => {
                        self.down = Some(KbKey::ArrowDown);
                        ctx.request_anim_frame();
                    }
                    KbKey::ArrowLeft => {
                        car.left_steer();
                    }
                    KbKey::ArrowRight => {
                        car.right_steer();
                    }
                    _ => {}
                }
            }
            Event::KeyUp(ke) => {
                match ke.key {
                    KbKey::ArrowUp | KbKey::ArrowDown => {
                        self.down = None;
                    }
                    _ => {}
                }
            }
            Event::AnimFrame(t) => {
                let mut interval = 0;
                if self.successive {
                    self.t += t;
                    interval = *t;
                    self.frames += 1;
                } else {
                    self.successive = true;
                }
                match &self.down {
                    Some(KbKey::ArrowUp) => {
                        car.forward((interval as f64) * 1e-9 * 1.);
                        ctx.request_anim_frame();
                    }
                    Some(KbKey::ArrowDown) => {
                        car.forward((interval as f64) * 1e-9 * -1.);
                        ctx.request_anim_frame();
                    }
                    _ => {
                        self.successive = false;
                        dbg!((self.frames as f64)/(self.t as f64 * 1e-9));
                        self.t = 0;
                        self.frames = 0;
                    }
                }
            }
            Event::WindowConnected => {
                ctx.request_focus();
            }
            _ => {}
        }
    }
}

fn main() {
    let window = WindowDesc::new(
        Painter::new(|ctx, car: &Car, env| {
            ctx.fill(Rect::from_center_size(
                <(f64, f64)>::from(car.body.origin.to_real()),
                (car.body.width*SCALE, car.body.height*SCALE)),
                &Color::GREEN
            );
        })
        .controller(CustomController {down: None, t: 0, frames: 0, successive: false})
    ).title("car-simu")
        .resizable(false)
        .window_size((800., 800.));
    let mut map = car_simu_lib::ParallelParking::new();
    let mut car = map.car();
    AppLauncher::with_window(window)
        .launch(Car {inner: car});
}

