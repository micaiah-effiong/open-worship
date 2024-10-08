// mod dto;

#[derive(Debug, Clone)]
pub struct Payload {
    pub text: String,
    pub position: u32,
    pub background_image: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListPayload {
    pub text: String,
    pub position: u32,
    pub list: Vec<String>,
    pub background_image: Option<String>,
}

impl ListPayload {
    pub fn new(
        text: String,
        position: u32,
        list: Vec<String>,
        background_image: Option<String>,
    ) -> ListPayload {
        return ListPayload {
            text,
            position,
            list,
            background_image,
        };
    }
}

#[derive(Debug, Clone)]
pub struct DisplayPayload {
    pub text: String,
    /// image src/filepath
    pub background_image: Option<String>,
}

impl DisplayPayload {
    pub fn new(text: String) -> Self {
        return DisplayPayload {
            text,
            background_image: None,
        };
    }
}
