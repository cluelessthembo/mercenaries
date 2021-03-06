// imports from bevy engine
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::pass::ClearColor,
    window::CursorMoved,
    input::mouse::{MouseButtonInput},
    input::keyboard::{ElementState, KeyboardInput},
};
// imports for the easing functions
use ezing;
// imports for id generation
use uuid::Uuid;
// imports for data structures
use std::collections::{VecDeque, HashMap};
// imports for random number generator
use rand::Rng;
// imports for noise generator
use noise::{NoiseFn, Perlin, Seedable};
// imports for reading file
use std::fs;
// imports for rapier2d bevy plugins
use bevy_rapier2d::physics::RapierPhysicsPlugin;
use bevy_rapier2d::render::RapierRenderPlugin;
// imports for pathfinding
use pathfinding::prelude::astar;
// settings for window width/height
static WINDOW_WIDTH: f32 = 800.0;
static WINDOW_HEIGHT: f32 = 450.0;
static TILE_SIZE: f32 = 10.0;
static PLAYER_Z_LEVEL: f32 = 10.0;
static MAP_PATH: &str = "assets/maps/ortho-map.tmx";
static MAX_PATHFINDERS: usize = 10;

// imports for bevy_tiled
use bevy_tiled;
// imports for ordered_float
use ordered_float::OrderedFloat;
// id component
// this should be spawned along side every entity
// it is responsible for keeping the unique id of each entity
// it is essentially impossible to keep track of an entity otherwise
// note that bevy has its own id system for entities but it uses a u32 id
// which carries the risk of collision
struct Id(String);

// id component implementation
impl Id {
    // new function generates new uuid
    fn new() -> Self {
        Id(
            // uuid removes all hyphens 
            Uuid::new_v4().to_simple().to_string()
        )
    }
    // id function returns id as a string
    fn id(&self) -> String {
        self.0.clone()
    }
}
// position component
// spawn this component along with any entity that has a physical position on the screen
struct Position(f32, f32);

// size component
// spawn this component along with any entity that should have collision turned on
struct Size(f32, f32);

// velocity component
// spawn this component along with any entity that has a physical velocity
struct Velocity(f32, f32);

// main function, this is what cargo run runs
fn main() {
    App::build()
    // details about the window, 
    // including the title, and the dimensions
    .add_resource(WindowDescriptor {
        title: "Mercenaries v0.0.1".to_string(),
        width: WINDOW_WIDTH as u32,
        height: WINDOW_HEIGHT as u32,
        vsync: false,
        ..Default::default()
    })
    // resource used to determine background colour of window
    .add_resource(ClearColor(Color::rgb(0.2, 0.2, 0.8)))
    // adds useful plugins for making a game
    .add_default_plugins()
    // add in physics plugins
    .add_plugin(RapierPhysicsPlugin)
    .add_plugin(RapierRenderPlugin)
    // add in the fps diagnostics plugin
    .add_plugin(FrameTimeDiagnosticsPlugin::default())
    // perform initial setup
    .add_startup_system(setup.system())
    // add in the fps counter system
    .add_system(fps_monitor_system.system())
    // add in the map plugin
    .add_plugin(MapPlugin)
    // add in the person plugin
    .add_plugin(PersonPlugin)
    // add in the encounter plugin
    .add_plugin(EncounterPlugin)
    // add in the draw plugin for moving objects
    .add_plugin(DrawMovingPlugin)
    // add in the moving plugin
    .add_plugin(MovingPlugin)
    // add in the player control plugin
    .add_plugin(ControlPlugin)
    // add in the actions plugin - lower level of control for entities
    .add_plugin(ActionsPlugin)
    // add in the animations plugin
    .add_plugin(AnimationPlugin)
    // add in the behaviour plugin
    .add_plugin(BehaviourPlugin)
    // add in bevy_tiled's TiledMap plugin
    .add_plugin(bevy_tiled::TiledMapPlugin)
    // run the app
    .run();
}

// fps counter component
// spawn this component along any text components that will be used as fps counters 
pub struct FPSMeter;

// initial setup function, 
// spawn in necessary entities (cameras)
// along with fps counter
fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>){
    let font_handle = asset_server.load("assets/fonts/LiberationMono-Regular.ttf").unwrap();

    // add in tile map
    commands
        .spawn(bevy_tiled::TiledMapComponents {
            map_asset: asset_server.load(MAP_PATH).unwrap(),
            center: true,
            ..Default::default()
        });

    commands
        // cameras
        .spawn(Camera2dComponents::default())
        .spawn(UiCameraComponents::default())
        // text for fps counter
        .spawn(TextComponents {
            style: Style {
                // for alignment of the text
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text {
                // the content of the text
                value: "FPS:".to_string(),
                // the font used to display it
                font: font_handle,
                // styling for the text, including font size and color
                style: TextStyle {
                    font_size: 20.0,
                    color: Color::BLACK,
                },
            },
            ..Default::default()
        })
        // make sure to spawn fps meter component so it displays fps
        .with(FPSMeter);
}

// fps counter system
fn fps_monitor_system(diagnostics: Res<Diagnostics>, mut query: Query<(&FPSMeter, &mut Text)>){
    for (_fpsmeter, mut text) in &mut query.iter() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.value = format!("FPS: {:.2}", average);
            }
        }
    }
}
// moving plugin 
// this plugin is in charge of moving everything with both a position component
// and a velocity component
pub struct MovingPlugin;

// implementation of the plugin trait,
// required for this to be used as a plugin
impl Plugin for MovingPlugin {
    fn build(&self, app: &mut AppBuilder){
        // add in the move system
        app.add_system(move_system.system());
    }
}

// move system
// this function goes through all entities with both position and velocity components
// and moves them
fn move_system(time: Res<Time>, mut query: Query<(&mut Position, &Velocity)>){
    for (mut pos, vel) in &mut query.iter() {
        // adjust the amount moved by the time passed since last tick - this keeps
        // movement consistent despite inconsistent fps
        pos.0 += vel.0 * time.delta_seconds;
        pos.1 += vel.1 * time.delta_seconds;
    }
}
// draw moving plugin
// this plugin updates everything drawable to their correct positions
// drawing itself happens within the bevy engine
pub struct DrawMovingPlugin;

// implementation of the plugin trait,
// required for this to be used as a plugin
impl Plugin for DrawMovingPlugin {
    fn build(&self, app: &mut AppBuilder){
        // add in the draw text system
        app.add_system(draw_text_system.system())
            .add_system(draw_sprite_system.system());
    }
}

// draw text system
// this function goes through all entities with text, style, and position components
// and updates the style component to reflect the correct position of the entity
// note that this only happens when any of those position components are changed
fn draw_text_system(mut query: Query<(&Text, &mut Style, &Position)>){
    for (_text, mut style, pos) in &mut query.iter() {
        // update the style component to have the correct position on the screen
        style.position.left = Val::Px(pos.0);
        style.position.top = Val::Px(pos.1);
    }
}

// function to get the correct translation coordinates from a given position
fn get_translate_from_position(x: f32, y: f32) -> (f32, f32) {
    // translation has (0, 0) at the center of the screen
    // it also has the y-coordinates increase from bottom to top
    // we must invert the y-coordinates to use the right scale, then
    // we must shift the position coordinates towards the 
    // upper left corner of the screen by half the 
    // screen dimensions
    (x - WINDOW_WIDTH / 2.0, (WINDOW_HEIGHT - y) - WINDOW_HEIGHT / 2.0)
}

// draw sprite system
// responsible for moving sprites to their proper positions for 
// display
fn draw_sprite_system(mut query: Query<(&Sprite, &mut Translation, &Position)>){
    for (_sprite, mut transl, pos) in &mut query.iter() {
        
        // get the proper coordinates for translation
        let adj_pos = get_translate_from_position(pos.0, pos.1);

        // assign coordinates
        transl.0 = Vec3::new(adj_pos.0, adj_pos.1, transl.0[2]);
    }
}
// person plugin
// adds in all the people
pub struct PersonPlugin;
// person component
// spawn this component along with any entity that should be considered a person

enum AttitudeType {
    Neutral,
    Squad,
    Hostile,
    Ally,
}

struct Person {
    attitude: AttitudeType,
}

impl Default for Person {
    fn default() -> Self {
        Person {
            attitude: AttitudeType::Neutral,
        }
    }
}

impl Person {
    fn new(att: AttitudeType) -> Self {
        Person {
            attitude: att,
        }
    }
}

// controlled component
// spawn this component along with any entity that should be considered controlled by the player
#[derive(Default)]
struct Controlled {
    // current command given to this entity
    current_command: Command,
    // index in the squad
    squad_pos: i32,
    // command queue 
    // currently not under use, new commands 
    // will replace old commands instead 
    // of queueing up
    command_queue: VecDeque<Command>,
}

impl Controlled {
    // initialize a new Controlled struct
    fn new(i: i32) -> Self {
        Controlled {
            // initialized with an empty command
            current_command: Command::default(),
            // needs to be an assigned a squad index
            squad_pos: i,
            // initialized with an empty command queue
            command_queue: VecDeque::new(),
        }
    }
}

// struct that represents a command
#[derive(Default)]
struct Command {
    // requires a command type
    command_type: CommandType,
    // target is the id of the target entity, if it exists
    target_id: Option<String>,
    // point is the target coordinate, if it exists
    target_point: Option<(f32, f32)>,
}
// Nerve component
// holds the current action as well as succeeding actions
struct Nerve {
    // current action is the action being worked on
    current_action: Action,
    // action queue holds the next actions, after the current one
    action_queue: VecDeque<Action>,
    // action timer allows for a sense of realtime
    action_timer: Option<Timer>,
}

impl Nerve {
    // function to initialise a new Nerve component
    fn new() -> Self {
        Nerve {
            // initialise with an empty action
            current_action: Action::default(),
            // initialise with an empty action queue
            action_queue: VecDeque::new(),
            // initialise with None
            action_timer: None,
        }
    }
    fn is_curr_action_empty(&self) -> bool {
        match self.current_action.action_type {
            ActionType::Empty => {
                true
            },
            _ => {
                false
            },
        }
    }
}

// enum for the type of action 
#[derive(Debug, Clone, Copy)]
enum ActionType {
    // move actions will move entities to a stationary point
    // or a moving entity
    // additional parameters include:
    // range: maximum distance from target allowable
    // min_range: minimum distance from target allowable
    Move,
    // attack actions will launch attacks at an entity until it 
    // ceases to become hostile
    Attack,
    // wait actions will do nothing for a specified amount of time
    Wait,
    // empty actions do nothing and are immediately popped
    Empty,
}

// action struct
#[derive(Debug, Clone)]
struct Action {
    action_type: ActionType,
    // target is either a coordinate point or an entity id
    target: (Option<(f32, f32)>, Option<String>),
    // params contains additional parameters
    // add additional parameters to an action by inserting
    // a string key with a f32 value
    params: Option<HashMap<String, f32>>,
}

impl Default for Action {
    // default function for action
    // gives a default Action, which is an empty action
    fn default() -> Self {
        Action {
            action_type: ActionType::Empty,
            target: (None, None),
            params: None,
        }
    }
}

// labels are currently not under use
/*
// label struct
// this struct is a convenient way of generating the necessary text components to label an entity
struct Label;

//implementation for the label struct
impl Label {
    // to generate a label, do Label::new(<name>, <font>, <color>, <font size>).0
    // the '.0' accesses the inner TextComponents, which is what you really need
    fn new(name: String, font: Handle<Font>, color: Color, font_size: f32) -> TextComponents {
        TextComponents {
            text: Text {
                // note that the font should be a handle to a font asset loaded in by the asset server
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
                    // hide text initially by sending it way out of bounds until
                    // its position is updated
                    top: Val::Px(-1000.0),
                    left: Val::Px(-1000.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
*/

// helper struct for a simple rectangle sprite
struct SimpleRect;

impl SimpleRect {
    // this function gives the necessary components for a rectanglular sprite
    // takes a color handle, and a size vector and returns SpriteComponents
    fn new(color_handle: Handle<ColorMaterial>, size: Vec2) -> SpriteComponents {
        SpriteComponents {
            material: color_handle,
            // move sprite off screen
            translation: Translation(Vec3::new(-1000.0, -1000.0, PLAYER_Z_LEVEL)),
            sprite: Sprite {
                size: size,
            },
            ..Default::default()
        }
    }
}

// implementation of the plugin trait,
// required for this to be used as a plugin
impl Plugin for PersonPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // add in the add people startup system 
        app
        .add_startup_system(add_people.system());
    }
}

// add people startup system
// this function runs once at the initialization of the plugin to add in six people
fn add_people(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>) {

    // this is a handle to a font asset, loaded in by the asset server from the local directory
    //let font_handle = asset_server.load("assets/fonts/LiberationMono-Regular.ttf").unwrap();

    let green_handle = materials.add(Color::GREEN.into());
    let blue_handle = materials.add(Color::BLUE.into());


    commands
        // spawn in text components for use as label
        .spawn(
            //Label::new("P0".to_string(), font_handle.clone(), Color::WHITE, 12.0),
            SimpleRect::new(green_handle, Vec2::new(10.0, 10.0))
        )
        .with(Id::new())
        // spawn person component along with to signify that this entity is a person
        .with(Person::new(AttitudeType::Squad))
        // spawn position component along with so that this entity has a physical position
        .with(Position(100.0, 100.0))
        // spawn velocity component along with so that this entity has a physical velocity and can move
        .with(Velocity(0.0, 0.0))
        // spawn controlled component along with so that this entity is controlled by the player
        .with(Controlled::new(0))
        .with(Nerve::new())
        .with(Size(10.0, 10.0))
        .with(Pathfinder::default())

        .with(get_player_sprite_template(&mut materials))
        // same deal for the other three persons
        // note however that only the first has the controlled component
        // we will consider that entity our player character
        .spawn(
            //Label::new("P1".to_string(), font_handle.clone(), Color::WHITE, 12.0),
            SimpleRect::new(blue_handle, Vec2::new(10.0, 10.0))
        )
        .with(Id::new())
        .with(Person::new(AttitudeType::Squad))
        .with(Position(200.0, 400.0))
        .with(Velocity(0.0, 0.0))
        .with(Controlled::new(1))
        .with(Nerve::new())
        .with(Size(10.0, 10.0))
        .with(Behaviour::default())
        .with(get_squadmate_sprite_template(&mut materials))
        .with(Pathfinder::default())

        .spawn(
            //Label::new("P3".to_string(), font_handle.clone(), Color::WHITE, 12.0),
            SimpleRect::new(blue_handle, Vec2::new(10.0, 10.0))
        )
        .with(Id::new())
        .with(Person::new(AttitudeType::Squad))
        .with(Position(600.0, 100.0))
        .with(Velocity(0.0, 0.0))
        .with(Controlled::new(2))
        .with(Nerve::new())
        .with(Size(10.0, 10.0))
        .with(Behaviour::default())
        .with(get_squadmate_sprite_template(&mut materials))
        .with(Pathfinder::default())

        .spawn(
            //Label::new("P4".to_string(), font_handle.clone(), Color::WHITE, 12.0),
            SimpleRect::new(blue_handle, Vec2::new(10.0, 10.0))
        )
        .with(Id::new())
        .with(Person::new(AttitudeType::Squad))
        .with(Position(500.0, 100.0))
        .with(Velocity(0.0, 0.0))
        .with(Controlled::new(3))
        .with(Nerve::new())
        .with(Size(10.0, 10.0))
        .with(Behaviour::default())
        .with(get_squadmate_sprite_template(&mut materials))
        .with(Pathfinder::default());
}
// encounter plugin
// responsible for generating encounters for the player
pub struct EncounterPlugin;

// implementation of the plugin trait,
// required for this to be used as a plugin
impl Plugin for EncounterPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // add in add hostiles start up system
        app.add_startup_system(add_hostiles.system());
    }
}

// add hostiles start up system
// this function adds in some hostiles
fn add_hostiles(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>) {
    // this is a font handle, a handle to a font asset loaded in by the asset server from the local directory
    //let font_handle = asset_server.load("assets/fonts/LiberationMono-Regular.ttf").unwrap();

    let black_handle = materials.add(Color::BLACK.into());
    
    commands
        // spawn in text components for the label
        .spawn(
            //Label::new("H0".to_string(), font_handle.clone(), Color::RED, 12.0),
            SimpleRect::new(black_handle, Vec2::new(10.0, 10.0)),
        )
        .with(Id::new())
        // spawn along the person component to signify that this entity is a person
        .with(Person::new(AttitudeType::Hostile))
        // spawn along the position component so that this entity has a physical position on the screen
        .with(Position(200.0, 400.0))
        // spawn along the velocity component so that this entity has a physical velocity and can move
        .with(Velocity(0.0, 0.0))
        .with(Nerve::new())
        .with(Size(10.0, 10.0))
        .with(Behaviour::default())
        .with(get_hostile_sprite_template(&mut materials))
        .with(Pathfinder::default())
        // repeat for another hostile entity
        .spawn(
            SimpleRect::new(black_handle, Vec2::new(10.0, 10.0)),
        )
        .with(Id::new())
        .with(Person::new(AttitudeType::Hostile))
        .with(Position(400.0, 200.0))
        .with(Velocity(0.0, 0.0))
        .with(Nerve::new())
        .with(Size(10.0, 10.0))
        .with(Behaviour::default())
        .with(get_hostile_sprite_template(&mut materials))
        .with(Pathfinder::default())
        ;
}
// control plugin
// responsible for reading player inputs from the mouse and keyboard
pub struct ControlPlugin;

// implementation of the plugin trait,
// required for this to be used as a plugin
impl Plugin for ControlPlugin {
    fn build(&self, app: &mut AppBuilder){
        // initialise the inputstate resource
        app.init_resource::<InputState>()
        // initialise the mousestate resource
        .init_resource::<MouseState>()
        // initialise the keyboardstate resource
        .init_resource::<KeyboardState>()
        // add in the mouse input system
        .add_system(mouse_input_system.system())
        // add in the keyboard input system
        .add_system(keyboard_input_system.system())
        // add in the move player system
        .add_system(move_controlled_system.system())
        // add in the control player system
        .add_system(player_control_system.system());
    }
}
// the inputstate struct is what we will read in the rest
// of the program to determine if a player is pressing a certain input or not
#[derive(Default, Debug)]
struct InputState{
    // mouse_position holds the location of the cursor
    mouse_position: (f32, f32),
    // mouse_just_presses holds which mouse buttons were JUST pressed
    mouse_just_presses: Vec<MouseButton>,
    // mouse_presses holds which mouse buttons are currently pressed
    mouse_presses: Vec<MouseButton>,
    // key_presses holds which keys are currently pressed
    key_presses: Vec<KeyCode>,
}
// the mousestate struct holds event readers for the mousebutton events and cursormoved events
#[derive(Default)]
struct MouseState {
    mouse_button_event_reader: EventReader<MouseButtonInput>,
    cursor_moved_event_reader: EventReader<CursorMoved>,
}

// mouse input system
// this function reads the input coming from the mouse and stores it in InputState for use in other parts
// of the program
fn mouse_input_system(mut inputs: ResMut<InputState>, 
    mut state: ResMut<MouseState>, window: Res<WindowDescriptor>,
    mouse_button_input_events: Res<Events<MouseButtonInput>>, 
    cursor_moved_events: Res<Events<CursorMoved>>) {
    
    // clear the mouse_just_presses vector so that we only capture the most recent button inputs
    inputs.mouse_just_presses.clear();

    for event in state
    .mouse_button_event_reader
    .iter(&mouse_button_input_events)   {
        // if a mouse button is pressed
        if event.state == ElementState::Pressed {
            // add this to the mouse_presses and mouse_just_presses vectors
            // no need to check if they're already present because mouse buttons must be released before they can be pressed again
            inputs.mouse_presses.push(event.button);
            inputs.mouse_just_presses.push(event.button);

        // if a mouse button is released 
        } else if event.state == ElementState::Released {
            // remove it from the mouse_presses vector
            if let Some(index) = inputs.mouse_presses.iter().position(|x| *x == event.button) {
                inputs.mouse_presses.remove(index);
            }
        }
    }

    // reads CursorMoved events to get the position of the cursor relative to the window
    for event in state
    .cursor_moved_event_reader
    .iter(&cursor_moved_events) {
        // this is where we set the mouse position from the cursor position
        inputs.mouse_position.0 = event.position[0];
        // convert the cursormoved event coordinates to mouse position coordinates we can use 
        inputs.mouse_position.1 = window.height as f32 - event.position[1];
    }
}
// keyboardstate holds an event reader for key presses from the keyboard
#[derive(Default)]
struct KeyboardState {
    event_reader: EventReader<KeyboardInput>,
}

// keyboard input system
// this system captures input from the keyboard and stores it in inputstate
fn keyboard_input_system(mut inputs: ResMut<InputState>, mut state: ResMut<KeyboardState>, keyboard_input_events: Res<Events<KeyboardInput>>) {
    for event in state.event_reader.iter(&keyboard_input_events) {
        // if a key is pressed
        if event.state == ElementState::Pressed {
            // check if its keycode exists
            if let Some(key) = event.key_code {
                // check if it's not already in the key_presses vector
                // note that holding down a key will send multiple keypressed events in succession
                if inputs.key_presses.iter().position(|x| *x == key) == None {
                    // add it into the key_presses vector
                    inputs.key_presses.push(key)
                }
            }
        // if a key is released
        }else if event.state == ElementState::Released {
            // check if its keycode exists
            if let Some(key) = event.key_code {
                // check if it's in the key_presses vector
                if let Some(index) = inputs.key_presses.iter().position(|x| *x == key) {
                    // reomve it from the key_presses vector
                    inputs.key_presses.remove(index);
                }
            }
        }
    }
}

// move controlled system
// responsible for calculating the velocity vector of the player to get to
// the desired move point and setting the player character's velocity
fn move_controlled_system(mut query: Query<(&mut Controlled, &mut Nerve)>) {
    for (mut state, mut actions) in &mut query.iter() {
        let command = &state.current_command;
        
        match command.command_type {
            CommandType::Move => {
                // clear current actions to replace with new actions
                actions.current_action = Action::default();
                actions.action_queue.clear();

                let mut params = HashMap::new();
                // range refers to the maximum range acceptable
                // set at zero to force entity to move to the target location
                params.insert("range".to_string(), 0.0);

                // add move action to the target location
                actions.action_queue.push_back(Action {
                    action_type: ActionType::Move,
                    target: (command.target_point, None),
                    params: Some(params.clone()),
                });
            },
            CommandType::Attack => {
                // clear current actions to replace with new actions
                actions.current_action = Action::default();
                actions.action_queue.clear();

                let mut params = HashMap::new();
                // range refers to the maximum range at which an attack can be launched
                params.insert("range".to_string(), 40.0);
                // min_range refers to the minimum range at which an attack can be launched
                params.insert("min_range".to_string(), 20.0);

                // add move action to the target entity
                // get within a certain distance of the target
                actions.action_queue.push_back(Action {
                    action_type: ActionType::Move,
                    target: (None, command.target_id.clone()),
                    params: Some(params.clone()),
                });

                // add attack action
                // attack the target
                actions.action_queue.push_back(Action {
                    action_type: ActionType::Attack,
                    target: (None, command.target_id.clone()),
                    params: Some(params),
                });
            },
            CommandType::Flee => {
                // clear current actions to replace with new actions
                actions.current_action = Action::default();
                actions.action_queue.clear();

                let mut params = HashMap::new();
                // min_range refers to the minimum distance that we want to put between
                // us and the point/entity
                params.insert("min_range".to_string(), 200.0);
                // add move action to the target entity
                // get away from a certain distance of the target
                actions.action_queue.push_back(Action {
                    action_type: ActionType::Move,
                    target: (command.target_point, command.target_id.clone()),
                    params: Some(params.clone()),
                });
            },
            CommandType::Follow => {
                // clear current actions to replace with new actions
                actions.current_action = Action::default();
                actions.action_queue.clear();

                let mut params = HashMap::new();
                // range refers to the maximum range acceptable
                params.insert("range".to_string(), 40.0);
                params.insert("no_skip".to_string(), 1.0);

                // add move action to the target entity
                actions.action_queue.push_back(Action {
                    action_type: ActionType::Move,
                    target: (None, command.target_id.clone()),
                    params: Some(params.clone()),
                });
            },
            _ => {

            },
        }

        // pop the command queue and ready the next command
        if let Some(command) = state.command_queue.pop_front() {
            // if there are more commands in the command queue
            // set it to be the current command
            state.current_command = command;
        }else{
            // if there are no more commands in the command queue
            // set the current command to be the empty command
            state.current_command = Command::default();
        }   
    }
}

// function to convert the keys pressed to the squad indices they're mapped to
fn convert_keycode_to_squad_pos(key: KeyCode) -> i32 {
    match key {
        KeyCode::Key0 => 0,
        KeyCode::Key1 => 1,
        KeyCode::Key2 => 2,
        KeyCode::Key3 => 3,
        KeyCode::Key4 => 4,
        KeyCode::Key5 => 5,
        KeyCode::Key6 => 6,
        KeyCode::Key7 => 7,
        KeyCode::Key8 => 8,
        KeyCode::Key9 => 9,
        // if this is not a valid mapping, return -1
        _ => -1
    }
}

// this function checks if a point is in the a box at a certain position with a certain 'radius'
// point is the coordinate being checked
// box_position is the coordinate where the box is
// box_size is the size of the box
fn check_point_collision(point: (f32, f32), box_position: (f32, f32), box_size: (f32, f32)) -> bool {
    let box_radius = (box_size.0 / 2.0, box_size.1 / 2.0);
    if (point.0 < box_position.0 + box_radius.0) && (point.0 > box_position.0 - box_radius.0)
        && (point.1 < box_position.1 + box_radius.1) && (point.1 > box_position.1 - box_radius.1) {
        true
    }else{
        false
    }
}

// enum for the command type
#[derive(Copy, Clone)]
enum CommandType {
    // move command orders a pawn to move to a certain spot
    Move,
    // attack command orders a pawn to attack a certain entity
    Attack,
    // flee command orders a pawn to move a certain distance away from a certain entity/spot
    Flee,  
    // follow command orders a pawn to follow another  
    Follow,
    // empty command does nothing
    Empty,
}

// implementation for the command type enum
impl Default for CommandType {
    // default function for the command type
    // returns the empty command type
    fn default() -> Self {
        CommandType::Empty
    }
}


// player control system
// responsible for translating all inputs into the respective actions in-game
fn player_control_system(inputs: Res<InputState>, mut controlstate: Query<&mut Controlled>, mut persons: Query<(&Id, &Person, &Position, &Size)>) {
    // if the left mouse button was just pressed
    if inputs.mouse_just_presses.contains(&MouseButton::Left) {
        
        // if the left mouse button was clicked, default to a move command
        let mut command_type = CommandType::Move;
        // a move command by default has no target entity
        let mut target_entity = None;

        // check if you clicked on something
        for (id, pers, pos, size) in &mut persons.iter() {
            // check if an entity was clicked
            if check_point_collision(inputs.mouse_position, (pos.0, pos.1), (size.0, size.1)) {
                // if an entity was clicked

                // check attitude of person clicked
                match &pers.attitude {
                    // if attitude is hostile
                    AttitudeType::Hostile => {
                        // switch command type to an attack type    
                        command_type = CommandType::Attack;
                    },
                    AttitudeType::Neutral => {
                        // switch command type to an attack type    
                        command_type = CommandType::Attack;
                    },
                    AttitudeType::Squad => {
                        // switch command type to an attack type    
                        command_type = CommandType::Follow;
                    },
                    _ => {}
                }
                
                // set the target entity to the entity clicked
                target_entity = Some(id.id());
                // once target entity is found, break out of the loop
                break;
            }
        }

        // check hotkeys pressed
        // left shift switches move/follow/attack -> flee
        if inputs.key_presses.contains(&KeyCode::LShift) {
            command_type = CommandType::Flee;
        // left control switches move/attack -> follow
        // left control takes precedence over left shift
        } else if inputs.key_presses.contains(&KeyCode::LControl) && target_entity.is_some() {
            command_type = CommandType::Follow;
        }
        

        // squad_control vector contains all the squad indices being ordered
        let mut squad_control = Vec::new();
        
        // check which hotkeys are being pressed
        for key in &mut inputs.key_presses.iter() {
            // attempt to convert the hotkey keycode to the corresponding squad index
            let squad_pos = convert_keycode_to_squad_pos(*key);
            // if the squad index is valid then add it to the squad_control vector
            if squad_pos >= 0 {
                squad_control.push(squad_pos);
            }
        }

        // check if squad_control is empty
        if squad_control.is_empty() {
            // if no squad keys are pressed, assume controls are for player
            squad_control.push(0);
        }

        // go through all the controlled components
        for mut state in &mut controlstate.iter() {
            // if this controlled component is one of the ones being commanded
            if squad_control.contains(&state.squad_pos) {
                
                // note that current behaviour is to replace the current command
                // at some point we may want the capability to queue up multiple commands
                // check the command type
                match command_type {
                    // if the command type is move
                    CommandType::Move => {
                        // set the current command to a move type command
                        // towards the cursor position
                        state.current_command = Command {
                            command_type: command_type,
                            target_point: Some(inputs.mouse_position.clone()),
                            target_id: None,
                        };
                    },
                    // if the command type is attack
                    CommandType::Attack => {
                        // set the current command to an attack type command
                        // at the entity clicked
                        state.current_command = Command {
                            command_type: command_type,
                            target_point: None,
                            target_id: target_entity.clone(),
                        };
                    },
                    // if the command type is flee
                    CommandType::Flee => {
                        // set the current command to a flee type command
                        // at what is clicked
                        state.current_command = Command {
                            command_type: command_type,
                            target_point: Some(inputs.mouse_position.clone()),
                            target_id: target_entity.clone(),
                        };
                    },
                    // if the command type is follow
                    CommandType::Follow => {
                        // set the current command to a follow type command
                        // at what is clicked
                        state.current_command = Command {
                            command_type: command_type,
                            target_point: None,
                            target_id: target_entity.clone(),
                        }
                    }
                    // if the command type is empty
                    CommandType::Empty => {
                        // set the current command to an empty type command
                        state.current_command = Command {
                            command_type: command_type,
                            target_point: None,
                            target_id: None,
                        }
                    },
                    _ => {

                    },
                }
                
                
            }
        }
    }
}

// actions plugin
// responsible for implementing/managing an interface that allows for lower level control of entities
struct ActionsPlugin;

// boilerplate code for the plugin
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut AppBuilder){
        // add in the run action system
        app.add_system(run_action_system.system());
    }
}

fn get_straightline_velocity(target: (f32, f32), curr: (f32, f32)) -> Vec2 {
    // get the distance vector from the player to the move point
    let dist_vector = Vec2::new(target.0 - curr.0, target.1 - curr.1);
    // the length of the distance vector is the distance between the two points
    let dist = dist_vector.length();
    // if distance is 0 then the velocity vector is 0
    let mut new_vel = Vec2::new(0.0, 0.0);
    // otherwise if distance is greater than 0
    if dist > 0.0 {
        // divide the distance by the distance factor and cap at 1.0
        let ease_input = (dist / 137.5).min(1.0);

        // new velocity vector is a rescaled exponential applied to the normalized distance vector
        // the result is that speed is based on distance and varies according to an exponential curve
        // and the velocity is always towards the move point
        // if pathfinding is implemented for the player, then this will need to be changed
        new_vel = ezing::expo_out( ease_input ) * 137.5 * dist_vector.normalize();
    }

    // if the new x-velocity has insignificant magnitude,
    // just set x-vel to the distance to target x
    // this is to avoid sliding
    if new_vel[0].abs() < 1.0 {
        new_vel[0] = dist_vector[0];        
    }
    // if the new y-velocity has insignificant magnitude,
    // just set y-vel to the distance to target y
    // this is to avoid sliding
    if new_vel[1].abs() < 1.0 {
        new_vel[1] = dist_vector[1];
    }
    
    new_vel
}

// close enough function
// checks if two floats are within a certain threshold of each other
// this function should be used whenever we want to measure distances and 
// establish distance thresholds, due to the way that velocity and moving 
// is implemented. otherwise, we have an achilles and the tortoise situation
// where an entity will asymptotically approach a border but never actually 
// reach it
fn close_enough (x: f32, y: f32, enough: f32) -> bool {
    if (x - y).abs() < enough {
        true
    }else{
        false
    }
}

// run action system
// responsible for implementing the various actions used for lower level control of entities
fn run_action_system(time: Res<Time>, mut query: Query<(&mut Nerve, &Id, &Position, &mut Velocity, &mut SpriteData)>, mut ent_query: Query<(&Id, &Position)>) {
    // go through all entities with a brain, position, and velocity
    for (mut actions, id, pos, mut vel, mut sprite) in &mut query.iter() {
        // get the current action
        let action = actions.current_action.clone();

        // check the action type
        match action.action_type {
            // move actions will move the entity to a stationary point
            ActionType::Move => {
                
                sprite.animation_type = AnimationType::Move;

                let mut move_to = (f32::NAN, f32::NAN);


                match action.target {
                    (_, Some(tid)) => {
                        // check if the targeted id is the same as this id
                        if tid == id.id() {
                            // if this move action is self-targeted
                            // skip it (it's pointless)
                            // this is important to do in cases where
                            // ordinarily the move isn't popped when at rest
                            // e.g. follow commands
                            
                            // pop actions queue and ready next action
                            // check if there are still actions in the action queue
                            if let Some(action) = actions.action_queue.pop_front() {
                                // if there are still more actions
                                // set the next action to be the current action
                                actions.current_action = action;
                            }else{
                                // if there are no more actions in the action queue
                                // set the current action to be the empty action
                                actions.current_action = Action::default();
                            }                              
                        }
                        for (eid, pos) in &mut ent_query.iter() {
                            // check if the id matches
                            if tid == eid.id() {
                                // set the target position
                                move_to = (pos.0, pos.1);
                                break;
                            }
                        }
                    },
                    (Some(target), None) => {
                        move_to = target;
                    },
                    (None, None) => {
                        // no target given
                        // should warn user that action is invalid then skip
                        // need warning system, currently just aborts
                        panic!("move action has no target");
                    },
                }

                // these parameters are technically optional, however
                // if not defined no movement will occur
                // range defaults to None
                let mut range = None;
                // min_range defaults to None
                let mut min_range = None;
                // no_skip defaults to None
                let mut no_skip = None;

                // check if additional parameters were passed in with the action
                if let Some(params) = &action.params {
                    // get range parameter from hashmap
                    range = params.get("range");
                    // get min_range parameter from hashmap
                    min_range = params.get("min_range");
                    // get no_skip parameter from hashmap
                    no_skip = params.get("no_skip");
                }

                // flag to check if move vector should be used
                // i.e. if position needs to be adjusted
                let mut use_move_vector = false;
                
                // check for parameter relational validity
                if let Some(&range) = range {
                    if let Some(&min_range) = min_range {
                        // if minimum range to launch the attack is greater than the maximmum range
                        // then give an error -> this needs to be handled and the action skipped
                        // but the player must also get a notification that this is an invalid action
                        if min_range > range {
                            // current behaviour causes the program to panic
                            panic!("min_range > range, need to implement warning system visible to players")
                        }
                    }
                }

                // get vector to target from current position
                let target_vector = Vec2::new(move_to.0 - pos.0, move_to.1 - pos.1);    
                // get distance between two points
                let dist = target_vector.length();
                // get normalized target vector
                let target_dir = target_vector.normalize();
                
                // check if min_range was specified
                if let Some(&min_range) = min_range {
                    // check if the entity is within the minimum range
                    if dist < min_range {
                        // if so
                        // get new target vector to appropriate range
                        let new_target_vector = target_dir * (dist - min_range);
                        // add target vector to current position vector to get
                        // the target coordinate
                        let edge_vector = new_target_vector + Vec2::new(pos.0, pos.1);
                        // update the target coordinates
                        move_to.0 = edge_vector[0];
                        move_to.1 = edge_vector[1];

                        // position must be adjusted
                        // set flag
                        use_move_vector = true;
                    }    
                }
                
                // check if range was specified
                if let Some(&range) = range {
                    // check if the entity is beyond the maximum range
                    if dist > range {
                        // if so
                        // get new target vector to appropriate range
                        let new_target_vector = target_dir * (dist - range);
                        // add target vector to current position vector to get 
                        // the target coordinate
                        let edge_vector = new_target_vector + Vec2::new(pos.0, pos.1);
                        // update the target coordinates
                        move_to.0 = edge_vector[0];
                        move_to.1 = edge_vector[1];
                        
                        // position must be adjusted
                        // set flag
                        use_move_vector = true;
                    }
                }

                if use_move_vector {
                    // only calculate velocity if velocity needs to be adjusted

                    // retrieve new straightline velocity to position
                    let new_vel = get_straightline_velocity(move_to, (pos.0, pos.1));
                
                    // set the velocity vector to use the new velocity vector
                    vel.0 = new_vel[0];
                    vel.1 = new_vel[1];
                }

                // check if this move can be skipped
                let mut can_skip = true;
                // check no_skip parameter to see if this move can be skipped
                if let Some(&no_skip) = no_skip {
                    // if no_skip is positive
                    if no_skip > 0.0 {
                        // set can_skip to false
                        can_skip = false;
                    }
                }

                // if no longer moving
                if vel.0.abs() < 1.0 && vel.1.abs() < 1.0 && can_skip {
                    // pop actions queue and ready next action
                    // check if there are still actions in the action queue
                    if let Some(action) = actions.action_queue.pop_front() {
                        // if there are still more actions
                        // set the next action to be the current action
                        actions.current_action = action;
                    }else{
                        // if there are no more actions in the action queue
                        // set the current action to be the empty action
                        actions.current_action = Action::default();
                    }
                }else{
                    
                }
            },
            // attack actions will attack a targeted entity
            ActionType::Attack => {
                // set to use attack animation
                sprite.animation_type = AnimationType::Attack;

                // update target position
                let mut target_pos = (f32::NAN, f32::NAN);

                // go through entities and find the correct position component
                for (id, pos) in &mut ent_query.iter() {
                    // check if the id matches
                    if *action.target.1.as_ref().unwrap() == id.id() {
                        // set the target position
                        target_pos = (pos.0, pos.1);
                    }
                }

                // these parameters are technically optional, however
                // if not defined the attack will never go out of range
                // range defaults to None
                let mut range = None;
                // min_range defaults to None
                let mut min_range = None;

                // check if additional parameters were passed in with the action
                if let Some(params) = &action.params {
                    // get range parameter from hashmap
                    range = params.get("range");
                    // get min_range parameter from hashmap
                    min_range = params.get("min_range");
                }

                // reattach flag, indicates whether or not
                // the attacker needs to enter optimal range again
                let mut reattach = false;

                // check if range was specified
                if let Some(&range) = range {
                    // get the distance vector from the player to the move point
                    let dist_vector = Vec2::new(target_pos.0 - pos.0, target_pos.1 - pos.1);
                    // the length of the distance vector is the distance between the two points
                    let dist = dist_vector.length();
                    // check if min_range was specified
                    if let Some(&min_range) = min_range {
                        // if minimum range to launch the attack is greater than the maximmum range
                        // then give an error -> this needs to be handled and the action skipped
                        // but the player must also get a notification that this is an invalid action
                        if min_range > range {
                            // current behaviour causes the program to panic
                            panic!("min_range > range, need to implement warning system visible to players")
                        }
                        // check if the entity is within the minimum range to launch the attack
                        // additional check to see if entity is barely on the border for minimum
                        // range - this is here because of the way that velocity is implemented
                        // we have an achilles and the tortoise type situation that makes it 
                        // difficult to actually get the entity exactly at the target point
                        if dist < min_range && !close_enough(dist, min_range, 1.0) {
                            // if so
                            // flag for reattachment
                            reattach = true;
                        }    
                    }
                    // check if the entity is beyond the maximum range to launch the attack
                    // additional check to see if entity is barely on the border for maximum
                    // range - this is here because of the way that velocity is implemented
                    // we have an achilles and the tortoise type situation that makes it 
                    // difficult to actually get the entity exactly at the target point
                    if dist > range && !close_enough(dist, range, 1.0) {
                        // if so
                        // flag for reattachment
                        reattach = true;
                    }
                }

                // check if flagged for reattachment
                if reattach {
                    // if so
                    // check the front of the action queue
                    match actions.action_queue.front() {
                        // if action queue is empty
                        None => {
                            // retrack target
                            actions.action_queue.push_back(Action {
                                action_type: ActionType::Move,
                                target: action.target.clone(),
                                params: action.params.clone(),
                            });
                            // attack target once target is tracked
                            actions.action_queue.push_back(Action {
                                action_type: ActionType::Attack,
                                target: action.target.clone(),
                                params: action.params.clone(),
                            });

                            // pop current action and move to next
                            if let Some(action) = actions.action_queue.pop_front() {
                                // if there are still more actions
                                // set the next action to be the current action
                                actions.current_action = action;
                            }else{
                                // if there are no more actions in the action queue
                                // set the current action to be the empty action
                                actions.current_action = Action::default();
                            }
                        },
                        // otherwise
                        _ => {
                            // if there are still actions in the queue
                            // assume that they override pressing the attack
                        }
                    }
                }

            },
            // wait actions do nothing for a specified amount of time
            ActionType::Wait => {
                if let Some(timer) = &mut actions.action_timer {
                    timer.tick(time.delta_seconds);
                    if timer.finished {
                        actions.action_timer = None;

                        // pop actions queue and ready next action
                        // check if there are still actions in the action queue
                        if let Some(action) = actions.action_queue.pop_front() {
                            // if there are still more actions
                            // set the next action to be the current action
                            actions.current_action = action;
                        }else{
                            // if there are no more actions in the action queue
                            // set the current action to be the empty action
                            actions.current_action = Action::default();
                        }
                    }
                }else{
                    // default duration is None
                    // if a duration parameter is not passed with the action
                    // default behaviour is to wait for one second
                    let mut duration = None;
                    // attempt to retrieve parameters
                    if let Some(params) = &action.params {
                        // attempt to retrieve duration parameter
                        duration = params.get("duration");
                    }
                    
                    match duration {
                        // if duration was specified
                        Some(&length) => {
                            // create a non-repeating timer that waits for <length> number of seconds
                            actions.action_timer = Some(Timer::from_seconds(length, false));
                        },
                        // if duration was not specified
                        None => {
                            // default behaviour is to wait for one second
                            // create a non-repeating timer to wait for one second
                            actions.action_timer = Some(Timer::from_seconds(1.0, false));
                        }
                    }
                }
            }
            // empty actions do nothing and are immediately popped
            ActionType::Empty => {
                // set to use idle animation
                sprite.animation_type = AnimationType::Idle;
                // empty actions do nothing, reset all moving parts and move on to the next
                vel.0 = 0.0;
                vel.1 = 0.0;
                // pop actions queue and ready next action
                // check if there are still actions in the action queue
                if let Some(action) = actions.action_queue.pop_front() {
                    // if there are still more actions
                    // set the next action to be the current action
                    actions.current_action = action;
                }else{
                    // if there are no more actions in the action queue
                    // set the current action to be the empty action
                    actions.current_action = Action::default();
                }
            },
            _ => {
                // this should never be reached
                panic!("unimplemented action");
            }
        }
    }
}

// animation plugin
// responsible for running the appropriate animation
struct AnimationPlugin;

// boilerplate code for plugin implementation
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // add frame rate regulator
        app.add_resource(AnimationFrameRate::new())
        // add animate system    
        .add_system(animate_system.system());
    }
}

// animation type enum
// should correspond to the different animation types we want
enum AnimationType {
    Attack,
    Move,
    Idle,
}

// sprite data component
// this allows for storage of frames and animation type
// and should be spawned along with any sprite that has animation
struct SpriteData {
    // animation type informs what animation should be run
    animation_type: AnimationType,

    // move frames contain the frames used when the sprite is moving
    move_frames: Vec<SpriteComponents>,
    // move frame index holds where the move animation currently is
    move_frame_index: usize,

    // idle frames contain the frames used when idling/the default
    // animation used
    idle_frames: Vec<SpriteComponents>,
    // idle frame index holds where the idle animation currently is
    idle_frame_index: usize,

    // attack frames contain the frames used when attacking
    attack_frames: Vec<SpriteComponents>,
    // attack frame index holds where the attack animation currently is
    attack_frame_index: usize,
}

// implementation for sprite data component
impl SpriteData {
    // new function provides an empty sprite data
    // set automatically to idle
    // note that frames need to be added for this component to work
    fn new() -> Self {
        SpriteData {
            animation_type: AnimationType::Idle,

            move_frames: Vec::new(),
            move_frame_index: 0,
            
            idle_frames: Vec::new(),
            idle_frame_index: 0,

            attack_frames: Vec::new(),
            attack_frame_index: 0,
        }
    }

    // add move frame
    // this function adds a frame (sprite) to the sprite data's move animation
    fn add_move_frame(&mut self, sprite: SpriteComponents) {
        self.move_frames.push(sprite);
    }
    // reset move animation
    // this function resets the move animation
    fn reset_move_animation(&mut self) {
        self.move_frame_index = 0;
    }
    // get move frame
    // this function gets the next frame for the move animation
    // it will also automatically reset the other animations
    fn get_move_frame(&mut self) -> SpriteComponents {
        // reset frame index for other animations
        self.reset_idle_animation();
        self.reset_attack_animation();

        // get frame for move animation
        let copyover = &self.move_frames[self.move_frame_index];
        // increment frame index
        self.move_frame_index = (self.move_frame_index + 1) % self.move_frames.len();
        
        // manually copy over sprite components because copy/clone aren't implemented for them
        SpriteComponents {
            material: copyover.material,
            translation: copyover.translation,
            sprite: Sprite {
                size: copyover.sprite.size,
            },
            ..Default::default()
        }
    }

    // add idle frame
    // this function adds a frame (sprite) to the sprite data's idle animation 
    fn add_idle_frame(&mut self, sprite: SpriteComponents) {
        self.idle_frames.push(sprite);
    }
    // reset idle animation
    // this function resets the idle animation
    fn reset_idle_animation(&mut self) {
        self.idle_frame_index = 0;
    }
    // get idle frame
    // this function gets the next frame for the idle animation
    // it will also automatically reset the other animations
    fn get_idle_frame(&mut self) -> SpriteComponents {
        // reset frame index for other animations
        self.reset_attack_animation();
        self.reset_move_animation();

        // get frame for idle animation
        let copyover = &self.idle_frames[self.idle_frame_index];
        // increment frame index
        self.idle_frame_index = (self.idle_frame_index + 1) % self.idle_frames.len();

        // manually copy over sprite components because copy/clone aren't implemented for them
        SpriteComponents {
            material: copyover.material,
            translation: copyover.translation,
            sprite: Sprite {
                size: copyover.sprite.size,
            },
            ..Default::default()
        }
    }

    // add attack frame
    // this function adds a frame (sprite) to the sprite data's attack animation 
    fn add_attack_frame(&mut self, sprite: SpriteComponents) {
        self.attack_frames.push(sprite);
    }
    // reset attack animation
    // this function resets the attack animation
    fn reset_attack_animation(&mut self) {
        self.attack_frame_index = 0;
    }
    // get attack frame
    // this function gets the next frame for the attack animation
    // it will also automatically reset the other animations
    fn get_attack_frame(&mut self) -> SpriteComponents {
        // reset frame index for other animations
        self.reset_idle_animation();
        self.reset_move_animation();
        
        // get frame for attack animation
        let copyover = &self.attack_frames[self.attack_frame_index];
        // increment frame index
        self.attack_frame_index = (self.attack_frame_index + 1) % self.attack_frames.len();
        
        // manually copy over sprite components because copy/clone aren't implemented for them
        SpriteComponents {
            material: copyover.material,
            translation: copyover.translation,
            sprite: Sprite {
                size: copyover.sprite.size,
            },
            ..Default::default()
        }
    }
}

// animation frame rate struct
// this struct contains a timer that is
// used to regulate the framerate of animations
// this means that the framerate of animations is potentially separate from
// the overall framerate of the game! (potential issue)
struct AnimationFrameRate(Timer);

// implementation for the animation frame rate struct
impl AnimationFrameRate {
    // gives a new animation frame rate struct, automatically set to a
    // default frame rate
    fn new() -> Self {
        // 6fps per second animation frame rate
        // create a repeating timer for animation frame rate
        AnimationFrameRate(Timer::from_seconds(4.0 / 24.0, true))
    }
    // generates a new animation frame rate struct from a given fps
    // fps refers to the desired number of frames per second
    fn from_frame_rate(fps: f32) -> Self {
        // create a repeating timer for animation frame rate
        AnimationFrameRate(Timer::from_seconds(1.0 / fps, true))
    }
}

// animate system
// responsible for playing the appropriate animations for each sprite
fn animate_system(time: Res<Time>, mut timer: ResMut<AnimationFrameRate>, mut query: Query<(&mut Handle<ColorMaterial>, &mut Sprite, &mut SpriteData)>) {
    // tick up on animation frame rate timer
    timer.0.tick(time.delta_seconds);
        
    // check if it's time for a new animation frame
    if timer.0.finished {
        // go through all sprites and get then assign new frames
        for (mut material, mut sprite, mut frames) in &mut query.iter() {    
            // sprite frame is defaulted to None
            let mut sprite_frame: Option<SpriteComponents> = None;

            // check animation type for current sprite
            match frames.animation_type {
                // if attack animation
                AnimationType::Attack => {
                    // get the next attack frame
                    sprite_frame = Some(frames.get_attack_frame());
                },
                // if move animation
                AnimationType::Move => {
                    // get the next move frame
                    sprite_frame = Some(frames.get_move_frame());
                },
                // if idle animation
                AnimationType::Idle => {
                    // get the next idle frame
                    sprite_frame = Some(frames.get_idle_frame());
                }
            }

            // if the frame exists
            if let Some(frame) = sprite_frame {
                *material = frame.material;
                *sprite = frame.sprite;
            }
        }
    }
}

// get player sprite template
// gives the template sprite for the player
// right now mostly just used for testing animation system
// actual method of getting player sprite may vary
fn get_player_sprite_template(materials: &mut ResMut<Assets<ColorMaterial>>) -> SpriteData {
    let mut template = SpriteData::new();
    
    let idle_one_handle = materials.add(Color::GREEN.into());
    let idle_two_handle = materials.add(Color::rgb(0.1, 1.0, 0.1).into());
    let idle_three_handle = materials.add(Color::rgb(0.25, 1.0, 0.25).into());
    let idle_four_handle = materials.add(Color::rgb(0.1, 1.0, 0.1).into());
    
    let attack_one_handle = materials.add(Color::rgb(1.0, 0.0, 0.0).into());
    let attack_two_handle = materials.add(Color::rgb(0.75, 0.25, 0.0).into());
    let attack_three_handle = materials.add(Color::rgb(0.5, 0.5, 0.0).into());
    let attack_four_handle = materials.add(Color::rgb(0.0, 1.0, 0.0).into());    
    
    let move_one_handle = materials.add(Color::GREEN.into());
    let move_two_handle = materials.add(Color::rgb(0.0, 0.75, 0.0).into());
    let move_three_handle = materials.add(Color::rgb(0.0, 0.5, 0.0).into());
    let move_four_handle = materials.add(Color::rgb(0.0, 0.75, 0.0).into());

    template.add_idle_frame(SimpleRect::new(idle_one_handle, Vec2::new(10.0, 10.0)));
    template.add_idle_frame(SimpleRect::new(idle_two_handle, Vec2::new(10.0, 10.0)));
    template.add_idle_frame(SimpleRect::new(idle_three_handle, Vec2::new(10.0, 10.0)));
    template.add_idle_frame(SimpleRect::new(idle_four_handle, Vec2::new(10.0, 10.0)));
    
    template.add_attack_frame(SimpleRect::new(attack_one_handle, Vec2::new(10.0, 10.0)));
    template.add_attack_frame(SimpleRect::new(attack_two_handle, Vec2::new(10.0, 10.0)));
    template.add_attack_frame(SimpleRect::new(attack_three_handle, Vec2::new(10.0, 10.0)));
    template.add_attack_frame(SimpleRect::new(attack_four_handle, Vec2::new(10.0, 10.0)));

    template.add_move_frame(SimpleRect::new(move_one_handle, Vec2::new(10.0, 10.0)));
    template.add_move_frame(SimpleRect::new(move_two_handle, Vec2::new(10.0, 10.0)));
    template.add_move_frame(SimpleRect::new(move_three_handle, Vec2::new(10.0, 10.0)));
    template.add_move_frame(SimpleRect::new(move_four_handle, Vec2::new(10.0, 10.0)));
    
    template
}

// get squadmate sprite template
// gives the template sprite for squadmates
// right now only used to test animation system
// actual method of getting squadmate sprites may vary
fn get_squadmate_sprite_template(materials: &mut ResMut<Assets<ColorMaterial>>) -> SpriteData {
    let mut template = SpriteData::new();
    
    let idle_one_handle = materials.add(Color::BLUE.into());
    let idle_two_handle = materials.add(Color::rgb(0.1, 0.1, 1.0).into());
    let idle_three_handle = materials.add(Color::rgb(0.25, 0.25, 1.0).into());
    let idle_four_handle = materials.add(Color::rgb(0.1, 0.1, 1.0).into());
    
    let attack_one_handle = materials.add(Color::rgb(1.0, 0.0, 0.0).into());
    let attack_two_handle = materials.add(Color::rgb(0.75, 0.0, 0.25).into());
    let attack_three_handle = materials.add(Color::rgb(0.5, 0.0, 0.5).into());
    let attack_four_handle = materials.add(Color::rgb(0.0, 0.0, 1.0).into());    
    
    let move_one_handle = materials.add(Color::BLUE.into());
    let move_two_handle = materials.add(Color::rgb(0.0, 0.0, 0.75).into());
    let move_three_handle = materials.add(Color::rgb(0.0, 0.0, 0.5).into());
    let move_four_handle = materials.add(Color::rgb(0.0, 0.0, 0.75).into());

    template.add_idle_frame(SimpleRect::new(idle_one_handle, Vec2::new(10.0, 10.0)));
    template.add_idle_frame(SimpleRect::new(idle_two_handle, Vec2::new(10.0, 10.0)));
    template.add_idle_frame(SimpleRect::new(idle_three_handle, Vec2::new(10.0, 10.0)));
    template.add_idle_frame(SimpleRect::new(idle_four_handle, Vec2::new(10.0, 10.0)));
    
    template.add_attack_frame(SimpleRect::new(attack_one_handle, Vec2::new(10.0, 10.0)));
    template.add_attack_frame(SimpleRect::new(attack_two_handle, Vec2::new(10.0, 10.0)));
    template.add_attack_frame(SimpleRect::new(attack_three_handle, Vec2::new(10.0, 10.0)));
    template.add_attack_frame(SimpleRect::new(attack_four_handle, Vec2::new(10.0, 10.0)));

    template.add_move_frame(SimpleRect::new(move_one_handle, Vec2::new(10.0, 10.0)));
    template.add_move_frame(SimpleRect::new(move_two_handle, Vec2::new(10.0, 10.0)));
    template.add_move_frame(SimpleRect::new(move_three_handle, Vec2::new(10.0, 10.0)));
    template.add_move_frame(SimpleRect::new(move_four_handle, Vec2::new(10.0, 10.0)));

    template
}

// get hostile sprite template
// gives the template sprite for hostiles
// right now only used to test animation system
// actual method of getting hostile sprites may vary
fn get_hostile_sprite_template(materials: &mut ResMut<Assets<ColorMaterial>>) -> SpriteData {
    let mut template = SpriteData::new();
    
    let idle_one_handle = materials.add(Color::BLACK.into());
    let idle_two_handle = materials.add(Color::rgb(0.1, 0.1, 0.1).into());
    let idle_three_handle = materials.add(Color::rgb(0.25, 0.25, 0.25).into());
    let idle_four_handle = materials.add(Color::rgb(0.1, 0.1, 0.1).into());
    
    let attack_one_handle = materials.add(Color::rgb(1.0, 0.0, 0.0).into());
    let attack_two_handle = materials.add(Color::rgb(0.75, 0.25, 0.25).into());
    let attack_three_handle = materials.add(Color::rgb(0.5, 0.5, 0.5).into());
    let attack_four_handle = materials.add(Color::rgb(0.0, 0.0, 0.0).into());    
    
    let move_one_handle = materials.add(Color::BLACK.into());
    let move_two_handle = materials.add(Color::rgb(0.25, 0.0, 0.25).into());
    let move_three_handle = materials.add(Color::rgb(0.5, 0.0, 0.5).into());
    let move_four_handle = materials.add(Color::rgb(0.25, 0.0, 0.25).into());

    template.add_idle_frame(SimpleRect::new(idle_one_handle, Vec2::new(10.0, 10.0)));
    template.add_idle_frame(SimpleRect::new(idle_two_handle, Vec2::new(10.0, 10.0)));
    template.add_idle_frame(SimpleRect::new(idle_three_handle, Vec2::new(10.0, 10.0)));
    template.add_idle_frame(SimpleRect::new(idle_four_handle, Vec2::new(10.0, 10.0)));
    
    template.add_attack_frame(SimpleRect::new(attack_one_handle, Vec2::new(10.0, 10.0)));
    template.add_attack_frame(SimpleRect::new(attack_two_handle, Vec2::new(10.0, 10.0)));
    template.add_attack_frame(SimpleRect::new(attack_three_handle, Vec2::new(10.0, 10.0)));
    template.add_attack_frame(SimpleRect::new(attack_four_handle, Vec2::new(10.0, 10.0)));

    template.add_move_frame(SimpleRect::new(move_one_handle, Vec2::new(10.0, 10.0)));
    template.add_move_frame(SimpleRect::new(move_two_handle, Vec2::new(10.0, 10.0)));
    template.add_move_frame(SimpleRect::new(move_three_handle, Vec2::new(10.0, 10.0)));
    template.add_move_frame(SimpleRect::new(move_four_handle, Vec2::new(10.0, 10.0)));

    template
}

// Behaviour plugin
// responsible for independent action generation
struct BehaviourPlugin;

// boilerplate code for Behaviour plugin
impl Plugin for BehaviourPlugin {
    fn build(&self, app: &mut AppBuilder){
        // add in simple idle system
        app.add_system(simple_idle_system.system());     
    }
}

// simple idle system
// allows AI actors to wander around aimlessly
// will probably be replaced, reworked or at least renamed
fn simple_idle_system(mut query: Query<(&Behaviour, &Nerve, &mut Pathfinder, &Position)>) {
    // initialise random number generator
    let mut rng = rand::thread_rng();

    // iterate through every entity with a brain, nervous system, and a physical position
    for (_control, actions, mut pf, pos) in &mut query.iter() {
        // check both current action as well as action queue
        match (actions.current_action.action_type, actions.action_queue.front()) {
            // if there is no current action and the action queue is empty
            (ActionType::Empty, None) => {
                // generate a random coordinate within 200 units of the current position
                // horizontal deviation
                let rand_x = rng.gen::<f32>() * 200.0 - rng.gen::<f32>() * 200.0;
                // vertical deviation
                let rand_y = rng.gen::<f32>() * 200.0 - rng.gen::<f32>() * 200.0;
                
                // get random coordinate and make sure it remains in bounds
                let loiter_x = (rand_x + pos.0).max(10.0).min(WINDOW_WIDTH - 10.0);
                let loiter_y = (rand_y + pos.1).max(10.0).min(WINDOW_HEIGHT - 10.0);

                pf.needs_pathfinding = true;
                pf.path_goal = TilePos::from_coords(loiter_x, loiter_y);
                pf.real_goal = (loiter_x, loiter_y);

                /*let mut params = HashMap::new();
                // range refers to the maximum range at which an attack can be launched
                params.insert("range".to_string(), 0.0);
                // add a move action to the randomly generated coordinate
                actions.action_queue.push_back(Action {
                    action_type: ActionType::Move,
                    target: (Some((loiter_x, loiter_y)), None),
                    params: Some(params),
                });

                // create parameters hashmap
                let mut params = HashMap::new();
                // add duration parameter with value 3.0 (seconds)
                params.insert("duration".to_string(), 3.0);

                // add Wait action
                actions.action_queue.push_back(Action {
                    action_type: ActionType::Wait,
                    target: (None, None),
                    params: Some(params),
                })*/
            }
            _ => {

            }     
        }
    }
}

enum BehaviourSet {
    AtRest,
    OnMarch,
    PreCombat,
    Combat,
    Retreat,
    Empty,
}

enum BehaviourType {
    Rest,
    Loiter,
    Alert,
    Hide,
    Preparation,
    AlertMove,
    LoiterMove,
    Scout,
    Stalk,
    Vantage,
    Charge,
    Flank,
    Defend,
    Kite,
    Flee,
    Empty,
}

struct Behaviour {
    current_behaviour_set: BehaviourSet,
    current_behaviour: BehaviourType,
}

impl Default for Behaviour {
    fn default() -> Self {
        Behaviour {
            current_behaviour_set: BehaviourSet::Empty,
            current_behaviour: BehaviourType::Empty,
        }
    }
}

fn select_behaviour_set_system(mut query: Query<(&Position, &mut Behaviour, &mut Nerve)>) {

}

fn select_behaviour_system(mut query: Query<(&Position, &mut Behaviour, &mut Nerve)>) {
    
}

fn run_behaviour_system(mut query: Query<(&Position, &mut Behaviour, &mut Nerve)>) {
    for (pos, mut behav, mut nerv) in &mut query.iter() {
        match &behav.current_behaviour {
            
            Empty => {

            },
            _ => {

            },
        }    
    }
}

struct MapCoords(f32, f32);

struct PathfindersQueue(usize);

#[derive(Default, PartialEq, Eq, Clone, Copy, Hash)]
struct TilePos(usize, usize);
 
impl TilePos {
    fn from_coords(x: f32, y: f32) -> Self {
        if x < 0.0 || y < 0.0 {
            panic!("bad coordinates: negative, non-existent tile");
        }
        TilePos((x / TILE_SIZE) as usize, (y / TILE_SIZE) as usize)
    }
    fn to_coords(&self) -> (f32, f32) {
        (TILE_SIZE / 2.0 + self.0 as f32 * TILE_SIZE, TILE_SIZE / 2.0 + self.1 as f32 * TILE_SIZE)
    }
}


struct Pathfinder {
    needs_pathfinding: bool,
    path_ready: bool,
    path_start: TilePos,
    path_goal: TilePos,
    real_goal: (f32, f32),
    tile_path: Vec<TilePos>,
    path: Vec<(f32, f32)>,
    path_index: usize,
}

impl Default for Pathfinder {
    fn default() -> Self {
        Pathfinder {
            needs_pathfinding: false,
            path_ready: false,
            path_start: TilePos::default(),
            path_goal: TilePos::default(),
            real_goal: (0.0, 0.0),
            path: Vec::new(),
            tile_path: Vec::new(),
            path_index: 0,
        }
    }
}

struct MapPlugin;

impl Plugin for MapPlugin {
    fn build (&self, app: &mut AppBuilder){
        app.add_resource(MapCoords(0.0, 0.0))
            .add_resource(MapData::default())
            .add_resource(PathfindersQueue(0))
            .add_system(update_map_system.system())
            .add_system(pathfind_system.system())
            .add_system(follow_path_system.system());
    }
}


fn update_map_system(coords: Res<MapCoords>, mut map: ResMut<MapData>, mut query: Query<(&Person, &Position)>) {
    map.update_map(coords.0 as i32, coords.1 as i32);
    for (_person, pos) in &mut query.iter() {
        map.set_tile_occupied(&TilePos::from_coords(pos.0, pos.1));
    }
}

fn follow_path_system(mut query: Query<(&mut Pathfinder, &mut Nerve, &Position)>) {
    for (mut pf, mut actions, pos) in &mut query.iter() {
        if pf.tile_path.len() == 0 {
            continue;
        }

        if pf.path_index < pf.tile_path.len() {

            let path_tile = pf.tile_path[pf.path_index];
            
            if TilePos::from_coords(pos.0, pos.1) == path_tile {
                pf.path_index += 1;
            }

        }

        if pf.path_index < pf.tile_path.len() && actions.is_curr_action_empty() && actions.action_queue.is_empty() {
            for i in pf.path_index..pf.tile_path.len() {
                let mut params = HashMap::new();

                params.insert("range".to_string(), TILE_SIZE);

                actions.action_queue.push_back(Action {
                    action_type: ActionType::Move,
                    target: (Some(pf.path[i]), None),
                    params: Some(params),
                });
            }
            pf.path_ready = true;
        }
    }
}

fn pathfind_system(mut waiting: ResMut<PathfindersQueue>, map: Res<MapData>, mut query: Query<(&mut Pathfinder, &Position)>) {
    for (mut pf, pos) in &mut query.iter() {
        if !pf.needs_pathfinding {
            continue;
        }
        if waiting.0 < MAX_PATHFINDERS {
            waiting.0 += 1;
            // update path index
            pf.path_index = 0;
            // update start coordinates
            pf.path_start = TilePos::from_coords(pos.0, pos.1);
            
            if map.is_tile_occupied(&pf.path_goal) {
                //panic!("tile destination is occupied");
            }

            // pathfind here
            let path = astar(&pf.path_start, |p| map.successors(p), |p| map.get_diag_dist(*p, pf.path_goal), |p| *p == pf.path_goal);
            //let path = None;
            match path {
                Some((path, cost)) => {
                    let real_goal = pf.real_goal;

                    pf.tile_path = path.clone();
                    pf.path = path.iter().map( |t| t.to_coords()).collect();

                    pf.tile_path.push(TilePos::from_coords(real_goal.0, real_goal.1));
                    pf.path.push((real_goal.0, real_goal.1));
                },
                None => {
                    panic!("path not found");
                }
            }

            pf.needs_pathfinding = false;
        }else{
            // max pathfinders this frame reached
            // break loop
            break;
        }
    }
    // reset pathfinders queue
    waiting.0 = 0;
}

enum TileType {
    Grass,
    Water,
    Empty,
}

#[derive(Clone)]
struct MapData {
    generator: noise::Perlin,
    size: (usize, usize),
    data: Vec::<f32>,
    occupied: Vec::<bool>,
}

fn get_map_weight_from_tile_type(tile: TileType) -> f32 {
    match tile {
        TileType::Grass => {
            1.0
        },
        _ => {
            1.0
            //f32::INFINITY
        },
    }
}

impl Default for MapData {
    fn default() -> Self {
        MapData::new(0)
    }
}


impl MapData {
    fn new(seed: u32) -> Self {
        let gen = Perlin::new();
        gen.set_seed(seed);
        let size = ((WINDOW_WIDTH / TILE_SIZE) as usize, (WINDOW_HEIGHT / TILE_SIZE) as usize);
        MapData {
            generator: gen,
            size: size,
            data: vec![0.0; size.0 * size.1],
            occupied: vec![false; size.0 * size.1],
        }
    }
    fn convert_f64_to_tiletype(float: f64) -> TileType {
        match float {
            0.0..=0.5 => {
                TileType::Water
            },
            0.5..=1.0 => {
                TileType::Grass
            },
            _ => {
                TileType::Empty
            }
        }
    }
    fn successors(&self, tile: &TilePos) -> Vec<(TilePos, OrderedFloat<f32>)> {
        let &TilePos(x, y) = tile;
        let mut output = Vec::new();

        for i in -1..2 {
            for j in -1..2 {
                if i == 0 && j == 0 {
                    continue
                }
                let mx = x as i32 + i;
                let my = y as i32 + j;
                if (mx as usize) < self.size.0 && (my as usize) < self.size.1 && mx >= 0 && my >= 0 {
                    output.push((TilePos(mx as usize, my as usize), self.get_weight(tile)))
                }
            }
        }
        
        output
    }
    fn is_tile_occupied(&self, tile: &TilePos) -> bool {
        let &TilePos(x, y) = tile;
        self.occupied[x + y * self.size.0]
    }
    fn set_tile_occupied(&mut self, tile: &TilePos) {
        let &TilePos(x, y) = tile;
        self.occupied[x + y * self.size.0] = true;
    }
    fn get_weight(&self, tile: &TilePos) -> OrderedFloat<f32> {
        let &TilePos(x, y) = tile;
        OrderedFloat(self.data[x + y * self.size.0])
    }
    fn get_diag_dist(&self, a: TilePos, b: TilePos) -> OrderedFloat<f32> {
        let TilePos(ax, ay) = a;
        let TilePos(bx, by) = b;
        let dx = (ax as f32 - bx as f32).abs();
        let dy = (ay as f32 - by as f32).abs();
        let c = self.get_weight(&a).0;
        OrderedFloat(c * (dx + dy) + (c * 1.414 - 2.0 * c) * dx.min(dy))
    }
    fn get_tile(&self, x: i32, y: i32) -> TileType{
        let noise = self.generator.get([x as f64, y as f64]);
        MapData::convert_f64_to_tiletype(noise)
    }
    fn update_map(&mut self, x: i32, y: i32) {
        for j in 0..self.size.1 {
            for i in 0..self.size.0 {
                self.data[i + j * self.size.0] = get_map_weight_from_tile_type(self.get_tile(i as i32 + x, j as i32 + y));
            }
        }
    }
}
