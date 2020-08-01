use seed::prelude::*;
use seed::*;
use seed_hooks::*;

use seed_style::px; // almost always want seed-style px instead of seed px
use seed_style::*;

mod global_styles;
mod theme;
mod home;
mod app;
mod shared;

mod raf_loop;
mod main_loop;
mod sound_scheduler;
use theme::*;
use sound_scheduler::*;

//
//  Model, Msg, Update, init(), and start()
//  ---------------------------------------

pub struct Model {
    current_time_step: u64,
    sound_scheduler: SoundScheduler,
}

impl Model {
    pub fn secs_elapsed(&self) -> f64 {
        (self.current_time_step as f64) / 60.0
    }
}

// In aps that make use of conditional rendering on breakpoints we We just need one Msg
// in order to handle a WindowResized event.
#[derive(Clone,Debug)]
pub enum Msg {
    TimeStepAdvanced,
    TimeStepLoopStopped,
    TimeStepLoopStarted,
    Scheduler(SoundSchedulerMsg),
    WindowResized, // needed for responsive styles
    NoOp,
}
fn update(msg: Msg, mut model: &mut Model, _orders: &mut impl Orders<Msg>) {
    // log!(msg);    // always worth logging the message in development for debug purposes.
    match msg {
        Msg::TimeStepAdvanced => main_loop::time_step_advanced(&mut model),
        Msg::TimeStepLoopStopped => main_loop::time_step_loop_stopped(&mut model),
        Msg::TimeStepLoopStarted => main_loop::time_step_loop_started(&mut model),
        Msg::Scheduler(msg) => sound_scheduler::update(msg ,  &mut model),
        _ => {}
    }
}


fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    
    orders
        .subscribe(move |subs::UrlChanged(mut url)| {
            let new_page = match url.remaining_path_parts().as_slice() {
                ["home"] => Page::Home,
                ["app"] => Page::App,
                _ =>  Page::Home,
            };

            if current_page().get() != new_page {
                window().scroll_to_with_x_and_y(0., 0.);
                current_page().set(new_page);
            }
            Msg::NoOp
            }
            // 
        )
        .notify(subs::UrlChanged(url));
    // allow app to access the user theme, plus seed style presets.
    // app_themes().set(vec![user_theme(), default_color_theme()]);
    load_app_themes(&[ user_theme, default_colors_theme,default_breakpoint_theme]);
    // themes values can now be used in the global style init
    global_styles::global_init();
    Model {
        current_time_step: 0,
        sound_scheduler:SoundScheduler::default(),
    }

}

#[wasm_bindgen(start)]
pub fn start() {
    let app  = App::start("app", init, update, view);
    my_app().set(Some(app));
}

#[derive(Clone,PartialEq,Debug,Copy)]
enum Page {
    Home,
    App,
}

#[atom]
fn current_page() -> Atom<Page>{
    Page::Home
}

// Provide access to the app incase one wants to force an update from anywhere in the app
#[atom]
fn my_app() -> Atom<Option<App<Msg,Model,Node<Msg>>>>{
    None
}

//  View Entry Here, Sets up theme access, two themes are allowed access
//
//  The first is the app defined theme, the second provides access to seed style presets.
//  The content block also activates themed global_styles.
//  ---------------
#[topo::nested] // Needed for Seed Hooks
pub fn view(model: &Model) -> Node<Msg> {
    raf_loop::raf_loop_atom().get();

    div![
        s().display_flex().flex_direction_row(),
        div![
            s().width(px(200)).flex_none().bg_color(seed_colors::Gray::No4),
            shared::sidebar(model),
        ],
        div![
            s().flex("0 1 100%"),
            play_controls(model),
            schedule_controls(),
            current_schedule(model),
        ]
        
    ]
}

#[topo::nested]
pub fn play_controls(model:&Model) -> Node<Msg>{
    div![
        button![
            s().bg_color("green").radius(px(4)).color("white").px(px(18)).py(px(8)).m(px(4)).cursor_pointer(),
            "Start Event Loop (aka Play)",
            raf_loop::raf_loop_atom().on_click(|raf| raf.start()),
        ],
        button![
            s().bg_color("red").radius(px(4)).color("white").px(px(18)).py(px(8)).m(px(4)).cursor_pointer(),
            "Stop Event Loop (aka Stop)",
            raf_loop::raf_loop_atom().on_click(|raf| raf.stop()),
        ],
        p![
            "Current Time:", format!("{:.2}", model.secs_elapsed()),
            ", Current Time step:", model.current_time_step
        ]
    ]
}


#[topo::nested]
pub fn schedule_controls() -> Node<Msg>{
    let new_scheduled_sound_name = use_state(|| "".to_string() );
    let new_scheduled_sound_time = use_state(|| 0.0 as f64 );
    div![
        s().style_children(&["label","input"]).display_block(),
        s().style_child("input").b_style_solid().b_width(px(1)).b_color("gray").radius(px(3)).px(px(8)).py(px(4)),
        s().style_child("label").px(px(8)).py(px(4)).my(px(8)),
        label![
            "Add Sound at time:"
        ],
        input![
            new_scheduled_sound_time.bind(At::Value),
        ],
        label![
            "With Name"
        ],
        input![
            attrs!(At::Value => new_scheduled_sound_name.get()),
            new_scheduled_sound_name.on_input(|n,inp| *n = inp),
        ],
        button![
            s().bg_color("blue").radius(px(4)).color("white").px(px(18)).py(px(8)).m(px(4)).cursor_pointer(),
            "Add Sound",
            mouse_ev(Ev::Click,move |_| {
                let sound_msg =  SoundSchedulerMsg::AddSound(new_scheduled_sound_time.get(), new_scheduled_sound_name.get());
             
                new_scheduled_sound_time.set(0.0);
                new_scheduled_sound_name.set("".to_string());
             
                Msg::Scheduler(
                   sound_msg
                )
            })
        ],
        pre!["Time: ", new_scheduled_sound_time.get(), "secs", " , Sound name: ",new_scheduled_sound_name.get() ],
    ]
}


pub fn current_schedule(model :&Model) -> Node<Msg> {
    div![
        h2!["Scheduled Sounds"],

        ul![
            model.sound_scheduler.schedule.iter().map(|(ts,sound)|  
                li![{
                    if sound.played {
                        s().color("green").font_weight_v900()
                    } else {
                        s().font_weight_v300()
                    }
                    }
                    ,
                    "At secs: ", (*ts  as f64)/ 60.0, ", 
                    At time step: ",ts, ", 
                    Sound name: ", &sound.data,
                ]
            )
        ]
    ]
    
}

