use std::collections::HashMap;
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
    fn as_code(&self) -> String {
        use Command::*;
        match self {
            Home(axes) => format!(
                "G28{}",
                axes.entries()
                    .map(|(axis, _)| format!(" {}", axis.to_str()))
                    .collect::<String>(),
            ),
            SetAbsolutePositioning => "G90".into(),
            SetRelativePositioning => "G91".into(),
            Move { amounts, speed } => format!(
                "G1 {}F{}",
                amounts.entries()
                    .map(|(axis, amnt)| format!("{}{} ", axis.to_str(), amnt))
                    .collect::<String>(),
                speed,
            ),
            ExtrudeMove { amounts, speed, extrude_len } => format!(
                "G1 {}E{} F{}",
                amounts.entries()
                    .map(|(axis, amnt)| format!("{}{} ", axis.to_str(), amnt))
                    .collect::<String>(),
                extrude_len,
                speed,
            ),
            SetPosition(pozs) => format!(
                "G92{}",
                pozs.entries()
                    .map(|(axis, pos)| format!(" {}{}", axis.to_str(), pos))
                    .collect::<String>(),
            ),
            SetExtruderPosition(pos) => format!("G92 E{}", pos),
            BlockingSetTemp(temp) => format!("M109 S{}", temp),
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
}

impl<'a> GCodeBuilder<'a> {
    fn new(config: &'a ConfigProfile) -> Self {
        Self {
            commands: Vec::new(),
            config,
        }
    }

    /// Insert a raw command
    fn command(&mut self, cmd: Command) {
        self.commands.push(cmd);
    }

    fn add_starting_gcode(&mut self) {
        self.command(Command::SetRelativePositioning);
        self.command(Command::Home(PerAxis::none()));
        self.command(Command::BlockingSetTemp(self.config.hotend_temperature));
    }

    /// Adds gcode to print the given slice
    fn add_slice(&mut self, slice: &Slice) {
        // increment z height
        self.command(Command::Move {
            speed: self.config.travel_speed,
            amounts: PerAxis::none()
                .set(Axis::Z, slice.thickness() as i64),
        });
        todo!();
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
