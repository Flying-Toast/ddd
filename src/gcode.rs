use std::collections::HashMap;
use crate::slice::Slice;

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
        amounts: PerAxis<i32>,
        speed: u32,
    },
    /// Like `Move`, but extrudes filament during the move
    ExtrudeMove {
        amounts: PerAxis<i32>,
        speed: u32,
        /// How much filament to extrude during the move
        extrude_len: u32,
    },
    SetPosition(PerAxis<i32>),
    SetExtruderPosition(i32),
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

pub struct GCodeBuilder {
    commands: Vec<Command>,
}

impl GCodeBuilder {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Insert a raw command
    pub fn command(&mut self, cmd: Command) {
        self.commands.push(cmd);
    }

    /// Adds gcode to print the given slice
    pub fn add_slice(&mut self, slice: &Slice) {
        todo!();
    }

    pub fn generate_gcode(&self) -> String {
        self.commands.iter()
            .map(Command::as_code)
            .collect()
    }
}
