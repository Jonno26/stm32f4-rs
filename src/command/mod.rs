#[derive(Debug, PartialEq)]
pub enum Command {
    GetTemp,
    GetHumidity,
    GetGpsCord,
    UnknownCommand,
}

impl Command {
    pub fn convert_from(int: u8) -> Self {
        match int {
            0 => Command::GetTemp,
            1 => Command::GetHumidity,
            2 => Command::GetGpsCord,
            _ => Command::UnknownCommand,
        }
    }
}
