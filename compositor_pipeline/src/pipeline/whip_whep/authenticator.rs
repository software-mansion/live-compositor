use std::{collections::HashMap, thread, time::Duration};

use rand::{distributions::Alphanumeric, Rng};

pub struct Authenticator(HashMap<String, String>);

pub enum AuthenticationError {
    NotAuthorized,
    InvalidInputId,
}

impl Authenticator {
    fn new() -> Authenticator {
        Authenticator(HashMap::new())
    }

    pub fn register_token(&mut self, input_id: String, token: Option<String>) -> String {
        let token = match token {
            Some(token_str) => token_str,
            None => self.generate_token(),
        };
        self.set_token(input_id, &token);
        token
    }

    pub fn validate_token(
        &self,
        input_id: String,
        token: String,
    ) -> Result<(), AuthenticationError> {
        match self.0.get(&input_id) {
            Some(saved_token) => {
                if Authenticator::compare_strings(saved_token, &token) {
                    Ok(())
                } else {
                    Err(AuthenticationError::NotAuthorized)
                }
            }
            None => Err(AuthenticationError::InvalidInputId),
        }
    }

    fn compare_strings(string1: &String, string2: &String) -> bool {
        let mut rng = rand::thread_rng();
        let sleep_interval = rng.gen_range(100..400);
        thread::sleep(Duration::from_millis(sleep_interval));
        string1 == string2
    }

    fn set_token(&mut self, input_id: String, token: &String) {
        self.0.insert(input_id, token.clone());
    }

    fn generate_token(&mut self) -> String {
        let rng = rand::thread_rng();
        let token: String = rng
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        token
    }
}
