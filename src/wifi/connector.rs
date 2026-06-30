use super::credentials::WifiCredentials;
use super::parser::parse_wifi_credentials;

#[derive(Debug)]
pub enum WifiConnectError {
    MissingPassword,
    UnsupportedSecurity,
    CommandFailed(String),
    Io(std::io::Error),
    NoCredentialsFound,
}

pub trait WifiConnector {
    fn connect(&self, credentials: &WifiCredentials) -> Result<(), WifiConnectError>;
}
// trait = promise of behavior; any type that wants to be called WifiConnector
// must implement connect function; also could be like a shared interface of
// sorts where you could have a bunch of different connectors or functions that
// all have connect, so this shared interface makes it easier traits are used
// because want a high-level connector.connect(), not a massive if-else
// statement two types of impl, one is to add methods directly to a type, other
// is a trait impl, teaching a type how to satisfy a trait

impl From<std::io::Error> for WifiConnectError {
    // converting from one to another since connect returns WifiConnectError, so
    // .into() is auto. available then (the conversion ? uses)
    fn from(error: std::io::Error) -> Self {
        WifiConnectError::Io(error)
    }
}

pub fn connect_to_wifi_from_qr_payloads(
    qr_payloads: &[String],
    connector: &impl WifiConnector
) -> Result<(), WifiConnectError> {
    // &impl WifiConnector means give me a reference to any type that implements
    // WifiConnector (abstracting away what exact connector is being passed in)
    let credentials = parse_wifi_credentials(qr_payloads).ok_or(
        WifiConnectError::NoCredentialsFound
    )?;
    connector.connect(&credentials)
    // ? means if it is an error, return the error from the curr. function
    // immediately; works for Result or Option; can only use it if this function
    // itself returns a valid Result or Option because there's always a chance
    // it needs to return an error

    // .ok_or converts option to result
}
