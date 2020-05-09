#[macro_use]
extern crate bitflags;

pub(crate) mod au;
pub(crate) mod controller;
pub(crate) mod twitch;

use crate::au::Lerper;

use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    str::from_utf8,
    sync::{Arc, RwLock},
    thread::spawn,
};

pub(crate) struct State {
    frame: u64,

    stickx: Lerper,
    sticky: Lerper,
    a_button: Lerper,
    b_button: Lerper,
    z_button: Lerper,
    r_button: Lerper,
    start: Lerper,
    c_left: Lerper,
    c_right: Lerper,
    c_up: Lerper,
    c_down: Lerper,
}

pub(crate) type MTState = Arc<RwLock<State>>;

fn main() -> Result<()> {
    pretty_env_logger::try_init()?;
    kankyo::init()?;

    let mut vblank = File::open("vblank")?;
    let mut input = OpenOptions::new().write(true).open("input")?;

    const STICK_LERP_TIME: f64 = 270.0; // 270 frames to lerp stick positions down to 0
    const BUTTON_LERP_TIME: f64 = 20.0; // 20 frames to lerp button inputs down to 0

    let st = {
        let st = State {
            frame: 0,

            stickx: Lerper::init(STICK_LERP_TIME, 127, -128, 0),
            sticky: Lerper::init(STICK_LERP_TIME, 127, -128, 0),
            a_button: Lerper::init(BUTTON_LERP_TIME, 64, -1, 0),
            b_button: Lerper::init(BUTTON_LERP_TIME, 64, -1, 0),
            z_button: Lerper::init(BUTTON_LERP_TIME, 64, -1, 0),
            r_button: Lerper::init(BUTTON_LERP_TIME, 64, -1, 0),
            start: Lerper::init(BUTTON_LERP_TIME / 4.0, 64, -1, 0), // z button is special
            c_left: Lerper::init(BUTTON_LERP_TIME, 64, -1, 0),
            c_right: Lerper::init(BUTTON_LERP_TIME, 64, -1, 0),
            c_up: Lerper::init(BUTTON_LERP_TIME, 64, -1, 0),
            c_down: Lerper::init(BUTTON_LERP_TIME, 64, -1, 0),
        };

        Arc::new(RwLock::new(st))
    };

    info!("ready");

    {
        let st = st.clone();
        spawn(move || twitch::run(st));
    }

    loop {
        let mut data = [0; 3];
        debug!("waiting for vblank");
        vblank.read(&mut data)?;
        let str = from_utf8(&data)?;
        debug!("got data: {}", str);

        let mut controller = [0; 4];

        match str {
            "OK\n" => {
                {
                    let mut data = st.write().unwrap();
                    data.frame += 1;
                }

                let mut data = st.write().unwrap();
                let frame = data.frame + 1;

                //data.stickx.update(data.controller[2] as i64);
                //data.sticky.update(data.controller[3] as i64);
                debug!("x before: {}", data.stickx.scalar);
                let mut stickx_scalar = data.stickx.apply(frame) as i8;
                debug!("x after:  {}", data.stickx.scalar);
                debug!("y before: {}", data.sticky.scalar);
                let mut sticky_scalar = data.sticky.apply(frame) as i8;
                debug!("y after:  {}", data.sticky.scalar);

                let dist = stick_distance(stickx_scalar, sticky_scalar);
                if dist <= 10 {
                    stickx_scalar = 0;
                    sticky_scalar = 0;
                }

                use controller::{HiButtons, LoButtons};

                let mut hi = HiButtons::NONE;
                let mut lo = LoButtons::NONE;
                const BUTTON_PUSH_THRESHOLD: i64 = 2;

                // high buttons
                data.a_button.apply(frame);
                if data.a_button.pressed(BUTTON_PUSH_THRESHOLD) {
                    hi = hi | HiButtons::A_BUTTON;
                }
                data.b_button.apply(frame);
                if data.b_button.pressed(BUTTON_PUSH_THRESHOLD) {
                    hi = hi | HiButtons::B_BUTTON;
                }
                data.z_button.apply(frame);
                if data.z_button.pressed(BUTTON_PUSH_THRESHOLD) {
                    hi = hi | HiButtons::Z_BUTTON;
                }
                data.start.apply(frame);
                if data.start.pressed(BUTTON_PUSH_THRESHOLD) {
                    hi = hi | HiButtons::START;
                }
                data.r_button.apply(frame);
                if data.r_button.pressed(BUTTON_PUSH_THRESHOLD) {
                    lo = lo | LoButtons::R_BUTTON;
                }
                data.c_up.apply(frame);
                if data.c_up.pressed(BUTTON_PUSH_THRESHOLD) {
                    lo = lo | LoButtons::C_UP;
                }
                data.c_down.apply(frame);
                if data.c_down.pressed(BUTTON_PUSH_THRESHOLD) {
                    lo = lo | LoButtons::C_DOWN;
                }
                data.c_left.apply(frame);
                if data.c_left.pressed(BUTTON_PUSH_THRESHOLD) {
                    lo = lo | LoButtons::C_LEFT;
                }
                data.c_right.apply(frame);
                if data.c_right.pressed(BUTTON_PUSH_THRESHOLD) {
                    lo = lo | LoButtons::C_RIGHT;
                }

                debug!(
                    "[      rust] {:02x}{:02x} {:02x}{:02x}",
                    hi.bits(),
                    lo.bits(),
                    stickx_scalar as u8,
                    sticky_scalar as u8
                );
                controller[0] = hi.bits() as u8;
                controller[1] = lo.bits() as u8;
                controller[2] = stickx_scalar as u8;
                controller[3] = sticky_scalar as u8;

                input.write(&controller)?;
            }
            "BYE" => {
                warn!("asked to exit by the game");
                return Ok(());
            }
            _ => {
                error!("got unknown FIFO data {}", str);
                return Err(anyhow!("unknown FIFO data received"));
            }
        };
    }
}

fn stick_distance(x: i8, y: i8) -> i8 {
    let x = (x as f64).powi(2);
    let y = (y as f64).powi(2);
    (x + y).sqrt() as i8
}

#[cfg(test)]
mod test {
    #[test]
    fn stick_distance() {
        for case in [
            (0, 0, 0),
            (127, 0, 127),
            (64, 64, 90),
            (-64, 64, 90),
            (-64, -64, 90),
        ]
        .iter()
        {
            let x = case.0;
            let y = case.1;
            assert_eq!(crate::stick_distance(x, y), case.2);
        }
    }
}
