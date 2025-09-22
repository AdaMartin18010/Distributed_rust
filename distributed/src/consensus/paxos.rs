#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusRole {
    Leader,
    Follower,
    Candidate,
}

pub trait ConsensusApi {
    fn role(&self) -> ConsensusRole;
}
