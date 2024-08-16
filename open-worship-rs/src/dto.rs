// mod dto;

#[derive(Debug, Clone)]
pub struct Payload {
    pub text: String,
    pub position: u32,
}

#[derive(Debug, Clone)]
pub struct ListPayload {
    pub text: String,
    pub position: u32,
    pub list: Vec<String>,
}
