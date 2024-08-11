use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<R> {
    status: u32,
    #[serde(flatten)]
    response: Option<R>,
    #[serde(default)]
    response_messages: Option<ResponseMessages>,
}

impl<T> ApiResponse<T> {
    pub(crate) fn new(response: T) -> Self {
        Self {
            status: 200,
            response: Some(response),
            response_messages: None,
        }
    }

    pub fn is_success(&self) -> bool {
        self.status == 200
    }

    pub fn get_messages(&self) -> Option<&ResponseMessages> {
        self.response_messages.as_ref()
    }

    pub fn response(&self) -> Option<&T> {
        self.response.as_ref()
    }

    pub fn response_owned(self) -> Option<T> {
        self.response
    }

    pub fn response_unsafe(&self) -> &T {
        self.response().unwrap()
    }

    pub fn response_unsafe_owned(self) -> T {
        self.response_owned().unwrap()
    }
}

#[derive(Deserialize, Debug)]
pub struct ResponseMessages {
    #[serde(default)]
    error: Vec<Message>,
    #[serde(default)]
    warning: Vec<Message>,
    #[serde(default)]
    info: Vec<Message>,
}

impl ResponseMessages {
    pub fn errors(&self) -> &[Message] {
        &self.error
    }

    pub fn warnings(&self) -> &[Message] {
        &self.warning
    }

    pub fn infos(&self) -> &[Message] {
        &self.info
    }
}

#[derive(Deserialize, Debug)]
pub struct Message {
    message: String,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    hint: Option<String>,
}

impl Message {
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }
}
