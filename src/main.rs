use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::Arc;

use car_simu_lib::{Map, Source, MAP_HEIGHT, SCALE, SPEED};
use druid::kurbo::{BezPath, PathEl};
use druid::widget::{prelude::*, Controller, Painter, SvgData};
use druid::{
    Affine, AppLauncher, Color, KbKey, Vec2, WidgetExt, WindowDesc,
};

#[derive(Clone)]
struct Car {
    inner: car_simu_lib::Car,
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
        self.lt.origin == other.lt.origin
            && self.lt.rotation_matrix == other.lt.rotation_matrix
            && self.rt.origin == other.rt.origin
            && self.rt.rotation_matrix == other.rt.rotation_matrix
    }
}

trait View {
    fn draw(&self, ctx: &mut PaintCtx, env: &mut HashMap<*const u8, SvgData>);
}

impl View for car_simu_lib::Rect {
    fn draw(&self, ctx: &mut PaintCtx, env: &mut HashMap<*const u8, SvgData>) {
        match self.source {
            Source::Color(color) => {
                let shape = BezPath::from_vec(vec![
                    PathEl::MoveTo(<(f64, f64)>::from(self.lt().to_real()).into()),
                    PathEl::LineTo(<(f64, f64)>::from(self.rt().to_real()).into()),
                    PathEl::LineTo(<(f64, f64)>::from(self.rb().to_real()).into()),
                    PathEl::LineTo(<(f64, f64)>::from(self.lb().to_real()).into()),
                    PathEl::ClosePath,
                ]);
                ctx.fill(shape, &Color::rgb8(color.r, color.g, color.b));
            }
            Source::Svg(data) => {
                let svg = env.entry(data.as_ptr())
                    .or_insert_with(|| {
                        SvgData::from_str(data).unwrap()
                    });
                let (ori_width, ori_height) = svg.size().into();
                let ratio = self.width * SCALE / ori_width;
                let offset_matrix = Affine::translate(Vec2::new(
                    self.origin.x * SCALE,
                    (MAP_HEIGHT - self.origin.y) * SCALE,
                )) * Affine::FLIP_Y
                    * Affine::new([
                        self.rotation_matrix.inner[0][0],
                        self.rotation_matrix.inner[1][0],
                        self.rotation_matrix.inner[0][1],
                        self.rotation_matrix.inner[1][1],
                        0.,
                        0.,
                    ])
                    * Affine::FLIP_Y
                    * Affine::scale(ratio)
                    * Affine::translate(Vec2::new(-ori_width / 2., -ori_height / 2.));
                svg.to_piet(offset_matrix, ctx)
            }
        }
    }
}

impl View for car_simu_lib::Car {
    fn draw(&self, ctx: &mut PaintCtx, env: &mut HashMap<*const u8, SvgData>) {
        self.body.draw(ctx, env);
        self.lt.draw(ctx, env);
        self.rt.draw(ctx, env);
        self.lb.draw(ctx, env);
        self.rb.draw(ctx, env);
        self.left_mirror.draw(ctx, env);
        self.right_mirror.draw(ctx, env);
        self.logo.draw(ctx, env);
    }
}

impl View for car_simu_lib::RightAngleTurn {
    fn draw(&self, ctx: &mut PaintCtx, env: &mut HashMap<*const u8, SvgData>) {
        let svg = env.entry(self.svg.as_ptr())
            .or_insert_with(|| {
                SvgData::from_str(self.svg).unwrap()
        });
        svg.to_piet(Affine::IDENTITY, ctx);
    }
}

struct CustomController {
    down: Option<KbKey>,
    t: u64,
    frames: u64,
    successive: bool,
}

impl Controller<Car, Painter<Car>> for CustomController {
    fn event(
        &mut self,
        child: &mut Painter<Car>,
        ctx: &mut EventCtx,
        event: &Event,
        car: &mut Car,
        env: &Env,
    ) {
        match &event {
            Event::KeyDown(ke) => match ke.key {
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
            },
            Event::KeyUp(ke) => match ke.key {
                KbKey::ArrowUp | KbKey::ArrowDown => {
                    self.down = None;
                }
                _ => {}
            },
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
                        car.forward((interval as f64) * 1e-9 * SPEED);
                        ctx.request_anim_frame();
                    }
                    Some(KbKey::ArrowDown) => {
                        car.forward((interval as f64) * 1e-9 * -SPEED);
                        ctx.request_anim_frame();
                    }
                    _ => {
                        self.successive = false;
                        dbg!((self.frames as f64) / (self.t as f64 * 1e-9));
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
    let mut draw_env = HashMap::new();
    let map = car_simu_lib::RightAngleTurn::new();
    let mut car = map.car();
    let window = WindowDesc::new(
        Painter::new(move |ctx, car: &Car, env| {
            let region = ctx.size().to_rect();
            ctx.fill(region, &Color::WHITE);
            map.draw(ctx, &mut draw_env);
            car.draw(ctx, &mut draw_env);
        })
        .controller(CustomController {
            down: None,
            t: 0,
            frames: 0,
            successive: false,
        }),
    )
    .title("car-simu")
    .resizable(false)
    .window_size((800., 800.));
    AppLauncher::with_window(window)
        .launch(Car { inner: car });
}
