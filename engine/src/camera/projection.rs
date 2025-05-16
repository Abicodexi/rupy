#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub enum Projection {
    #[default]
    FirstPerson,
    ThirdPerson,
}
