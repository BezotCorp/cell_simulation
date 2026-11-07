#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    Pause,
    Step,
    Slower,
    Faster,
    Restart,
    Reload,
    Save,
    Load,
    CycleView,
    Focus,
}
