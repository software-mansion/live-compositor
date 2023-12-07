pub struct ErrorStack<'a>(Option<&'a (dyn std::error::Error + 'static)>);

impl<'a> ErrorStack<'a> {
    pub fn new(value: &'a (dyn std::error::Error + 'static)) -> Self {
        ErrorStack(Some(value))
    }

    pub fn into_string(self) -> String {
        let stack: Vec<String> = self.map(ToString::to_string).collect();
        stack.join("\n")
    }
}

impl<'a> Iterator for ErrorStack<'a> {
    type Item = &'a (dyn std::error::Error + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.map(|err| {
            self.0 = err.source();
            err
        })
    }
}
