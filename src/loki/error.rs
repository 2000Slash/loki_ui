use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    pub source: Option<Box<dyn std::error::Error>>,
    pub message: String,
}

impl Error {
    pub fn new(message: String) -> Self {
        Self {
            source: None,
            message,
        }
    }

    pub fn with_source(source: Box<dyn std::error::Error>, message: String) -> Self {
        Self {
            source: Some(source),
            message,
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // return source
        match &self.source {
            Some(source) => Some(source.as_ref()),
            None => None,
        }
    }

    fn description(&self) -> &str {
        &self.message
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Iterate over source and print to_string()
        writeln!(f, "{}", self.message)?;
        if let Some(source) = &self.source {
            write!(f, "{}", source)?;
        }
        Ok(())
    }
}
