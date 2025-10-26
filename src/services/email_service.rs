use crate::models::{AppState, dj::GuestRequest};
use anyhow::Result;
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use std::sync::Arc;

pub struct EmailService {
    transport: SmtpTransport,
    from_address: Mailbox,
}

impl EmailService {
    pub fn new(app_state: Arc<AppState>) -> Result<Self> {
        let config = &app_state.config.email_config;
        
        let credentials = Credentials::new(
            config.username.clone(),
            config.password.clone(),
        );

        let transport = SmtpTransport::relay(&config.smtp_server)?
            .port(config.smtp_port)
            .credentials(credentials)
            .build();

        let from_address = config.from_address.parse::<Mailbox>()?;

        Ok(Self {
            transport,
            from_address,
        })
    }

    pub async fn send_guest_request_email(
        &self,
        dj_email: &str,
        dj_name: &str,
        guest_request: &GuestRequest,
    ) -> Result<()> {
        let subject = format!("Set Request from {}", guest_request.guest_name);
        
        let body = format!(
            r#"Hello {},

You have received a new set request from a guest:

Guest Name: {}
Guest Email: {}
Message: {}

You can respond directly to this email to contact the guest, or use the DJ system interface to approve/reject the request.

Best regards,
DJ Session Recorder System
"#,
            dj_name,
            guest_request.guest_name,
            guest_request.guest_email,
            guest_request.message.as_ref().unwrap_or(&"No message provided".to_string())
        );

        let email = Message::builder()
            .from(self.from_address.clone())
            .to(dj_email.parse::<Mailbox>()?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)?;

        match self.transport.send(&email) {
            Ok(_) => {
                tracing::info!("Guest request email sent to {} for DJ {}", dj_email, dj_name);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to send guest request email: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn send_session_download_link(
        &self,
        dj_email: &str,
        dj_name: &str,
        session_id: &str,
        download_link: &str,
    ) -> Result<()> {
        let subject = format!("Your DJ Set Recording is Ready - {}", session_id);
        
        let body = format!(
            r#"Hello {},

Your DJ set recording is now ready for download!

Session ID: {}
Download Link: {}

The link will be available for 30 days. You can download the file as many times as needed within this period.

Thank you for using our DJ Session Recorder system!

Best regards,
DJ Session Recorder System
"#,
            dj_name,
            session_id,
            download_link
        );

        let email = Message::builder()
            .from(self.from_address.clone())
            .to(dj_email.parse::<Mailbox>()?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)?;

        match self.transport.send(&email) {
            Ok(_) => {
                tracing::info!("Download link email sent to {} for session {}", dj_email, session_id);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to send download link email: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn send_lottery_winner_notification(
        &self,
        dj_email: &str,
        dj_name: &str,
        position: i32,
    ) -> Result<()> {
        let subject = "You've been selected in the DJ Lottery!";
        
        let body = format!(
            r#"Hello {},

Congratulations! You have been selected in the DJ lottery.

Your position in the queue: #{}
Estimated time: Please check the display for current queue status

Please be ready when your turn comes up. You can start your session using the DJ interface.

Good luck with your set!

Best regards,
DJ Session Recorder System
"#,
            dj_name,
            position
        );

        let email = Message::builder()
            .from(self.from_address.clone())
            .to(dj_email.parse::<Mailbox>()?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)?;

        match self.transport.send(&email) {
            Ok(_) => {
                tracing::info!("Lottery winner notification sent to {}", dj_email);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to send lottery winner notification: {}", e);
                Err(e.into())
            }
        }
    }

    pub async fn send_system_notification(
        &self,
        to_email: &str,
        subject: &str,
        message: &str,
    ) -> Result<()> {
        let email = Message::builder()
            .from(self.from_address.clone())
            .to(to_email.parse::<Mailbox>()?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(message.to_string())?;

        match self.transport.send(&email) {
            Ok(_) => {
                tracing::info!("System notification sent to {}: {}", to_email, subject);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to send system notification: {}", e);
                Err(e.into())
            }
        }
    }
}