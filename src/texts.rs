use std::fmt::Display;

pub enum SecuredDisplayText {
    Secured,
    SecuredPq,
    Blocked,
    Securing,
    SecuringPq,
    Unsecured,
    Unsecuring,
    FailedToSecure,
}

impl SecuredDisplayText {
    pub fn as_str(&self) -> &'static str {
        use SecuredDisplayText::*;
        match self {
            Secured => "SECURE CONNECTION",
            SecuredPq => "QUANTUM SECURE CONNECTION",
            Blocked => "BLOCKED CONNECTION",
            Securing => "CREATING SECURE CONNECTION",
            SecuringPq => "CREATING QUANTUM SECURE CONNECTION",
            Unsecured => "UNSECURED CONNECTION",
            Unsecuring => "",
            FailedToSecure => "FAILED TO SECURE CONNECTION",
        }
    }
}

impl Display for SecuredDisplayText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
