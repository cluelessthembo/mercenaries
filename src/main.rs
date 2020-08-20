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
    // run the app
    .run();
}

// fps counter component
// spawn this component along any text components that will be used as fps counters 
pub struct FPSMeter;

// initial setup function, 
// spawn in necessary entities (cameras)
// along with fps counter
fn setup(mut commands: Commands, asset_server: Res<AssetServer>){
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

fn get_translate_from_position(x: f32, y: f32) -> (f32, f32) {
    // translation has (0, 0) at the center of the screen
    // it also has the y-coordinates increase from bottom to top
    // we must invert the y-coordinates to use the right scale, then
    // we must shift the position coordinates towards the 
    // upper left corner of the screen by half the 
    // screen dimensions
    (x - 1600.0 / 2.0, (900.0 - y) - 900.0 / 2.0)
}

fn draw_sprite_system(mut query: Query<(&Sprite, &mut Translation, &Position)>){
    for (_sprite, mut transl, pos) in &mut query.iter() {
        
        let adj_pos = get_translate_from_position(pos.0, pos.1);

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
    current_command: Command,
    squad_pos: i32,
    command_queue: Vec<Command>,
}

impl Controlled {
    fn new(i: i32) -> Self {
        Controlled {
            current_command: Command::default(),
            squad_pos: i,
            command_queue: Vec::new(),
        }
    }
}

#[derive(Default)]
struct Command {
    command_type: CommandType,
    target: String,
    move_to: (f32, f32),
}
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

struct SimpleRect;

impl SimpleRect {
    fn new(color_handle: Handle<ColorMaterial>, size: Vec2) -> SpriteComponents {
        SpriteComponents {
            material: color_handle,
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
        .with(Size(5.0, 5.0))
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
        .with(Size(5.0, 5.0))
        .spawn(
            //Label::new("P3".to_string(), font_handle.clone(), Color::WHITE, 12.0),
            SimpleRect::new(blue_handle, Vec2::new(10.0, 10.0))
        )
        .with(Id::new())
        .with(Person)
        .with(Position(600.0, 100.0))
        .with(Velocity(0.0, 0.0))
        .with(Controlled::new(2))
        .with(Size(5.0, 5.0))
        .spawn(
            //Label::new("P4".to_string(), font_handle.clone(), Color::WHITE, 12.0),
            SimpleRect::new(blue_handle, Vec2::new(10.0, 10.0))
        )
        .with(Id::new())
        .with(Person)
        .with(Position(1000.0, 300.0))
        .with(Velocity(0.0, 0.0))
        .with(Controlled::new(3))
        .with(Size(5.0, 5.0))
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

    let red_handle = materials.add(Color::RED.into());
    
    commands
        // spawn in text components for the label
        .spawn(
            //Label::new("H0".to_string(), font_handle.clone(), Color::RED, 12.0),
            SimpleRect::new(red_handle, Vec2::new(10.0, 10.0)),
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
        .with(Size(5.0, 5.0))
        // repeat for another hostile entity
        .spawn(
            SimpleRect::new(red_handle, Vec2::new(10.0, 10.0)),
        )
        .with(Id::new())
        .with(Person)
        .with(Position(400.0, 200.0))
        .with(Velocity(0.0, 0.0))
        .with(Hostile)
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
        .add_system(control_player_system.system());
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
fn move_controlled_system(mut query: Query<(&Controlled, &mut Velocity, &Position)>) {
    for (state, mut vel, pos) in &mut query.iter() {
        let command = &state.current_command;
        
        match command.command_type {
            CommandType::Move => {
                // get the distance vector from the player to the move point
                let dist_vector = Vec2::new(command.move_to.0 - pos.0, command.move_to.1 - pos.1);
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
            },
            CommandType::Attack => {
                
            }
            _ => {

            },
        }
    }
}

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
        _ => -1
    }
}

fn check_point_collision(point: (f32, f32), box_position: (f32, f32), box_radius: (f32, f32)) -> bool {
    if (point.0 < box_position.0 + box_radius.0) && (point.0 > box_position.0 - box_radius.0)
        && (point.1 < box_position.1 + box_radius.1) && (point.1 > box_position.1 - box_radius.1) {
        true
    }else{
        false
    }
}

#[derive(Copy, Clone)]
enum CommandType {
    Move,
    Attack,
    Empty,
}

impl Default for CommandType {
    fn default() -> Self {
        CommandType::Empty
    }
}


// player control system
// responsible for translating all inputs into the respective actions in-game
fn control_player_system(inputs: Res<InputState>, mut controlstate: Query<&mut Controlled>, mut hostiles: Query<(&Id, &Hostile, &Position, &Size)>) {
    // if the left mouse button is pressed
    if inputs.mouse_presses.contains(&MouseButton::Left) {
        
        let mut command_type = CommandType::Move;
        let mut target_entity = "null".to_string();

        // check if you clicked on something
        for (id, _host, pos, size) in &mut hostiles.iter() {
            //println!("entity is {}", id.id());
            if check_point_collision(inputs.mouse_position, (pos.0, pos.1), (size.0, size.1)) {
                //println!("clicked on {}", id.id());
                command_type = CommandType::Attack;
                target_entity = id.id();
            }
        }
        
        
        let mut squad_control = Vec::new();
        
        for key in &mut inputs.key_presses.iter() {
            let squad_pos = convert_keycode_to_squad_pos(*key);
            if squad_pos >= 0 {
                squad_control.push(squad_pos);
            }
        }

        if squad_control.is_empty() {
            // if no squad keys are pressed, assume controls are for player
            squad_control.push(0);
        }

        for mut state in &mut controlstate.iter() {
            if squad_control.contains(&state.squad_pos) {
                
                match command_type {
                    CommandType::Move => {
                        state.current_command = Command {
                            command_type: command_type,
                            move_to: inputs.mouse_position.clone(),
                            target: "null".to_string(),
                        };
                    },
                    CommandType::Attack => {
                        state.current_command = Command {
                            command_type: command_type,
                            move_to: (f32::NAN, f32::NAN),
                            target: target_entity.clone(),
                        };
                    },
                    CommandType::Empty => {
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
