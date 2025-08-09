#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum Input {
    MouseLeft,
    MouseMiddle,
    MouseRight,
    RestartGame,
    Save,
    Load,
    MoveN,
    MoveNW,
    MoveNE,
    MoveE,
    MoveW,
    MoveS,
    MoveSW,
    MoveSE,
    MoveSkip,
    Confirm,
    Cancel,
}
