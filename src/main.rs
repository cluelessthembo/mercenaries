use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::pass::ClearColor,
    window::CursorMoved,
    input::mouse::{MouseButtonInput, MouseMotion},
    input::keyboard::{ElementState, KeyboardInput},
};

struct Position(f32, f32);
struct Velocity(f32, f32);

fn main() {
    App::build()
    .add_resource(WindowDescriptor {
        title: "Mercenaries v0.0.0".to_string(),
        width: 1600,
        height: 900,
        vsync: true,
        ..Default::default()
    })
    .add_resource(ClearColor(Color::rgb(0.2, 0.2, 0.8)))
    .add_default_plugins()
    .add_plugin(FrameTimeDiagnosticsPlugin::default())
    .add_startup_system(setup.system())
    .add_system(fps_monitor_system.system())
    .add_plugin(PersonPlugin)
    .add_plugin(EncounterPlugin)
    .add_plugin(DrawMovingPlugin)
    .add_plugin(MovingPlugin)
    .add_plugin(ControlPlugin)
    .run();
}

pub struct FPSMeter;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>){
    let font_handle = asset_server.load("assets/fonts/LiberationMono-Regular.ttf").unwrap();

    commands
        // cameras
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default())
        // texture
        .spawn(TextComponents {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text {
                value: "FPS:".to_string(),
                font: font_handle,
                style: TextStyle {
                    font_size: 20.0,
                    color: Color::BLACK,
                },
            },
            ..Default::default()
        })
        .with(FPSMeter);
}

fn fps_monitor_system(diagnostics: Res<Diagnostics>, mut query: Query<(&FPSMeter, &mut Text)>){
    for (_fpsmeter, mut text) in &mut query.iter() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.value = format!("FPS: {:.2}", average);
            }
        }
    }
}

pub struct MovingPlugin;

impl Plugin for MovingPlugin {
    fn build(&self, app: &mut AppBuilder){
        app.add_system(move_system.system());
    }
}

fn move_system(time: Res<Time>, mut query: Query<(&mut Position, &Velocity)>){
    for (mut pos, vel) in &mut query.iter() {
        pos.0 += vel.0 * time.delta_seconds;
        pos.1 += vel.1 * time.delta_seconds;
    }
}

pub struct DrawMovingPlugin;

impl Plugin for DrawMovingPlugin {
    fn build(&self, app: &mut AppBuilder){
        app.add_system(draw_text_system.system());
    }
}

fn draw_text_system(mut query: Query<(&Text, &mut Style, Changed<Position>)>){
    //println!("draw_text");
    for (_text, mut style, pos) in &mut query.iter() {
        //println!("updating");
        style.position.left = Val::Px(pos.0);
        style.position.top = Val::Px(pos.1);
    }
}

pub struct PersonPlugin;

struct Person;

struct Controlled;

struct Label(TextComponents);

impl Label {
    fn new(name: String, font: Handle<Font>, color: Color, font_size: f32) -> Self {
        Label(
            TextComponents {
                text: Text {
                    font: font,
                    value: name,
                    style: TextStyle {
                        color: color,
                        font_size: font_size,
                    },
                },
                style: Style {
                    position_type: PositionType::Absolute,
                    position: Rect {
                        // hide text initially
                        top: Val::Px(-1000.0),
                        left: Val::Px(-1000.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        )
    }
}

impl Plugin for PersonPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .add_startup_system(add_people.system());
    }
}

fn add_people(mut commands: Commands, asset_server: Res<AssetServer>) {

    let font_handle = asset_server.load("assets/fonts/LiberationMono-Regular.ttf").unwrap();

    commands
        .spawn(
            Label::new("P0".to_string(), font_handle.clone(), Color::WHITE, 12.0).0,
        )
        .with(Person)
        .with(Position(100.0, 100.0))
        .with(Velocity(0.0, 0.0))
        .with(Controlled)
        .spawn(
            Label::new("P1".to_string(), font_handle.clone(), Color::WHITE, 12.0).0,
        )
        .with(Person)
        .with(Position(800.0, 500.0))
        .with(Velocity(0.0, 0.0))
        .with(Controlled)
        .spawn(
            Label::new("P3".to_string(), font_handle.clone(), Color::WHITE, 12.0).0,
        )
        .with(Person)
        .with(Position(600.0, 100.0))
        .with(Velocity(0.0, 0.0))
        .with(Controlled)
        .spawn(
            Label::new("P4".to_string(), font_handle.clone(), Color::WHITE, 12.0).0,
        )
        .with(Person)
        .with(Position(1000.0, 300.0))
        .with(Velocity(0.0, 0.0))
        .with(Controlled)
        ;
}

pub struct EncounterPlugin;

struct Hostile;

impl Plugin for EncounterPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(add_hostiles.system());
    }
}

fn add_hostiles(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle = asset_server.load("assets/fonts/LiberationMono-Regular.ttf").unwrap();

    commands
        .spawn(
            Label::new("H0".to_string(), font_handle.clone(), Color::RED, 12.0).0,
        )
        .with(Person)
        .with(Position(200.0, 400.0))
        .with(Velocity(0.0, 0.0))
        .with(Hostile)
        .spawn(
            Label::new("H1".to_string(), font_handle.clone(), Color::RED, 12.0).0,
        )
        .with(Person)
        .with(Position(400.0, 200.0))
        .with(Velocity(0.0, 0.0))
        .with(Hostile)
        ;
}

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut AppBuilder){
        app.init_resource::<InputState>()
        .init_resource::<MouseState>()
        .init_resource::<KeyboardState>()
        .add_system(mouse_input_system.system())
        .add_system(keyboard_input_system.system());
    }
}

#[derive(Default, Debug)]
struct InputState{
    mouse_position: (f32, f32),
    mouse_just_presses: Vec<MouseButton>,
    mouse_presses: Vec<MouseButton>,
    key_presses: Vec<KeyCode>,
}
#[derive(Default)]
struct MouseState {
    mouse_button_event_reader: EventReader<MouseButtonInput>,
    cursor_moved_event_reader: EventReader<CursorMoved>,
}

fn mouse_input_system(mut inputs: ResMut<InputState>, 
    mut state: ResMut<MouseState>, 
    mouse_button_input_events: Res<Events<MouseButtonInput>>, 
    cursor_moved_events: Res<Events<CursorMoved>>) {
    
    inputs.mouse_just_presses.clear();

    for event in state
    .mouse_button_event_reader
    .iter(&mouse_button_input_events)   {
        if event.state == ElementState::Pressed {
            inputs.mouse_presses.push(event.button);
            inputs.mouse_just_presses.push(event.button);
        } else if event.state == ElementState::Released {
            if let Some(index) = inputs.mouse_presses.iter().position(|x| *x == event.button) {
                inputs.mouse_presses.remove(index);
            }
        }
    }

    for event in state
    .cursor_moved_event_reader
    .iter(&cursor_moved_events) {
        inputs.mouse_position.0 = event.position[0];
        inputs.mouse_position.1 = event.position[1];
    }

}

#[derive(Default)]
struct KeyboardState {
    event_reader: EventReader<KeyboardInput>,
}

fn keyboard_input_system(mut inputs: ResMut<InputState>, mut state: ResMut<KeyboardState>, keyboard_input_events: Res<Events<KeyboardInput>>) {
    for event in state.event_reader.iter(&keyboard_input_events) {
        if event.state == ElementState::Pressed {
            if let Some(key) = event.key_code {
                if inputs.key_presses.iter().position(|x| *x == key) == None {
                    inputs.key_presses.push(key)
                }
            }
        }else if event.state == ElementState::Released {
            if let Some(key) = event.key_code {
                if let Some(index) = inputs.key_presses.iter().position(|x| *x == key) {
                    inputs.key_presses.remove(index);
                }
            }
        }
    }
    println!("{:?}", inputs.key_presses);
}