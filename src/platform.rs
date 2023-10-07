pub mod agents;
pub mod message;
pub mod organization;

enum ErrorCode {
    NoError,
    Found,
    HandleNull,
    ListFull,
    Duplicated,
    NotFound,
    Timeout,
    Invalid,
    NotRegistered,
}