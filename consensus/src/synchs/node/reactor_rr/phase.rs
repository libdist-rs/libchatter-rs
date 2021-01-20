use types::View;

#[derive(Debug, PartialEq)]
pub enum Phase {
    /// Not a leader, and waiting for a proposal from the leader
    ProposeWait,
    /// The leader ready to propose
    Propose,
    /// Nodes waiting for f+1 votes on the proposal to quit the view and change
    /// the leader
    CollectVote,
    /// We have f+1 votes to quit the view, we are switching to the next view
    NextView(View),

    Status,
    StatusWait,
}