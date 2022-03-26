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

struct CustomController;

impl Controller<Car, Painter<Car>> for CustomController {
    fn event(&mut self, child: &mut Painter<Car>, ctx: &mut EventCtx, event: &Event, car: &mut Car, env: &Env) {
        match &event {
            Event::KeyDown(ke) => {
                match ke.key {
                    KbKey::ArrowUp => {
                        // car.forward(0.3);
                        ctx.request_anim_frame()
                    }
                    KbKey::ArrowDown => {
                        car.forward(-0.3);
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
            Event::WindowConnected => {
                ctx.request_focus();
            }
            Event::AnimFrame(x) => {
                dbg!(x);
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
        .controller(CustomController {})
    ).title("car-simu")
        .resizable(false)
        .window_size((800., 800.));
    let mut map = car_simu_lib::ParallelParking::new();
    let mut car = map.car();
    AppLauncher::with_window(window)
        .launch(Car {inner: car});
}

