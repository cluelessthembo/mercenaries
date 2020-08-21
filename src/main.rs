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
// imports for VecDeque - which is more efficient at removing items from the front
use std::collections::VecDeque;
struct Id(String);

impl Id {
    fn new() -> Self {
        Id(
            Uuid::new_v4().to_simple().to_string()
        )
    }
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
        title: "Mercenaries v0.0.0".to_string(),
        width: 1600,
        height: 900,
        vsync: true,
        ..Default::default()
    })
    // resource used to determine background colour of window
    .add_resource(ClearColor(Color::rgb(0.2, 0.2, 0.8)))
    // adds useful plugins for making a game
    .add_default_plugins()
    // add in the fps diagnostics plugin
    .add_plugin(FrameTimeDiagnosticsPlugin::default())
    // perform initial setup
    .add_startup_system(setup.system())
    // add in the fps counter system
    .add_system(fps_monitor_system.system())
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
    // add in the AI plugin
    .add_plugin(ActionsPlugin)
    .add_plugin(AnimationPlugin)
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
    (x - 1600.0 / 2.0, (900.0 - y) - 900.0 / 2.0)
}

// draw sprite system
// responsible for moving sprites to their proper positions for 
// display
fn draw_sprite_system(mut query: Query<(&Sprite, &mut Translation, &Position)>){
    for (_sprite, mut transl, pos) in &mut query.iter() {
        
        // get the proper coordinates for translation
        let adj_pos = get_translate_from_position(pos.0, pos.1);

        // assign coordinates
        transl.0 = Vec3::new(adj_pos.0, adj_pos.1, 0.0);
    }
}
// person plugin
// adds in all the people
pub struct PersonPlugin;
// person component
// spawn this component along with any entity that should be considered a person
struct Person;
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
    target: String,
    // move_to is the target coordinate, if it exists
    move_to: (f32, f32),
}
// Brain component
// holds the current action as well as succeeding actions
struct Brain {
    // current action is the action being worked on
    current_action: Action,
    // action queue holds the next actions, after the current one
    action_queue: VecDeque<Action>,
}

impl Brain {
    // function to initialise a new Brain component
    fn new() -> Self {
        Brain {
            // initialise with an empty action
            current_action: Action::default(),
            // initialise with an empty action queue
            action_queue: VecDeque::new(),
        }
    }
}

// enum for the type of action 
#[derive(Debug)]
enum ActionType {
    // move actions will move entities to a stationary point
    Move,
    // attack actions will launch attacks at an entity until it 
    // ceases to become hostile
    Attack,
    // track actions will move entities to follow a moving entity
    Track,
    // empty actions do nothing
    Empty,
}

// action struct
#[derive(Debug)]
struct Action {
    action_type: ActionType,
    // target is either a coordinate point or an entity id
    target: (Option<(f32, f32)>, Option<String>),
}

impl Default for Action {
    // default function for action
    // gives a default Action, which is an empty action
    fn default() -> Self {
        Action {
            action_type: ActionType::Empty,
            target: (None, None),
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
            translation: Translation(Vec3::new(-1000.0, -1000.0, 0.0)),
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
        .with(Person)
        // spawn position component along with so that this entity has a physical position
        .with(Position(100.0, 100.0))
        // spawn velocity component along with so that this entity has a physical velocity and can move
        .with(Velocity(0.0, 0.0))
        // spawn controlled component along with so that this entity is controlled by the player
        .with(Controlled::new(0))
        .with(Brain::new())
        .with(Size(5.0, 5.0))

        .with(get_player_sprite_template(&mut materials))
        // same deal for the other three persons
        // note however that only the first has the controlled component
        // we will consider that entity our player character
        .spawn(
            //Label::new("P1".to_string(), font_handle.clone(), Color::WHITE, 12.0),
            SimpleRect::new(blue_handle, Vec2::new(10.0, 10.0))
        )
        .with(Id::new())
        .with(Person)
        .with(Position(800.0, 500.0))
        .with(Velocity(0.0, 0.0))
        .with(Controlled::new(1))
        .with(Brain::new())
        .with(Size(5.0, 5.0))

        .with(get_squadmate_sprite_template(&mut materials))

        .spawn(
            //Label::new("P3".to_string(), font_handle.clone(), Color::WHITE, 12.0),
            SimpleRect::new(blue_handle, Vec2::new(10.0, 10.0))
        )
        .with(Id::new())
        .with(Person)
        .with(Position(600.0, 100.0))
        .with(Velocity(0.0, 0.0))
        .with(Controlled::new(2))
        .with(Brain::new())
        .with(Size(5.0, 5.0))

        .with(get_squadmate_sprite_template(&mut materials))

        .spawn(
            //Label::new("P4".to_string(), font_handle.clone(), Color::WHITE, 12.0),
            SimpleRect::new(blue_handle, Vec2::new(10.0, 10.0))
        )
        .with(Id::new())
        .with(Person)
        .with(Position(1000.0, 300.0))
        .with(Velocity(0.0, 0.0))
        .with(Controlled::new(3))
        .with(Brain::new())
        .with(Size(5.0, 5.0))

        .with(get_squadmate_sprite_template(&mut materials))
        ;
}
// encounter plugin
// responsible for generating encounters for the player
pub struct EncounterPlugin;
// hostile component
// spawn this component along side any entity that is hostile to the player
struct Hostile;

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
        .with(Person)
        // spawn along the position component so that this entity has a physical position on the screen
        .with(Position(200.0, 400.0))
        // spawn along the velocity component so that this entity has a physical velocity and can move
        .with(Velocity(0.0, 0.0))
        // spawn along the hostile component so that this entity is considered hostile to the player
        .with(Hostile)
        .with(Brain::new())
        .with(Size(5.0, 5.0))
        // repeat for another hostile entity
        .spawn(
            SimpleRect::new(black_handle, Vec2::new(10.0, 10.0)),
        )
        .with(Id::new())
        .with(Person)
        .with(Position(400.0, 200.0))
        .with(Velocity(0.0, 0.0))
        .with(Hostile)
        .with(Brain::new())
        .with(Size(5.0, 5.0))
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
fn move_controlled_system(mut query: Query<(&mut Controlled, &mut Brain, &mut Velocity, &Position)>) {
    for (mut state, mut actions, mut vel, pos) in &mut query.iter() {
        let command = &state.current_command;
        
        match command.command_type {
            CommandType::Move => {
                // clear current actions to replace with new actions
                actions.current_action = Action::default();
                actions.action_queue.clear();

                // add move action to the target location
                actions.action_queue.push_back(Action {
                    action_type: ActionType::Move,
                    target: (Some(command.move_to), None),
                });
            },
            CommandType::Attack => {
                // clear current actions to replace with new actions
                actions.current_action = Action::default();
                actions.action_queue.clear();

                // add move action to the target entity
                // get within a certain distance of the target
                actions.action_queue.push_back(Action {
                    action_type: ActionType::Track,
                    target: (None, Some(command.target.clone())),
                });

                // add attack action
                // attack the target
                actions.action_queue.push_back(Action {
                    action_type: ActionType::Attack,
                    target: (None, Some(command.target.clone())),
                });
            }
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
// box_radius is the distance the edges are from box_position
fn check_point_collision(point: (f32, f32), box_position: (f32, f32), box_radius: (f32, f32)) -> bool {
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
fn player_control_system(inputs: Res<InputState>, mut controlstate: Query<&mut Controlled>, mut hostiles: Query<(&Id, &Hostile, &Position, &Size)>) {
    // if the left mouse button was just pressed
    if inputs.mouse_just_presses.contains(&MouseButton::Left) {
        
        // if the left mouse button was clicked, default to a move command
        let mut command_type = CommandType::Move;
        // a move command by default has no target entity
        let mut target_entity = "null".to_string();

        // check if you clicked on something
        for (id, _host, pos, size) in &mut hostiles.iter() {
            // check if an entity was clicked
            if check_point_collision(inputs.mouse_position, (pos.0, pos.1), (size.0, size.1)) {
                // if an entity was clicked
                // switch command type to an attack type    
                command_type = CommandType::Attack;
                // set the target entity to the entity clicked
                target_entity = id.id();
                // once target entity is found, break out of the loop
                break;
            }
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
                            move_to: inputs.mouse_position.clone(),
                            target: "null".to_string(),
                        };
                    },
                    // if the command type is attack
                    CommandType::Attack => {
                        // set the current command to an attack type command
                        // at the entity clicked
                        state.current_command = Command {
                            command_type: command_type,
                            move_to: (f32::NAN, f32::NAN),
                            target: target_entity.clone(),
                        };
                    },
                    // if the command type is empty
                    CommandType::Empty => {
                        // set the current command to an empty type command
                        state.current_command = Command {
                            command_type: command_type,
                            move_to: (f32::NAN, f32::NAN),
                            target: "null".to_string(),
                        }
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

// run action system
// responsible for implementing the various actions used for lower level control of entities
fn run_action_system(mut query: Query<(&mut Brain, &Position, &mut Velocity, &mut SpriteData)>, mut ent_query: Query<(&Id, &Position)>) {
    // go through all entities with a brain, position, and velocity
    for (mut actions, pos, mut vel, mut sprite) in &mut query.iter() {
        // get the current action
        let action = &actions.current_action;

        // check the action type
        match action.action_type {
            // move actions will move the entity to a stationary point
            ActionType::Move => {

                sprite.animation_type = AnimationType::Move;

                // get the distance vector from the player to the move point
                let dist_vector = Vec2::new(action.target.0.unwrap().0 - pos.0, action.target.0.unwrap().1 - pos.1);
                // the length of the distance vector is the distance between the two points
                let dist = dist_vector.length();
                // if distance is 0 then the velocity vector is 0
                let mut new_vel = Vec2::new(0.0, 0.0);
                // otherwise if distance is greater than 0
                if dist > 0.0 {
                    // divide the distance by the distance factor and cap at 1.0
                    let ease_input = (dist / (2.75 * 50.0)).min(1.0);

                    // new velocity vector is a rescaled exponential applied to the normalized distance vector
                    // the result is that speed is based on distance and varies according to an exponential curve
                    // and the velocity is always towards the move point
                    // if pathfinding is implemented for the player, then this will need to be changed
                    new_vel = ezing::expo_out( ease_input ) * (2.75 * 50.0) * dist_vector.normalize();
                }
                // if the new x-velocity has insignificant magnitude,
                // reduce it to 0
                // this is to avoid sliding
                if new_vel[0].abs() < 1.0 {
                    new_vel[0] = 0.0;            
                }
                // if the new y-velocity has insignificant magnitude,
                // reduce it to 0
                // this is to avoid sliding
                if new_vel[1].abs() < 1.0 {
                    new_vel[1] = 0.0;
                }
                // set the velocity vector to use the new velocity vector
                vel.0 = new_vel[0];
                vel.1 = new_vel[1];

                // if no longer moving
                if vel.0 == 0.0 && vel.1 == 0.0 {
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
            },
            // track actions move the entity to a moving entity
            ActionType::Track => {

                sprite.animation_type = AnimationType::Move;

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


                // get the distance vector from the player to the move point
                let dist_vector = Vec2::new(target_pos.0 - pos.0, target_pos.1 - pos.1);
                // the length of the distance vector is the distance between the two points
                let dist = dist_vector.length();
                // if distance is 0 then the velocity vector is 0
                let mut new_vel = Vec2::new(0.0, 0.0);
                // otherwise if distance is greater than 0
                if dist > 0.0 {
                    // divide the distance by the distance factor and cap at 1.0
                    let ease_input = (dist / (2.75 * 50.0)).min(1.0);

                    // new velocity vector is a rescaled exponential applied to the normalized distance vector
                    // the result is that speed is based on distance and varies according to an exponential curve
                    // and the velocity is always towards the move point
                    // if pathfinding is implemented for the player, then this will need to be changed
                    new_vel = ezing::expo_out( ease_input ) * (2.75 * 50.0) * dist_vector.normalize();
                }
                // if the new x-velocity has insignificant magnitude,
                // reduce it to 0
                // this is to avoid sliding
                if new_vel[0].abs() < 1.0 {
                    new_vel[0] = 0.0;            
                }
                // if the new y-velocity has insignificant magnitude,
                // reduce it to 0
                // this is to avoid sliding
                if new_vel[1].abs() < 1.0 {
                    new_vel[1] = 0.0;
                }
                // set the velocity vector to use the new velocity vector
                vel.0 = new_vel[0];
                vel.1 = new_vel[1];

                // if no longer moving (probably at destination)
                if vel.0 == 0.0 && vel.1 == 0.0 {
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
            },
            // attack actions will attack a targeted entity
            ActionType::Attack => {
                // currently attacking does nothing but print out that you're attacking
                sprite.animation_type = AnimationType::Attack;
            },
            // empty actions do nothing
            ActionType::Empty => {
                sprite.animation_type = AnimationType::Idle;
                // empty actions do nothing, immediately move to the next
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
        AnimationFrameRate(Timer::from_seconds(4.0 / 24.0))
    }
    // generates a new animation frame rate struct from a given fps
    // fps refers to the desired number of frames per second
    fn from_frame_rate(fps: f32) -> Self {
        AnimationFrameRate(Timer::from_seconds(1.0 / fps))
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
        // reset frame timer
        timer.0.reset();
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