use std::cell::RefCell;
use std::rc::Rc;

use druid::kurbo::BezPath;
use druid::piet::{FontFamily, ImageFormat, InterpolationMode, Text, TextLayoutBuilder};
use druid::widget::{prelude::*, Controller, Painter};
use druid::{
    Affine, AppLauncher, Color, FontDescriptor, LocalizedString, Point, Rect, TextLayout,
    WindowDesc, KbKey, WidgetExt, KeyEvent,
};
use car_simu_lib::{Map, SCALE};

#[derive(Data)]
struct Car(car_simu_lib::Car);

struct CustomController;

impl Controller<Car, Painter<Car>> for CustomController {
    fn event(&mut self, child: &mut Painter<Car>, ctx: &mut EventCtx, event: &Event, car: &mut Car, env: &Env) {
        match &event {
            Event::KeyDown(ke) => {
                match ke.key {
                    KbKey::ArrowUp => {
                        car.borrow_mut().forward(0.3);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let window = WindowDesc::new(
        Painter::new(|ctx, car: &Car, env| {
            ctx.fill(Rect::from_center_size(
                <(f64, f64)>::from(car.borrow().body.origin.to_real()),
                (car.borrow().body.width*SCALE, car.borrow().body.height*SCALE)),
                &Color::GREEN
            );
        })
        .controller(CustomController {})
    ).title("car-simu")
        .window_size_policy(druid::WindowSizePolicy::Content)
        .resizable(false);
    let mut map = car_simu_lib::ParallelParking::new();
    let mut car = map.car();
    AppLauncher::with_window(window)
        .launch(Rc::new(RefCell::new(car)));
}

