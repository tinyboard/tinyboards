use crate::{error::TinyBoardsError, settings::{structs::Settings}};
use html2text;
use lettre::{
    message::{Mailbox, MultiPart},
    transport::smtp::{authentication::Credentials, extension::ClientId},
    Address,
    Message,
    SmtpTransport,
    Transport,
};
use std::str::FromStr;
use uuid::Uuid;

pub fn send_email(
    subject: &str,
    to_email: &str,
    to_username: &str,
    html: &str,
    settings: &Settings,
) -> Result<(), TinyBoardsError> {
    let email_config = settings
        .email
        .to_owned()
        .ok_or_else(|| TinyBoardsError::from_string("no email setup", 500))?;
    
    let domain = settings.hostname.to_owned();

    let (smtp_server, smtp_port) = {
        let email_and_port = email_config.smtp_server.split(':').collect::<Vec<&str>>();
        if email_and_port.len() == 1 {
            return Err(
                TinyBoardsError::from_string(
                    "email.smtp_server needs a port, EX: smtp.example.com:401", 500)
            );
        }
        (
            email_and_port[0],
            email_and_port[1]
                .parse::<u16>()
                .expect("email needs a port"),
        )
    };

    let plain_text = html2text::from_read(html.as_bytes(), 78);

    let email = Message::builder()
        .from(
            email_config
                .smtp_from_address
                .parse()
                .expect("email from address is not valid.")
        )
        .to(Mailbox::new(
            Some(to_username.to_string()), 
            Address::from_str(to_email)
            .expect("email to address is not valid."))
        )
        .message_id(Some(format!("{}@{}", Uuid::new_v4(), settings.hostname)))
        .subject(subject)
        .multipart(MultiPart::alternative_plain_html(
            plain_text, 
            html.to_string(),
        ))
        .expect("email was not built correctly.");
    
    // set tls
    let builder_dangerous = SmtpTransport::builder_dangerous(smtp_server).port(smtp_port);

    let mut builder = match email_config.tls_type.as_str() {
        "starttls" => SmtpTransport::starttls_relay(smtp_server).map_err(|_| TinyBoardsError::err_500())?,
        "tls" => SmtpTransport::relay(smtp_server).map_err(|_| TinyBoardsError::err_500())?,
        _ => builder_dangerous,
    };

    if let (Some(username), Some(password)) = (email_config.smtp_login, email_config.smtp_password) {
        builder = builder.credentials(Credentials::new(username, password));
    }

    let mailer = builder.hello_name(ClientId::Domain(domain)).build();

    let result = mailer.send(&email);

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(TinyBoardsError::from_string(e.to_string().as_str(), 500)),
    }
}