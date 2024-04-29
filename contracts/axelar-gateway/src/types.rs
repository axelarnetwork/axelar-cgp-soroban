use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandType {
    ApproveMessages,
    RotateSigners,
}
