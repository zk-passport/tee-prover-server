pub struct Proofs {
    id: String,
    status: Status,
    proof: sqlx::types::Json<serde_json::Value>,
}

pub enum Status {
    Pending,
    WitnessGenerated,
    Completed,
}

impl Status {
    pub fn to_int(&self) -> u8 {
        match self {
            Status::Pending => 0,
            Status::WitnessGenerated => 1,
            Status::Completed => 2,
        }
    }
}
