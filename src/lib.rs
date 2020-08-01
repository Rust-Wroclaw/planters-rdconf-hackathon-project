use seed::prelude::*;
use seed::*;
use seed_hooks::*;
use web_sys::OscillatorType;

use seed_style::px; // almost always want seed-style px instead of seed px
use seed_style::{pc, *};
use web_sys::{HtmlCanvasElement, HtmlElement};

mod app;
mod global_styles;
mod sound;
use sound::Sound;
mod rhythm;
use sound::{Tone, ToneBuilder};

mod main_loop;
mod raf_loop;
mod sound_scheduler;
use sound_scheduler::*;

//
//  Model, Msg, Update, init(), and start()
//  ---------------------------------------

pub struct Model {
    current_time_step: u64,
    sound_scheduler: SoundScheduler,
    sound: Tone,
    canvas: ElRef<HtmlCanvasElement>,
}

impl Model {
    pub fn secs_elapsed(&self) -> f64 {
        (self.current_time_step as f64) / 60.0
    }
}

// In aps that make use of conditional rendering on breakpoints we We just need one Msg
// in order to handle a WindowResized event.
#[derive(Clone, Debug)]
pub enum Msg {
    TimeStepAdvanced,
    TimeStepLoopStopped,
    TimeStepLoopStarted,
    Scheduler(SoundSchedulerMsg),
    // WindowResized, // needed for responsive styles
    // NoOp,
    ProduceSound,
    StopSound,
    Click(i32, i32),
}
fn update(msg: Msg, mut model: &mut Model, _orders: &mut impl Orders<Msg>) {
    // log!(msg);    // always worth logging the message in development for debug purposes.
    match msg {
        Msg::TimeStepAdvanced => main_loop::time_step_advanced(&mut model),
        Msg::TimeStepLoopStopped => main_loop::time_step_loop_stopped(&mut model),
        Msg::TimeStepLoopStarted => main_loop::time_step_loop_started(&mut model),
        Msg::Scheduler(msg) => sound_scheduler::update(msg, &mut model),
        Msg::ProduceSound => {
            model.sound.play();
        }
        Msg::StopSound => {
            model.sound.pause();
        }
        Msg::Click(x, y) => {
            let canvas_el = model.canvas.get().unwrap();
            let width = canvas_el.width() as f32;
            let height = canvas_el.height() as f32;

            let el: HtmlElement = canvas_el.into();
            let freq = ((x - el.offset_left()) as f32 * 11_00. / width) as f32;
            let vol = ((y - el.offset_top()) as f32 * 10. / height) as f32;
            model.sound = ToneBuilder::new().gain(vol).freq(freq).build().unwrap();
        }
    }
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    global_styles::global_init();

    let sound = Sound::new()
        .add_tone(ToneBuilder::new().freq(500.0).gain(0.5).build().unwrap())
        .add_tone(ToneBuilder::new().freq(250.0).gain(0.5).build().unwrap());
    let sound = ToneBuilder::new().freq(500.).build().unwrap();
    Model {
        sound,
        canvas: ElRef::<HtmlCanvasElement>::default(),
        current_time_step: 0,
        sound_scheduler: SoundScheduler::default(),
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    let app = App::start("app", init, update, view);
    my_app().set(Some(app));
}

// Provide access to the app incase one wants to force an update from anywhere in the app
#[atom]
fn my_app() -> Atom<Option<App<Msg, Model, Node<Msg>>>> {
    None
}

//  View Entry Here, Sets up theme access, two themes are allowed access
//
//  The first is the app defined theme, the second provides access to seed style presets.
//  The content block also activates themed global_styles.
//  ---------------
pub fn view(model: &Model) -> Node<Msg> {
    raf_loop::raf_loop_atom().get();

    div![
        s().display_grid()
            .grid_template_rows("auto 300px")
            .height(pc(100))
            .width(pc(100)),
        div![
            canvas![
                el_ref(&model.canvas),
                style![
                    St::Border => "1px solid black",
                ],
                mouse_ev(Ev::MouseDown, |event| Msg::Click(event.x(), event.y()))
            ],
            // Where to put the canvas
        ],
        div![
            s().display_grid().grid_template_columns("200px auto"),
            div![
                s().background_color("#000"),
                button!["Select this rhythm", input_ev(Ev::Click, |_| panic!())]
            ],
            div![s().background_color("#F00")],
            div![
                s().background_color("#000"),
                button!["Select this rhythm", input_ev(Ev::Click, |_| panic!())]
            ],
            div![s().background_color("#F00")],
            div![
                s().background_color("#000"),
                button!["Select this rhythm", input_ev(Ev::Click, |_| panic!())]
            ],
            div![s().background_color("#F00")],
            div![
                s().background_color("#000"),
                button!["Select this rhythm", input_ev(Ev::Click, |_| panic!())]
            ],
            div![s().background_color("#F00")],
            div![
                s().background_color("#000"),
                button!["Select this rhythm", input_ev(Ev::Click, |_| panic!())]
            ],
            div![s().background_color("#F00")],
            div![
                s().background_color("#000"),
                button!["Select this rhythm", input_ev(Ev::Click, |_| panic!())]
            ],
            div![s().background_color("#F00")],
            s().flex("0 1 100%"),
            play_controls(model),
            schedule_controls(),
            current_schedule(model),
            s().display_flex().flex_direction_row(),
            div![s().width(px(200)).flex_none(), "Hello World"],
            button!["start", input_ev(Ev::Click, |_| Msg::ProduceSound)],
            button!["stop", input_ev(Ev::Click, |_| Msg::StopSound)],
        ]
    ]
}

#[topo::nested]
pub fn play_controls(model: &Model) -> Node<Msg> {
    div![
        button![
            s().bg_color("green")
                .radius(px(4))
                .color("white")
                .px(px(18))
                .py(px(8))
                .m(px(4))
                .cursor_pointer(),
            "Start Event Loop (aka Play)",
            raf_loop::raf_loop_atom().on_click(|raf| raf.start()),
        ],
        button![
            s().bg_color("red")
                .radius(px(4))
                .color("white")
                .px(px(18))
                .py(px(8))
                .m(px(4))
                .cursor_pointer(),
            "Stop Event Loop (aka Stop)",
            raf_loop::raf_loop_atom().on_click(|raf| raf.stop()),
        ],
        p![
            "Current Time:",
            format!("{:.2}", model.secs_elapsed()),
            ", Current Time step:",
            model.current_time_step
        ]
    ]
}

#[topo::nested]
pub fn schedule_controls() -> Node<Msg> {
    let new_scheduled_sound_name = use_state(|| "".to_string());
    let new_scheduled_sound_time = use_state(|| 0.0 as f64);
    div![
        s().style_children(&["label", "input"]).display_block(),
        s().style_child("input")
            .b_style_solid()
            .b_width(px(1))
            .b_color("gray")
            .radius(px(3))
            .px(px(8))
            .py(px(4)),
        s().style_child("label").px(px(8)).py(px(4)).my(px(8)),
        label!["Add Sound at time:"],
        input![new_scheduled_sound_time.bind(At::Value),],
        label!["With Name"],
        input![
            attrs!(At::Value => new_scheduled_sound_name.get()),
            new_scheduled_sound_name.on_input(|n, inp| *n = inp),
        ],
        button![
            s().bg_color("blue")
                .radius(px(4))
                .color("white")
                .px(px(18))
                .py(px(8))
                .m(px(4))
                .cursor_pointer(),
            "Add Sound",
            mouse_ev(Ev::Click, move |_| {
                let sound_msg = SoundSchedulerMsg::AddSound(
                    new_scheduled_sound_time.get(),
                    new_scheduled_sound_name.get(),
                );

                new_scheduled_sound_time.set(0.0);
                new_scheduled_sound_name.set("".to_string());

                Msg::Scheduler(sound_msg)
            })
        ],
        pre![
            "Time: ",
            new_scheduled_sound_time.get(),
            "secs",
            " , Sound name: ",
            new_scheduled_sound_name.get()
        ],
    ]
}

pub fn current_schedule(model: &Model) -> Node<Msg> {
    div![
        h2!["Scheduled Sounds"],
        ul![model.sound_scheduler.schedule.iter().map(|(ts, sound)| li![
            {
                if sound.played {
                    s().color("green").font_weight_v900()
                } else {
                    s().font_weight_v300()
                }
            },
            "At secs: ",
            (*ts as f64) / 60.0,
            ", At time step: ",
            ts,
            ", Sound name: ",
            &sound.data,
        ])]
    ]
}
