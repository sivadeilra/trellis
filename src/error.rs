#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    FoundCycle,
    EmptyGraph,
}
