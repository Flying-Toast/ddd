use std::collections::HashMap;
use std::borrow::Cow;
use crate::slice::Slice;
use crate::ConfigProfile;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    fn to_str(&self) -> &'static str {
        match self {
            Axis::X => "X",
            Axis::Y => "Y",
            Axis::Z => "Z",
        }
    }
}

/// A GCode instruction
pub enum Command {
    /// Homes each axis in the PerAxis. If no axes are specified, homes all axes.
    Home(PerAxis<()>),
    SetAbsolutePositioning,
    SetRelativePositioning,
    /// Moves each axis by the given amount (or *to* the given location, depending on the
    /// positioning mode)
    Move {
        amounts: PerAxis<i64>,
        speed: u32,
    },
    /// Like `Move`, but extrudes filament during the move
    ExtrudeMove {
        amounts: PerAxis<i64>,
        speed: u32,
        /// How much filament to extrude during the move
        extrude_len: u32,
    },
    SetPosition(PerAxis<i64>),
    SetExtruderPosition(i64),
    BlockingSetTemp(u32),
}

impl Command {
    fn as_code(&self) -> Cow<'static, str> {
        use Command::*;
        match self {
            Home(axes) => format!(
                "G28{}",
                axes.entries()
                    .map(|(axis, _)| format!(" {}", axis.to_str()))
                    .collect::<String>(),
            ).into(),
            SetAbsolutePositioning => "G90".into(),
            SetRelativePositioning => "G91".into(),
            Move { amounts, speed } => format!(
                "G1 {}F{}",
                amounts.entries()
                    .map(|(axis, amnt)| format!("{}{} ", axis.to_str(), amnt))
                    .collect::<String>(),
                speed,
            ).into(),
            ExtrudeMove { amounts, speed, extrude_len } => format!(
                "G1 {}E{} F{}",
                amounts.entries()
                    .map(|(axis, amnt)| format!("{}{} ", axis.to_str(), amnt))
                    .collect::<String>(),
                extrude_len,
                speed,
            ).into(),
            SetPosition(pozs) => format!(
                "G92{}",
                pozs.entries()
                    .map(|(axis, pos)| format!(" {}{}", axis.to_str(), pos))
                    .collect::<String>(),
            ).into(),
            SetExtruderPosition(pos) => format!("G92 E{}", pos).into(),
            BlockingSetTemp(temp) => format!("M109 S{}", temp).into(),
        }
    }
}

/// Holds a value of type `T` for each axis
pub struct PerAxis<T> {
    map: HashMap<Axis, T>,
}

impl<T> PerAxis<T> {
    pub fn none() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set(mut self, axis: Axis, value: T) -> Self {
        self.map.insert(axis, value);
        self
    }

    fn entries(&self) -> impl Iterator<Item=(&Axis, &T)> {
        self.map.iter()
    }
}

pub fn slices_to_gcode(config: &ConfigProfile, slices: &[Slice]) -> String {
    let mut gcoder = GCodeBuilder::new(config);
    gcoder.add_starting_gcode();
    for slice in slices {
        gcoder.add_slice(slice);
    }
    gcoder.generate_gcode()
}

struct GCodeBuilder<'a> {
    commands: Vec<Command>,
    config: &'a ConfigProfile,
    top_height: i64,
}

impl<'a> GCodeBuilder<'a> {
    fn new(config: &'a ConfigProfile) -> Self {
        Self {
            commands: Vec::new(),
            config,
            top_height: 0,
        }
    }

    /// Insert a raw command
    fn command(&mut self, cmd: Command) {
        self.commands.push(cmd);
    }

    fn add_starting_gcode(&mut self) {
        self.command(Command::SetAbsolutePositioning);
        self.command(Command::Home(PerAxis::none()));
        self.command(Command::BlockingSetTemp(self.config.hotend_temperature));
    }

    /// Adds gcode to print the given slice
    fn add_slice(&mut self, slice: &Slice) {
        //FIXME: don't hardcode nm/mm conversion (200000)

        self.top_height += (slice.thickness() * 200_000) as i64;
        // increment z height
        self.command(Command::Move {
            speed: self.config.travel_speed,
            amounts: PerAxis::none()
                .set(Axis::Z, self.top_height),
        });

        for island in slice.islands() {
            //TODO: island holes
            for vertex in island.outline().vertices() {
                self.command(Command::ExtrudeMove {
                    speed: 1, //TODO
                    extrude_len: 1, //TODO
                    amounts: PerAxis::none()
                        .set(Axis::X, vertex.x * 200_000)
                        .set(Axis::Y, vertex.y * 200_000),
                })
            }
        }
    }

    fn generate_gcode(&self) -> String {
        let mut s = String::new();
        for cmd in self.commands.iter().map(Command::as_code) {
            s.push_str(&cmd);
            s.push('\n');
        }
        // remove trailing newline
        s.pop();
        s
    }
}
